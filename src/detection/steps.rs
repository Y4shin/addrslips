use crate::pipeline::{PipelineData, PipelineStep, PipelineContext, BoundingBox, MetadataValue};
use crate::detection::{preprocessing, contours, ocr};
use crate::models::Contour;
use anyhow::Result;
use image::GenericImageView;
use std::sync::{Arc, Mutex};

/// Convert image to grayscale
pub struct GrayscaleStep;

impl PipelineStep for GrayscaleStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext) -> Result<Vec<PipelineData>> {
        let mut result = Vec::new();
        for item in data {
            let gray = preprocessing::to_grayscale(&item.image);
            let new_item = PipelineData {
                image: image::DynamicImage::ImageLuma8(gray),
                original: item.original.clone(),
                bbox: item.bbox.clone(),
                metadata: item.metadata.clone(),
            };
            result.push(new_item);
        }
        Ok(result)
    }

    fn name(&self) -> &str {
        "Grayscale Conversion"
    }
}

/// Apply Gaussian blur
pub struct BlurStep {
    pub sigma: f32,
}

impl PipelineStep for BlurStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext) -> Result<Vec<PipelineData>> {
        let mut result = Vec::new();
        for item in data {
            let gray = item.image.to_luma8();
            let blurred = preprocessing::apply_blur(&gray, self.sigma);
            let new_item = PipelineData {
                image: image::DynamicImage::ImageLuma8(blurred),
                original: item.original.clone(),
                bbox: item.bbox.clone(),
                metadata: item.metadata.clone(),
            };
            result.push(new_item);
        }
        Ok(result)
    }

    fn name(&self) -> &str {
        "Gaussian Blur"
    }
}

/// Detect edges using Canny
pub struct EdgeDetectionStep {
    pub low_threshold: f32,
    pub high_threshold: f32,
}

impl PipelineStep for EdgeDetectionStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext) -> Result<Vec<PipelineData>> {
        let mut result = Vec::new();
        for item in data {
            let gray = item.image.to_luma8();
            let edges = preprocessing::detect_edges(&gray, self.low_threshold, self.high_threshold);
            let new_item = PipelineData {
                image: image::DynamicImage::ImageLuma8(edges),
                original: item.original.clone(),
                bbox: item.bbox.clone(),
                metadata: item.metadata.clone(),
            };
            result.push(new_item);
        }
        Ok(result)
    }

    fn name(&self) -> &str {
        "Edge Detection"
    }
}

/// Find contours in edge image - splits one image into many regions
pub struct ContourDetectionStep {
    pub min_area: u32,
    pub padding: u32,
}

impl PipelineStep for ContourDetectionStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext) -> Result<Vec<PipelineData>> {
        let mut result = Vec::new();

        for item in data {
            let gray = item.image.to_luma8();
            let detected_contours = contours::find_contours(&gray, self.min_area);
            let (img_width, img_height) = item.original.as_ref().dimensions();

            // Each contour becomes its own PipelineData
            for contour in detected_contours {
                // Add padding around the contour to avoid cutting off edges

                // Calculate padded bounding box, clamped to image boundaries
                let padded_x = contour.min_x.saturating_sub(self.padding);
                let padded_y = contour.min_y.saturating_sub(self.padding);
                let padded_max_x = (contour.max_x + self.padding).min(img_width - 1);
                let padded_max_y = (contour.max_y + self.padding).min(img_height - 1);

                let bbox = BoundingBox {
                    x: padded_x,
                    y: padded_y,
                    width: padded_max_x - padded_x + 1,
                    height: padded_max_y - padded_y + 1,
                };

                // Crop the region from the original image with padding
                let cropped = item.original.crop_imm(
                    bbox.x,
                    bbox.y,
                    bbox.width,
                    bbox.height
                );

                // Store contour information in metadata
                let mut contour_data = PipelineData::from_region(
                    cropped,
                    item.original.clone(),
                    bbox,
                );
                contour_data.metadata.insert("contour_min_x".to_string(), MetadataValue::Int(contour.min_x as i32));
                contour_data.metadata.insert("contour_min_y".to_string(), MetadataValue::Int(contour.min_y as i32));
                contour_data.metadata.insert("contour_max_x".to_string(), MetadataValue::Int(contour.max_x as i32));
                contour_data.metadata.insert("contour_max_y".to_string(), MetadataValue::Int(contour.max_y as i32));
                contour_data.metadata.insert("pixel_count".to_string(), MetadataValue::Int(contour.pixel_count as i32));
                contour_data.metadata.insert("radius".to_string(), MetadataValue::Float(contour.radius()));
                contour_data.metadata.insert("circularity".to_string(), MetadataValue::Float(contour.circularity()));
                contour_data.metadata.insert("aspect_ratio".to_string(), MetadataValue::Float(contour.aspect_ratio()));

                result.push(contour_data);
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "Contour Detection"
    }
}

/// Filter contours to keep only circular shapes
pub struct CircleFilterStep {
    pub min_radius: f32,
    pub max_radius: f32,
    pub circularity_threshold: f32,
}

impl PipelineStep for CircleFilterStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext) -> Result<Vec<PipelineData>> {
        let mut result = Vec::new();

