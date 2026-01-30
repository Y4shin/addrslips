use image::{DynamicImage, GrayImage, Luma};
pub use ocrs::{OcrEngine, ImageSource};  // Re-export for use in other modules
use ocrs::OcrEngineParams;
use rten::Model;
use std::path::Path;

/// Initialize OCR engine with models from standard cache location
pub fn init_ocr_engine() -> anyhow::Result<OcrEngine> {
    // Try to load models from standard locations
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))?;

    let cache_dir = Path::new(&home_dir).join(".cache/ocrs");
    let detection_model_path = cache_dir.join("text-detection.rten");
    let recognition_model_path = cache_dir.join("text-recognition.rten");

    // Check if models exist
    if !detection_model_path.exists() || !recognition_model_path.exists() {
        anyhow::bail!(
            "OCR models not found. Please run: ocrs-cli --help (or download models manually)\n\
             Expected locations:\n  - {}\n  - {}",
            detection_model_path.display(),
            recognition_model_path.display()
        );
    }

    // Load models
    let detection_model = Model::load_file(&detection_model_path)?;
    let recognition_model = Model::load_file(&recognition_model_path)?;

    // Create engine
    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;

    Ok(engine)
}

/// Preprocess ROI to isolate black text on white background
/// Strategy: Remove background, crop to content, add uniform border, upscale to 100x100px
pub fn preprocess_roi_for_ocr(roi: &DynamicImage) -> DynamicImage {
    let gray = roi.to_luma8();
    let (width, height) = gray.dimensions();

    // Circle is centered in the ROI (we added 5px padding when extracting)
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    // Estimate circle radius: ROI size minus padding, divided by 2
    // The bounding box is roughly 2*radius + 10 (5px padding each side)
    let estimated_radius = ((width.min(height)) as f32 / 2.0) - 5.0;

    // The outline is about 2-3 pixels thick, shrink to exclude it
    let inner_radius = estimated_radius - 3.5;

    // Create output image - start with all white
    let mut processed = GrayImage::from_pixel(width, height, Luma([255u8]));

    // For each pixel in the input, keep pixels inside the circle
    for (x, y, pixel) in gray.enumerate_pixels() {
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Keep ALL pixels that are well inside the circle (excludes outline)
        if distance < inner_radius {
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

    // If no content found, return the processed image as-is
    if !has_content {
        return DynamicImage::ImageLuma8(processed);
    }

    // Crop to content with uniform border
    let border = 5u32;
    let crop_x = min_x.saturating_sub(border);
    let crop_y = min_y.saturating_sub(border);
    let crop_w = (max_x - min_x + 1 + 2 * border).min(width - crop_x);
    let crop_h = (max_y - min_y + 1 + 2 * border).min(height - crop_y);

    let cropped = image::imageops::crop_imm(&processed, crop_x, crop_y, crop_w, crop_h).to_image();

    // Upscale to 100x100px while maintaining aspect ratio
    let target_size = 100u32;
    let (cropped_w, cropped_h) = cropped.dimensions();

    // Calculate scaling to fit within 100x100 while maintaining aspect ratio
    let scale = (target_size as f32 / cropped_w as f32).min(target_size as f32 / cropped_h as f32);
    let scaled_w = (cropped_w as f32 * scale) as u32;
    let scaled_h = (cropped_h as f32 * scale) as u32;

    // Resize with high-quality interpolation
    let scaled = image::imageops::resize(&cropped, scaled_w, scaled_h, image::imageops::FilterType::CatmullRom);

    // Center the scaled image in a 100x100 white canvas
    let mut canvas = GrayImage::from_pixel(target_size, target_size, Luma([255u8]));
    let offset_x = (target_size - scaled_w) / 2;
    let offset_y = (target_size - scaled_h) / 2;

    image::imageops::overlay(&mut canvas, &scaled, offset_x.into(), offset_y.into());

    DynamicImage::ImageLuma8(canvas)
}

/// Recognize house number from a circle ROI
pub fn recognize_house_number(
    engine: &OcrEngine,
    roi: &DynamicImage,
) -> Option<(String, f32)> {
    // Preprocess: remove background and circle outline, leaving only black text on white
    let preprocessed = preprocess_roi_for_ocr(roi);

    // Convert to RGB8 format for OCR
    let img = preprocessed.to_rgb8();

    // Prepare image for OCR
    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions()).ok()?;
    let ocr_input = engine.prepare_input(img_source).ok()?;

    // Run OCR - use simple get_text for straightforward extraction
    match engine.get_text(&ocr_input) {
        Ok(text) => {
            let text = text.trim().to_string();
            if text.is_empty() {
                None
            } else {
                // For now, we'll use a default confidence since get_text doesn't provide it
                // In a future phase, we can use the detailed API for per-character confidence
                Some((text, 0.9))
            }
        }
        Err(_) => None,
    }
}
