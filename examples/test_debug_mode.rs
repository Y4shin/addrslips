use addrslips::Pipeline;
use addrslips::detection::steps::*;
use image::ImageReader;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let img = ImageReader::open("image.png")?
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

    println!("Testing debug mode with lineage tracking...\n");

    // Create debug output directory
    let debug_dir = PathBuf::from("debug_output");

    // Remove directory if it exists (for testing)
    if debug_dir.exists() {
        std::fs::remove_dir_all(&debug_dir)?;
    }

    // Build a pipeline with debug mode enabled
    let mut pipeline = Pipeline::new()
        .with_verbose(true)
        .with_debug(debug_dir.clone())?
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

    println!("Running pipeline with debug mode...");
    let result = pipeline.run(img)?;

    println!("\n✓ Pipeline completed!");
    println!("  Detected {} white circles", result.len());
    println!("\nDebug outputs saved to: {}/", debug_dir.display());
    println!("\nDirectory structure:");
    println!("  00_input/          - Original input image");
    println!("  01_grayscale_conversion/ - After grayscale conversion");
    println!("  02_gaussian_blur/  - After blur");
    println!("  03_edge_detection/ - Edge detected image");
    println!("  04_contour_detection/ - Each detected contour (1→100 images)");
    println!("  05_circle_filtering/ - Circular contours only (100→40 images)");
    println!("  06_white_circle_filtering/ - White circles only (~40 images)");

    // List some files to show the structure
    println!("\nExample files:");
    if let Ok(entries) = std::fs::read_dir(&debug_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap().to_string_lossy();
                if let Ok(files) = std::fs::read_dir(&path) {
                    let file_count = files.count();
                    println!("  {}/ - {} files", dir_name, file_count);

                    // Show first 3 files as examples
                    if let Ok(files) = std::fs::read_dir(&path) {
                        for (i, file) in files.flatten().take(3).enumerate() {
                            let filename = file.file_name();
                            println!("    {}", filename.to_string_lossy());
                            if i == 2 {
                                if let Ok(total_files) = std::fs::read_dir(&path) {
                                    let count = total_files.count();
                                    if count > 3 {
                                        println!("    ... and {} more", count - 3);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
