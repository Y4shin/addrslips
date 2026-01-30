use clap::Parser;
use image::ImageReader;
use std::path::PathBuf;

use addrslips::Pipeline;
use addrslips::detection::steps::*;

#[derive(Parser)]
#[command(name = "addrslips")]
#[command(about = "Detect and read house numbers from images")]
struct Cli {
    /// Path to input image file
    #[arg(value_name = "IMAGE")]
    image_path: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Save debug outputs to directory (must be empty)
    #[arg(long, value_name = "DIR")]
    debug_out: Option<PathBuf>,

    /// Skip OCR step (faster, for testing circle detection only)
    #[arg(long)]
    skip_ocr: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.verbose {
        println!("Loading image: {:?}", args.image_path);
    }

    // Load image
    let img = ImageReader::open(&args.image_path)?
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

    if args.verbose {
        println!("Image loaded: {}x{}\n", img.width(), img.height());
    }

    // Build pipeline
    let mut pipeline_builder = Pipeline::new()
        .with_verbose(args.verbose)
        .add_step_boxed(Box::new(GrayscaleStep))
        .add_step_boxed(Box::new(BlurStep { sigma: 1.5 }))
        .add_step_boxed(Box::new(EdgeDetectionStep {
            low_threshold: 50.0,
            high_threshold: 100.0,
        }))
        .add_step_boxed(Box::new(ContourDetectionStep {
            min_area: 10,
            padding: 10,
        }))
        .add_step_boxed(Box::new(CircleFilterStep {
            min_radius: 10.0,
            max_radius: 200.0,
            circularity_threshold: 2.0,
        }))
        .add_step_boxed(Box::new(WhiteCircleFilterStep {
            brightness_threshold: 200.0,
        }))
        .add_step_boxed(Box::new(BackgroundRemovalStep))
        .add_step_boxed(Box::new(UpscaleStep { target_size: 100 }));
        // Sharpening removed - doesn't seem to improve OCR results

    // Add OCR step unless skipped
    if !args.skip_ocr {
        pipeline_builder = pipeline_builder
            .add_step_boxed(Box::new(OcrStep::new()));
    }

    // Enable debug mode if requested
    if let Some(debug_dir) = args.debug_out {
        pipeline_builder = pipeline_builder.with_debug(debug_dir)?;
    }

    // Run pipeline with executor (always)
    if args.verbose {
        println!("Running pipeline...\n");
    }
    let results = pipeline_builder.run_with_executor(img)?;

    // Print results
    if args.skip_ocr {
        println!("\n=== White Circle Detection Results ===");
        println!("Total white circles detected: {}", results.len());

        if !results.is_empty() && args.verbose {
            println!("\nDetected circles:");
            for (i, item) in results.iter().enumerate() {
                if let Some(bbox) = &item.bbox {
                    let brightness = item.get_float("brightness").unwrap_or(0.0);
                    println!("  Circle {} at ({}, {}) - brightness: {:.1}",
                            i + 1, bbox.x, bbox.y, brightness);
                }
            }
        }
    } else {
        println!("\n=== House Number Detection Results ===");
        println!("Total detections: {}", results.len());

        if results.is_empty() {
            println!("No house numbers detected.");
        } else {
            println!("\nDetected house numbers:");
            for item in &results {
                if let (Some(text), Some(confidence)) = (
                    item.get_string("ocr_text"),
                    item.get_float("ocr_confidence")
                ) {
                    if let Some(bbox) = &item.bbox {
                        println!("  {} at ({}, {}) - confidence: {:.2}",
                                text, bbox.x, bbox.y, confidence);
                    }
                }
            }
        }
    }

    Ok(())
}