        for item in data {
            // Extract contour properties from metadata
            let circularity = item.get_float("circularity").unwrap_or(999.0);
            let radius = item.get_float("radius").unwrap_or(0.0);
            let aspect_ratio = item.get_float("aspect_ratio").unwrap_or(0.0);

            // Check if it's circular
            let is_circular = circularity <= self.circularity_threshold
                && radius >= self.min_radius
                && radius <= self.max_radius
                && aspect_ratio >= 0.7
                && aspect_ratio <= 1.4;

            if is_circular {
                let mut new_item = item.clone();
                new_item.metadata.insert("is_circle".to_string(), MetadataValue::Bool(true));
                result.push(new_item);
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "Circle Filtering"
    }
}

/// Filter circles to keep only white ones
pub struct WhiteCircleFilterStep {
    pub brightness_threshold: f32,
}

impl PipelineStep for WhiteCircleFilterStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext) -> Result<Vec<PipelineData>> {
        let mut result = Vec::new();

        for item in data {
            // Reconstruct contour from metadata to calculate brightness
            let min_x = item.metadata.get("contour_min_x")
                .and_then(|v| if let MetadataValue::Int(i) = v { Some(*i as u32) } else { None })
                .ok_or_else(|| anyhow::anyhow!("Missing contour_min_x"))?;
            let min_y = item.metadata.get("contour_min_y")
                .and_then(|v| if let MetadataValue::Int(i) = v { Some(*i as u32) } else { None })
                .ok_or_else(|| anyhow::anyhow!("Missing contour_min_y"))?;
            let max_x = item.metadata.get("contour_max_x")
                .and_then(|v| if let MetadataValue::Int(i) = v { Some(*i as u32) } else { None })
                .ok_or_else(|| anyhow::anyhow!("Missing contour_max_x"))?;
            let max_y = item.metadata.get("contour_max_y")
                .and_then(|v| if let MetadataValue::Int(i) = v { Some(*i as u32) } else { None })
                .ok_or_else(|| anyhow::anyhow!("Missing contour_max_y"))?;
            let pixel_count = item.metadata.get("pixel_count")
                .and_then(|v| if let MetadataValue::Int(i) = v { Some(*i as u32) } else { None })
                .ok_or_else(|| anyhow::anyhow!("Missing pixel_count"))?;

            let contour = Contour {
                label: 0, // Not needed for brightness check
                min_x,
                min_y,
                max_x,
                max_y,
                pixel_count,
            };

            let brightness = contour.average_brightness(&item.original);

            if brightness >= self.brightness_threshold {
                let mut new_item = item.clone();
                new_item.metadata.insert("is_white".to_string(), MetadataValue::Bool(true));
                new_item.metadata.insert("brightness".to_string(), MetadataValue::Float(brightness));
                result.push(new_item);
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "White Circle Filtering"
    }
}

/// Remove background and crop to content (circular mask + brightness filter)
pub struct BackgroundRemovalStep;

impl PipelineStep for BackgroundRemovalStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext) -> Result<Vec<PipelineData>> {
        let mut result = Vec::new();

