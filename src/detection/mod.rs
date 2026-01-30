pub mod preprocessing;
pub mod contours;
pub mod circles;
pub mod ocr;
pub mod steps;

use image::DynamicImage;
use crate::models::{Contour, HouseNumberDetection};

/// Main detection pipeline orchestrator
pub struct DetectionPipeline {
    // Detection parameters
    pub min_radius: f32,
    pub max_radius: f32,
    pub circularity_threshold: f32,
    pub brightness_threshold: f32,
    pub verbose: bool,
}

impl DetectionPipeline {
    pub fn new() -> Self {
        Self {
            min_radius: 10.0,
            max_radius: 200.0,
            circularity_threshold: 2.0,
            brightness_threshold: 200.0,
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Run the full detection pipeline on an image
    pub fn detect(&self, img: &DynamicImage) -> anyhow::Result<Vec<HouseNumberDetection>> {
        // Step 1: Preprocess image
        if self.verbose {
            println!("\nPreprocessing image...");
            println!("Converting to grayscale...");
        }
        let gray = preprocessing::to_grayscale(img);

        if self.verbose {
            println!("Applying Gaussian blur...");
        }
        let blurred = preprocessing::apply_blur(&gray, 1.5);

        // Step 2: Detect edges
        if self.verbose {
            println!("\nDetecting edges...");
        }
        let edges = preprocessing::detect_edges(&blurred, 50.0, 100.0);

        // Step 3: Find contours
        if self.verbose {
            println!("\nFinding contours...");
        }
        let all_contours = contours::find_contours(&edges, 10);

        if self.verbose {
            println!("Found {} contours", all_contours.len());
        }

        // Step 4: Filter for circular shapes
        if self.verbose {
            println!("\nFiltering for circular shapes...");
            println!("Analyzing contours (showing first 10):");
            for (i, contour) in all_contours.iter().take(10).enumerate() {
                println!("  Contour {}: radius={:.1}, circ={:.3}, aspect={:.2}, pixels={}",
                        i + 1, contour.radius(), contour.circularity(),
                        contour.aspect_ratio(), contour.area());
            }
        }

        let circular_contours = circles::filter_circles(
            &all_contours,
            self.min_radius,
            self.max_radius,
            self.circularity_threshold,
        );

        if self.verbose {
            println!("Found {} circular shapes (from {} total contours)",
                    circular_contours.len(), all_contours.len());
        }

        // Step 5: Filter for white circles
        if self.verbose {
            println!("\nFiltering for white circles...");
            println!("Analyzing brightness (showing first 5):");
            for (i, circle) in circular_contours.iter().take(5).enumerate() {
                let brightness = circle.average_brightness(img);
                println!("  Circle {}: brightness={:.1}/255", i + 1, brightness);
            }
        }

        let white_circles = circles::filter_white_circles(
            &circular_contours,
            img,
            self.brightness_threshold,
        );

        if self.verbose {
            println!("Found {} white circles (from {} circular shapes)",
                    white_circles.len(), circular_contours.len());

            if !white_circles.is_empty() {
                println!("Example white circles:");
                for (i, circle) in white_circles.iter().take(5).enumerate() {
                    println!("  Circle {}: radius={:.1}, brightness={:.1}",
                            i + 1, circle.radius(), circle.average_brightness(img));
                }
            }
        }

        // Step 6: Run OCR on white circles
        if white_circles.is_empty() {
            return Ok(Vec::new());
        }

        if self.verbose {
            println!("\nInitializing OCR engine...");
        }

        let ocr_engine = ocr::init_ocr_engine()?;

        if self.verbose {
            println!("OCR engine initialized successfully");
            println!("\nRunning OCR on {} white circles...", white_circles.len());
        }

        let mut detections = Vec::new();

        for (i, circle) in white_circles.iter().enumerate() {
            if self.verbose {
                println!("  Processing circle {} of {}...", i + 1, white_circles.len());
            }

            if let Some(roi) = circle.extract_roi(img) {
                if let Some((text, confidence)) = ocr::recognize_house_number(&ocr_engine, &roi) {
                    let (x, y) = circle.center();
                    detections.push(HouseNumberDetection {
                        number: text.clone(),
                        x,
                        y,
                        confidence,
                    });

                    if self.verbose {
                        println!("    Detected: '{}' (confidence: {:.2})", text, confidence);
                    }
                } else if self.verbose {
                    println!("    No text detected");
                }
            } else if self.verbose {
                println!("    Failed to extract ROI");
            }
        }

        Ok(detections)
    }

    /// Get all contours from an image (for debugging)
    pub fn get_contours(&self, img: &DynamicImage) -> anyhow::Result<Vec<Contour>> {
        let gray = preprocessing::to_grayscale(img);
        let blurred = preprocessing::apply_blur(&gray, 1.5);
        let edges = preprocessing::detect_edges(&blurred, 50.0, 100.0);
        Ok(contours::find_contours(&edges, 10))
    }

    /// Get circular contours from an image (for debugging)
    pub fn get_circles(&self, img: &DynamicImage) -> anyhow::Result<Vec<Contour>> {
        let all_contours = self.get_contours(img)?;
        Ok(circles::filter_circles(
            &all_contours,
            self.min_radius,
            self.max_radius,
            self.circularity_threshold,
        ))
    }

    /// Get white circles from an image (for debugging)
    pub fn get_white_circles(&self, img: &DynamicImage) -> anyhow::Result<Vec<Contour>> {
        let circular_contours = self.get_circles(img)?;
        Ok(circles::filter_white_circles(
            &circular_contours,
            img,
            self.brightness_threshold,
        ))
    }
}

impl Default for DetectionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a standard detection pipeline using the composable pipeline system
pub fn build_standard_pipeline(verbose: bool) -> crate::pipeline::Pipeline {
    use crate::pipeline::Pipeline;
    use crate::detection::steps::*;
    use std::sync::Arc;

    Pipeline::new()
        .with_verbose(verbose)
        .add_step(Arc::new(GrayscaleStep))
        .add_step(Arc::new(BlurStep { sigma: 1.5 }))
        .add_step(Arc::new(EdgeDetectionStep {
            low_threshold: 50.0,
            high_threshold: 100.0,
        }))
        .add_step(Arc::new(ContourDetectionStep { min_area: 10, padding: 10 }))
        .add_step(Arc::new(CircleFilterStep {
            min_radius: 10.0,
            max_radius: 200.0,
            circularity_threshold: 2.0,
        }))
        .add_step(Arc::new(WhiteCircleFilterStep {
            brightness_threshold: 200.0,
        }))
        .add_step(Arc::new(BackgroundRemovalStep))
        .add_step(Arc::new(UpscaleStep { target_size: 100 }))
        // Sharpening removed - doesn't improve OCR results
        .add_step(Arc::new(OcrStep::new()))
}
