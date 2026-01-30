use addrslips::Pipeline;
use addrslips::detection::steps::*;
use image::ImageReader;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let img = ImageReader::open("image.png")?
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

    println!("Testing debug mode with lineage tracking (executor)...\n");

    // Create debug output directory
    let debug_dir = PathBuf::from("debug_lineage");

    // Remove directory if it exists (for testing)
    if debug_dir.exists() {
        std::fs::remove_dir_all(&debug_dir)?;
    }

    // Build a pipeline with debug mode enabled
    let pipeline = Pipeline::new()
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

    println!("Running pipeline with executor (lineage tracking)...");
    let result = pipeline.run_with_executor(img)?;

    println!("\n✓ Pipeline completed!");
    println!("  Detected {} white circles", result.len());
    println!("\nDebug outputs with lineage tracking saved to: {}/", debug_dir.display());

    println!("\nLineage explanation:");
    println!("  - Filenames show the path through the pipeline");
    println!("  - Format: parent1-parent2-...-current.png");
    println!("  - Example: 01-03-02.png means:");
    println!("    • Item 1 from step N-2");
    println!("    • → produced item 3 in step N-1");
    println!("    • → which produced item 2 in step N");

    // Show some example lineage files
    println!("\nExample lineage in contour detection (step splits 1→100):");
    if let Ok(files) = std::fs::read_dir(debug_dir.join("04_contour_detection")) {
        let mut filenames: Vec<_> = files
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        filenames.sort();
        for filename in filenames.iter().take(5) {
            println!("  {}", filename);
        }
        if filenames.len() > 5 {
            println!("  ... {} more files", filenames.len() - 5);
        }
    }

    println!("\nExample lineage in circle filtering (filtering step):");
    if let Ok(files) = std::fs::read_dir(debug_dir.join("05_circle_filtering")) {
        let mut filenames: Vec<_> = files
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        filenames.sort();
        for filename in filenames.iter().take(5) {
            println!("  {}", filename);
        }
        if filenames.len() > 5 {
            println!("  ... {} more files", filenames.len() - 5);
        }
    }

    Ok(())
}