        for item in data {
            let gray = item.image.to_luma8();
            let (width, height) = gray.dimensions();

            // Circle is centered in the ROI (we added 10px padding in ContourDetectionStep)
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;

            // Estimate circle radius from bounding box
            // We added 10px padding, so subtract that to get the actual radius
            let padding = 10.0;
            let estimated_radius = ((width.min(height)) as f32 / 2.0) - padding;

            // Shrink less aggressively - only by 2px to avoid cutting off digits
            let inner_radius = estimated_radius - 2.0;

            // Create output image - start with all white
            let mut processed = image::GrayImage::from_pixel(width, height, image::Luma([255u8]));

            // Two-pass approach:
            // 1. Use circular mask to roughly isolate the interior
            // 2. Apply brightness filter to remove light pixels (outline/background)
            for (x, y, pixel) in gray.enumerate_pixels() {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();

                // Keep pixels that are:
                // 1. Inside the circle (with generous radius)
                // 2. AND sufficiently dark (not outline or background)
                if distance < inner_radius && pixel[0] < 150 {
                    processed.put_pixel(x, y, *pixel);
                }
            }

            // Find bounding box of non-white content (brightness < 250)
            let mut min_x = width;
            let mut min_y = height;
            let mut max_x = 0;
            let mut max_y = 0;
            let mut has_content = false;

            for (x, y, pixel) in processed.enumerate_pixels() {
                if pixel[0] < 250 {  // Non-white pixel
                    has_content = true;
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }

            // If no content found, skip this item
            if !has_content {
                continue;
            }

            // Crop to content with uniform border
            let border = 5u32;
            let crop_x = min_x.saturating_sub(border);
            let crop_y = min_y.saturating_sub(border);
            let crop_w = (max_x - min_x + 1 + 2 * border).min(width - crop_x);
            let crop_h = (max_y - min_y + 1 + 2 * border).min(height - crop_y);

            let cropped = image::imageops::crop_imm(&processed, crop_x, crop_y, crop_w, crop_h).to_image();

            let mut new_item = item.clone();
            new_item.image = image::DynamicImage::ImageLuma8(cropped);
            result.push(new_item);
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "Background Removal"
    }
}

/// Upscale images to target size while maintaining aspect ratio
pub struct UpscaleStep {
    pub target_size: u32,
}

impl PipelineStep for UpscaleStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext) -> Result<Vec<PipelineData>> {
        let mut result = Vec::new();

        for item in data {
            let gray = item.image.to_luma8();
            let (width, height) = gray.dimensions();

            // Calculate scaling to fit within target size while maintaining aspect ratio
            let scale = (self.target_size as f32 / width as f32).min(self.target_size as f32 / height as f32);
            let scaled_w = (width as f32 * scale) as u32;
            let scaled_h = (height as f32 * scale) as u32;

            // Resize with high-quality interpolation
            let scaled = image::imageops::resize(&gray, scaled_w, scaled_h, image::imageops::FilterType::CatmullRom);

            // Center the scaled image in a target_size x target_size white canvas
            let mut canvas = image::GrayImage::from_pixel(self.target_size, self.target_size, image::Luma([255u8]));
            let offset_x = (self.target_size - scaled_w) / 2;
            let offset_y = (self.target_size - scaled_h) / 2;

            image::imageops::overlay(&mut canvas, &scaled, offset_x.into(), offset_y.into());

            let mut new_item = item.clone();
            new_item.image = image::DynamicImage::ImageLuma8(canvas);
            result.push(new_item);
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "Upscale"
    }
}

/// Sharpen images to enhance text edges
pub struct SharpenStep {
    pub strength: f32,
}

