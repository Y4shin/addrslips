use anyhow::Result;
use clap::{Parser, Subcommand};
use image::ImageReader;
use std::path::PathBuf;

use addrslips::Pipeline;
use addrslips::detection::steps::*;

#[derive(Parser)]
#[command(name = "addrslips")]
#[command(about = "Campaign canvassing address management tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch GUI application
    Gui,
    /// Run CLI detection (existing functionality)
    Detect {
        /// Path to input image file
        #[arg(help = "Path to input image file")]
        image: PathBuf,
        /// Enable verbose output
        #[arg(short, long, help = "Enable verbose output")]
        verbose: bool,
        /// Save debug outputs to directory
        #[arg(long, help = "Save debug outputs to directory")]
        debug_out: Option<PathBuf>,
        /// Skip OCR step
        #[arg(long, help = "Skip OCR step")]
        skip_ocr: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Gui) | None => {
            // Default to GUI if no command specified
            #[cfg(feature = "gui")]
            {
                gui::run()?;
            }
            #[cfg(not(feature = "gui"))]
            {
                eprintln!("GUI not available in this build");
                std::process::exit(1);
            }
        }
        Some(Commands::Detect {
            image,
            verbose,
            debug_out,
            skip_ocr,
        }) => {
            run_cli_detection(image, verbose, debug_out, skip_ocr)?;
        }
    }

    Ok(())
}

#[cfg(feature = "gui")]
mod gui {
    use anyhow::Result;
    use iced::Application;

    pub fn run() -> Result<()> {
        use addrslips::gui::AddrslipsApp;
        iced::application(
            AddrslipsApp::default,
            AddrslipsApp::update,
            AddrslipsApp::view,
        )
        .title(AddrslipsApp::title)
        .theme(AddrslipsApp::theme)
        .centered()
        .run()
        .map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

        Ok(())
    }
}

fn run_cli_detection(
    image_path: PathBuf,
    verbose: bool,
    debug_out: Option<PathBuf>,
    skip_ocr: bool,
) -> Result<()> {
    if verbose {
        println!("Loading image: {:?}", image_path);
    }

    // Load image
    let img = ImageReader::open(&image_path)?
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

    if verbose {
        println!("Image loaded: {}x{}\n", img.width(), img.height());
    }

    // Build pipeline
    let mut pipeline_builder = Pipeline::new()
        .with_verbose(verbose)
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
    if !skip_ocr {
        pipeline_builder = pipeline_builder.add_step_boxed(Box::new(OcrStep::new()));
    }

    // Enable debug mode if requested
    if let Some(debug_dir) = debug_out {
        pipeline_builder = pipeline_builder.with_debug(debug_dir)?;
    }

    // Run pipeline with executor (always)
    if verbose {
        println!("Running pipeline...\n");
    }
    let results = pipeline_builder.run_with_executor(img)?;

    // Print results
    if skip_ocr {
        println!("\n=== White Circle Detection Results ===");
        println!("Total white circles detected: {}", results.len());

        if !results.is_empty() && verbose {
            println!("\nDetected circles:");
            for (i, item) in results.iter().enumerate() {
                if let Some(bbox) = &item.bbox {
                    let brightness = item.get_float("brightness").unwrap_or(0.0);
                    println!(
                        "  Circle {} at ({}, {}) - brightness: {:.1}",
                        i + 1,
                        bbox.x,
                        bbox.y,
                        brightness
                    );
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
                    item.get_float("ocr_confidence"),
                ) {
                    if let Some(bbox) = &item.bbox {
                        println!(
                            "  {} at ({}, {}) - confidence: {:.2}",
                            text, bbox.x, bbox.y, confidence
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
