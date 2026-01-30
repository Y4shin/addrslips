use addrslips::Pipeline;
use addrslips::detection::steps::*;
use image::ImageReader;
use std::env;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image_path>", args[0]);
        std::process::exit(1);
    }

    let image_path = &args[1];
    let img = ImageReader::open(image_path)?
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

    println!("Loaded image: {}x{}", img.width(), img.height());

    // Example 1: Standard pipeline (without OCR for faster demo)
    println!("\n=== Standard Detection Pipeline ===");
    let mut standard_pipeline = Pipeline::new()
        .with_verbose(true)
        .add_step_boxed(Box::new(GrayscaleStep))
        .add_step_boxed(Box::new(BlurStep { sigma: 1.5 }))
        .add_step_boxed(Box::new(EdgeDetectionStep {
            low_threshold: 50.0,
            high_threshold: 100.0,
        }))
        .add_step_boxed(Box::new(ContourDetectionStep { min_area: 10, padding: 10 }))
        .add_step_boxed(Box::new(CircleFilterStep {
            min_radius: 10.0,
            max_radius: 200.0,
            circularity_threshold: 2.0,
        }))
        .add_step_boxed(Box::new(WhiteCircleFilterStep {
            brightness_threshold: 200.0,
        }));

    let detections = standard_pipeline.run(img.clone())?;

    println!("\n=== Results ===");
    println!("Total detections: {}", detections.len());
    for (i, detection) in detections.iter().take(10).enumerate() {
        let text = detection.get_string("ocr_text").unwrap_or("N/A");
        let confidence = detection.get_float("ocr_confidence").unwrap_or(0.0);
        let radius = detection.get_float("radius").unwrap_or(0.0);

        if let Some(bbox) = &detection.bbox {
            println!("  {}: '{}' (conf: {:.2}) at ({}, {}) radius={:.1}",
                    i + 1, text, confidence, bbox.x, bbox.y, radius);
        }
    }

    // Example 2: Custom pipeline with modified parameters
    println!("\n\n=== Custom Pipeline (Stricter Circle Filter) ===");
    let mut custom_pipeline = Pipeline::new()
        .with_verbose(false)
        .add_step_boxed(Box::new(GrayscaleStep))
        .add_step_boxed(Box::new(BlurStep { sigma: 2.0 }))  // More blur
        .add_step_boxed(Box::new(EdgeDetectionStep {
            low_threshold: 40.0,  // Lower threshold
            high_threshold: 120.0,
        }))
        .add_step_boxed(Box::new(ContourDetectionStep { min_area: 20, padding: 10 }))  // Larger min area
        .add_step_boxed(Box::new(CircleFilterStep {
            min_radius: 15.0,  // Larger minimum
            max_radius: 150.0,
            circularity_threshold: 1.5,  // Stricter
        }))
        .add_step_boxed(Box::new(WhiteCircleFilterStep {
            brightness_threshold: 210.0,  // Whiter
        }));

    let custom_detections = custom_pipeline.run(img.clone())?;
    println!("Custom pipeline found {} circles", custom_detections.len());

    // Example 3: Pipeline with only first 3 steps (partial execution for debugging)
    println!("\n\n=== Partial Pipeline (Stop After Edge Detection) ===");
    let mut partial_pipeline = Pipeline::new()
        .with_verbose(false)
        .add_step_boxed(Box::new(GrayscaleStep))
        .add_step_boxed(Box::new(BlurStep { sigma: 1.5 }))
        .add_step_boxed(Box::new(EdgeDetectionStep {
            low_threshold: 50.0,
            high_threshold: 100.0,
        }));

    let partial_result = partial_pipeline.run(img)?;
    println!("Partial pipeline returned {} items", partial_result.len());
    if let Some(first) = partial_result.first() {
        println!("  First item: {}x{} image",
                first.image.width(), first.image.height());

        // Could save this for debugging:
        // first.image.save("debug_edges.png")?;
    }

    Ok(())
}
