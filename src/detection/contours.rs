use image::{GrayImage, Luma};
use imageproc::region_labelling::{connected_components, Connectivity};
use std::collections::HashMap;
use crate::models::Contour;

/// Find contours in binary edge image using connected components
pub fn find_contours(edges: &GrayImage, min_area: u32) -> Vec<Contour> {
    // Label connected components (white pixels = edges)
    let labeled = connected_components(edges, Connectivity::Eight, Luma([0]));

    // Build contours from labeled regions
    let mut regions: HashMap<u32, (u32, u32, u32, u32, u32)> = HashMap::new();

    for (x, y, label) in labeled.enumerate_pixels() {
        let label_val = label[0] as u32;
        if label_val == 0 {
            continue; // Skip background
        }

        regions.entry(label_val)
            .and_modify(|(min_x, min_y, max_x, max_y, count)| {
                *min_x = (*min_x).min(x);
                *min_y = (*min_y).min(y);
                *max_x = (*max_x).max(x);
                *max_y = (*max_y).max(y);
                *count += 1;
            })
            .or_insert((x, y, x, y, 1));
    }

    // Convert to Contour structs and filter by minimum area
    regions.into_iter()
        .map(|(label, (min_x, min_y, max_x, max_y, count))| {
            Contour {
                label,
                min_x,
                min_y,
                max_x,
                max_y,
                pixel_count: count,
            }
        })
        .filter(|c| c.pixel_count >= min_area)
        .collect()
}