impl PipelineStep for SharpenStep {
    fn process(&self, data: Vec<PipelineData>, _context: &PipelineContext) -> Result<Vec<PipelineData>> {
        let mut result = Vec::new();

        for item in data {
            let gray = item.image.to_luma8();
            let (width, height) = gray.dimensions();

            // Create sharpened output
            let mut sharpened = image::GrayImage::new(width, height);

            // Apply sharpening kernel
            // Kernel: center weight + (4 * strength), edges -strength
            // This enhances edges while preserving overall brightness
            for y in 1..height - 1 {
                for x in 1..width - 1 {
                    let center = gray.get_pixel(x, y)[0] as f32;
                    let top = gray.get_pixel(x, y - 1)[0] as f32;
                    let bottom = gray.get_pixel(x, y + 1)[0] as f32;
                    let left = gray.get_pixel(x - 1, y)[0] as f32;
                    let right = gray.get_pixel(x + 1, y)[0] as f32;

                    // Sharpening formula: center * (1 + 4*strength) - neighbors * strength
                    let sharpened_value = center * (1.0 + 4.0 * self.strength)
                        - (top + bottom + left + right) * self.strength;

                    // Clamp to valid range [0, 255]
                    let clamped = sharpened_value.max(0.0).min(255.0) as u8;
                    sharpened.put_pixel(x, y, image::Luma([clamped]));
                }
            }

            // Copy edges without sharpening
            for x in 0..width {
                sharpened.put_pixel(x, 0, *gray.get_pixel(x, 0));
                sharpened.put_pixel(x, height - 1, *gray.get_pixel(x, height - 1));
            }
            for y in 0..height {
                sharpened.put_pixel(0, y, *gray.get_pixel(0, y));
                sharpened.put_pixel(width - 1, y, *gray.get_pixel(width - 1, y));
            }

            let mut new_item = item.clone();
            new_item.image = image::DynamicImage::ImageLuma8(sharpened);
            result.push(new_item);
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "Sharpen"
    }
}

/// Run OCR on detected circles
pub struct OcrStep {
    // Lazy-initialized OCR engine, initialized once on first use
    // Using Arc so we can clone the reference and release the mutex lock
    engine: Mutex<Option<Arc<ocr::OcrEngine>>>,
}

impl OcrStep {
    pub fn new() -> Self {
        Self {
            engine: Mutex::new(None),
        }
    }
}

impl PipelineStep for OcrStep {
    fn process(&self, data: Vec<PipelineData>, context: &PipelineContext) -> Result<Vec<PipelineData>> {
        // Initialize OCR engine once on first call, reuse for all subsequent calls
        // Clone the Arc to release the mutex lock before processing
        let engine = {
            let mut engine_guard = self.engine.lock().unwrap();
            if engine_guard.is_none() {
                if context.verbose {
                    println!("Initializing OCR engine...");
                }
                *engine_guard = Some(Arc::new(ocr::init_ocr_engine()?));
                if context.verbose {
                    println!("OCR engine initialized successfully");
                }
            }
            engine_guard.as_ref().unwrap().clone()
        }; // Mutex lock is released here

        let mut result = Vec::new();
        let total = data.len();

        for (i, item) in data.into_iter().enumerate() {
            if context.verbose && total > 5 {
                println!("  Processing item {} of {}...", i + 1, total);
            }

            // Image is already preprocessed (background removed, upscaled)
            // Convert to RGB8 format for OCR
            let img = item.image.to_rgb8();

            // Prepare image for OCR
            if let Ok(img_source) = ocr::ImageSource::from_bytes(img.as_raw(), img.dimensions()) {
                if let Ok(ocr_input) = engine.prepare_input(img_source) {
                    // Run OCR
                    if let Ok(text) = engine.get_text(&ocr_input) {
                        let text = text.trim().to_string();
                        if !text.is_empty() {
                            let mut new_item = item.clone();
                            new_item.metadata.insert("ocr_text".to_string(), MetadataValue::String(text));
                            new_item.metadata.insert("ocr_confidence".to_string(), MetadataValue::Float(0.9));
                            result.push(new_item);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "OCR Recognition"
    }
}
