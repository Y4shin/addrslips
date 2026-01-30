use image::{DynamicImage, GrayImage};
use imageproc::filter::gaussian_blur_f32;
use imageproc::edges::canny;

/// Convert image to grayscale
pub fn to_grayscale(img: &DynamicImage) -> GrayImage {
    img.to_luma8()
}

/// Apply Gaussian blur to reduce noise
pub fn apply_blur(img: &GrayImage, sigma: f32) -> GrayImage {
    gaussian_blur_f32(img, sigma)
}

/// Detect edges using Canny edge detector
pub fn detect_edges(img: &GrayImage, low_threshold: f32, high_threshold: f32) -> GrayImage {
    canny(img, low_threshold, high_threshold)
}
