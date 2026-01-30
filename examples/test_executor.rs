use addrslips::Pipeline;
use addrslips::detection::steps::*;
use image::ImageReader;

fn main() -> anyhow::Result<()> {
    let img = ImageReader::open("image.png")?
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

    println!("Testing pipeline executor with work queue...\n");

    // Build a pipeline
    let pipeline = Pipeline::new()
        .with_verbose(false)
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

    println!("Running with executor (work queue)...");
    let start = std::time::Instant::now();
    let result = pipeline.run_with_executor(img.clone())?;
    let executor_time = start.elapsed();

    println!("✓ Executor completed in {:?}", executor_time);
    println!("  Detected {} white circles", result.len());

    // Compare with sequential execution
    println!("\nRunning with sequential execution...");
    let start = std::time::Instant::now();
    let mut pipeline_seq = pipeline;
    let result_seq = pipeline_seq.run(img)?;
    let sequential_time = start.elapsed();

    println!("✓ Sequential completed in {:?}", sequential_time);
    println!("  Detected {} white circles", result_seq.len());

    println!("\nExecution time comparison:");
    println!("  Executor:   {:?}", executor_time);
    println!("  Sequential: {:?}", sequential_time);

    Ok(())
}
