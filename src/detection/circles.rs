use image::DynamicImage;
use crate::models::Contour;

/// Filter contours to find circular shapes
pub fn filter_circles(
    contours: &[Contour],
    min_radius: f32,
    max_radius: f32,
    circularity_threshold: f32,
) -> Vec<Contour> {
    contours
        .iter()
        .filter(|c| {
            let aspect = c.aspect_ratio();
            c.is_circular(circularity_threshold) &&
            c.is_reasonable_size(min_radius, max_radius) &&
            aspect >= 0.7 && aspect <= 1.4  // Roughly square bounding box
        })
        .cloned()
        .collect()
}

/// Filter circles to keep only white ones
pub fn filter_white_circles(
    circles: &[Contour],
    img: &DynamicImage,
    brightness_threshold: f32,
) -> Vec<Contour> {
    circles
        .iter()
        .filter(|c| c.is_white(img, brightness_threshold))
        .cloned()
        .collect()
}
