use addrslips::Pipeline;
use addrslips::detection::steps::*;
use image::ImageReader;

fn main() -> anyhow::Result<()> {
    let img = ImageReader::open("image.png")?
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

    println!("Testing composable pipeline with Vec<PipelineData>...\n");

    // Build a pipeline without OCR (faster for testing)
    let mut pipeline = Pipeline::new()
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

    // Run pipeline without OCR
    println!("\n=== Running Detection Pipeline (No OCR) ===");
    let result = pipeline.run(img.clone())?;

    println!("\n✓ Pipeline completed successfully!");
    println!("  Detected {} white circles", result.len());

    // Show first 5 circles
    println!("\nFirst 5 circles:");
    for (i, item) in result.iter().take(5).enumerate() {
        let radius = item.get_float("radius").unwrap_or(0.0);
        let brightness = item.get_float("brightness").unwrap_or(0.0);
        let is_circle = item.get_bool("is_circle").unwrap_or(false);
        let is_white = item.get_bool("is_white").unwrap_or(false);

        println!("  Circle {}: radius={:.1}, brightness={:.1}, is_circle={}, is_white={}",
                i + 1, radius, brightness, is_circle, is_white);

        if let Some(bbox) = &item.bbox {
            println!("    bbox: ({}, {}) {}x{}", bbox.x, bbox.y, bbox.width, bbox.height);
        }
    }

    // Demonstrate composability: create a custom pipeline with different parameters
    println!("\n\n=== Custom Pipeline with Stricter Parameters ===");
    let mut custom_pipeline = Pipeline::new()
        .with_verbose(false)
        .add_step_boxed(Box::new(GrayscaleStep))
        .add_step_boxed(Box::new(BlurStep { sigma: 2.0 }))  // More blur
        .add_step_boxed(Box::new(EdgeDetectionStep {
            low_threshold: 60.0,
            high_threshold: 120.0,
        }))
        .add_step_boxed(Box::new(ContourDetectionStep { min_area: 20, padding: 10 }))
        .add_step_boxed(Box::new(CircleFilterStep {
            min_radius: 15.0,  // Stricter minimum
            max_radius: 150.0,
            circularity_threshold: 1.5,  // More circular
        }))
        .add_step_boxed(Box::new(WhiteCircleFilterStep {
            brightness_threshold: 210.0,  // Whiter
        }));

    let custom_result = custom_pipeline.run(img)?;

    println!("✓ Custom pipeline completed!");
    println!("  Detected {} white circles (with stricter parameters)", custom_result.len());

    Ok(())
}
