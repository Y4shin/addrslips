use clap::Parser;
use image::{DynamicImage, GrayImage, ImageReader, Luma, Rgb, RgbImage};
use imageproc::filter::gaussian_blur_f32;
use imageproc::edges::canny;
use imageproc::region_labelling::{connected_components, Connectivity};
use imageproc::drawing::{draw_hollow_rect_mut, draw_hollow_circle_mut};
use imageproc::rect::Rect;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "addrslips")]
#[command(about = "Detect and read house numbers from images")]
struct Cli {
    /// Path to input image file
    #[arg(value_name = "IMAGE")]
    image_path: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Save preprocessed debug images
    #[arg(long)]
    debug_preprocess: bool,

    /// Save edge-detected debug images
    #[arg(long)]
    debug_edges: bool,

    /// Show detected contours on the image
    #[arg(long)]
    show_contours: bool,

    /// Detect and show only circular contours
    #[arg(long)]
    detect_circles: bool,

    /// Output directory for debug images
    #[arg(long, value_name = "DIR", default_value = ".")]
    output_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct Contour {
    label: u32,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
    pixel_count: u32,
}

impl Contour {
    fn width(&self) -> u32 {
        self.max_x - self.min_x + 1
    }

    fn height(&self) -> u32 {
        self.max_y - self.min_y + 1
    }

    fn area(&self) -> u32 {
        self.pixel_count
    }

    fn perimeter(&self) -> f32 {
        // Approximate perimeter from bounding box
        2.0 * (self.width() as f32 + self.height() as f32)
    }

    fn circularity(&self) -> f32 {
        let perimeter = self.perimeter();
        // Use bounding box area instead of pixel count for better circularity estimate
        let area = (self.width() * self.height()) as f32;

        if area == 0.0 {
            return 0.0;
        }

        // Circularity = perimeter² / (4π × area)
        (perimeter * perimeter) / (4.0 * std::f32::consts::PI * area)
    }

    fn aspect_ratio(&self) -> f32 {
        let w = self.width() as f32;
        let h = self.height() as f32;
        if h == 0.0 {
            return 0.0;
        }
        w / h
    }

    fn is_circular(&self, threshold: f32) -> bool {
        let circ = self.circularity();
        circ >= 0.7 && circ <= threshold
    }

    fn radius(&self) -> f32 {
        // Approximate radius from bounding box
        let w = self.width() as f32;
        let h = self.height() as f32;
        (w + h) / 4.0
    }

    fn is_reasonable_size(&self, min_radius: f32, max_radius: f32) -> bool {
        let r = self.radius();
        r >= min_radius && r <= max_radius
    }

    /// Calculate average brightness of pixels in the circle region
    fn average_brightness(&self, img: &DynamicImage) -> f32 {
        let gray = img.to_luma8();
        let mut sum: u64 = 0;
        let mut count: u64 = 0;

        let center_x = (self.min_x + self.max_x) / 2;
        let center_y = (self.min_y + self.max_y) / 2;
        let radius = self.radius();

        // Sample pixels within the circle
        for y in self.min_y..=self.max_y {
            for x in self.min_x..=self.max_x {
                // Check if pixel is within circle
                let dx = x as f32 - center_x as f32;
                let dy = y as f32 - center_y as f32;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= radius {
                    if x < gray.width() && y < gray.height() {
                        sum += gray.get_pixel(x, y)[0] as u64;
                        count += 1;
                    }
                }
            }
        }

        if count > 0 {
            sum as f32 / count as f32
        } else {
            0.0
        }
    }

    fn is_white(&self, img: &DynamicImage, threshold: f32) -> bool {
        self.average_brightness(img) >= threshold
    }
}

/// Convert image to grayscale
fn to_grayscale(img: &DynamicImage) -> GrayImage {
    img.to_luma8()
}

/// Apply Gaussian blur to reduce noise
fn apply_blur(img: &GrayImage, sigma: f32) -> GrayImage {
    gaussian_blur_f32(img, sigma)
}

/// Detect edges using Canny edge detector
fn detect_edges(img: &GrayImage, low_threshold: f32, high_threshold: f32) -> GrayImage {
    canny(img, low_threshold, high_threshold)
}

/// Find contours in binary edge image using connected components
fn find_contours(edges: &GrayImage, min_area: u32) -> Vec<Contour> {
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

/// Draw contours on an RGB image
fn draw_contours(img: &DynamicImage, contours: &[Contour]) -> RgbImage {
    let mut output = img.to_rgb8();
    let color = Rgb([255u8, 0u8, 0u8]); // Red

    for contour in contours {
        let rect = Rect::at(contour.min_x as i32, contour.min_y as i32)
            .of_size(contour.width(), contour.height());
        draw_hollow_rect_mut(&mut output, rect, color);
    }

    output
}

/// Filter contours to find circular shapes
fn filter_circles(
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
fn filter_white_circles(
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

/// Draw detected circles on an RGB image
fn draw_circles(img: &DynamicImage, circles: &[Contour]) -> RgbImage {
    let mut output = img.to_rgb8();
    let color = Rgb([0u8, 255u8, 0u8]); // Green for circles

    for circle in circles {
        let center_x = (circle.min_x + circle.max_x) / 2;
        let center_y = (circle.min_y + circle.max_y) / 2;
        let radius = circle.radius() as i32;

        draw_hollow_circle_mut(
            &mut output,
            (center_x as i32, center_y as i32),
            radius,
            color
        );
    }

    output
}

/// Save debug image to specified path
fn save_debug_image(img: &GrayImage, output_dir: &Path, filename: &str) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join(filename);
    img.save(&output_path)?;
    println!("Saved debug image: {}", output_path.display());
    Ok(())
}

/// Save RGB debug image
fn save_rgb_image(img: &RgbImage, output_dir: &Path, filename: &str) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join(filename);
    img.save(&output_path)?;
    println!("Saved debug image: {}", output_path.display());
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.verbose {
        println!("Loading image: {:?}", args.image_path);
    }

    // Load image
    let img = ImageReader::open(&args.image_path)?
        .decode()?;

    // Print image information
    println!("Successfully loaded image:");
    println!("  Dimensions: {}x{}", img.width(), img.height());
    println!("  Color type: {:?}", img.color());

    // Get base filename for output files
    let base_name = args.image_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    // Preprocessing pipeline
    if args.debug_preprocess || args.debug_edges || args.show_contours || args.detect_circles {
        if args.verbose {
            println!("\nPreprocessing image...");
        }

        // Convert to grayscale
        if args.verbose {
            println!("Converting to grayscale...");
        }
        let gray = to_grayscale(&img);

        if args.debug_preprocess {
            let gray_filename = format!("{}_grayscale.jpg", base_name);
            save_debug_image(&gray, &args.output_dir, &gray_filename)?;
        }

        // Apply Gaussian blur
        if args.verbose {
            println!("Applying Gaussian blur...");
        }
        let blurred = apply_blur(&gray, 1.5);

        if args.debug_preprocess {
            let blur_filename = format!("{}_blurred.jpg", base_name);
            save_debug_image(&blurred, &args.output_dir, &blur_filename)?;
        }

        if args.debug_preprocess {
            println!("\nPreprocessing complete!");
        }

        // Edge detection
        if args.debug_edges || args.show_contours || args.detect_circles {
            if args.verbose {
                println!("\nDetecting edges...");
            }

            // Canny edge detection with thresholds
            let edges = detect_edges(&blurred, 50.0, 100.0);

            if args.debug_edges {
                let edges_filename = format!("{}_edges.jpg", base_name);
                save_debug_image(&edges, &args.output_dir, &edges_filename)?;
            }

            if args.debug_edges && !args.show_contours && !args.detect_circles {
                println!("\nEdge detection complete!");
            }

            // Contour detection
            if args.show_contours || args.detect_circles {
                if args.verbose {
                    println!("\nFinding contours...");
                }

                // Find contours with minimum area filter to reduce noise
                let contours = find_contours(&edges, 10);

                if args.verbose {
                    println!("Found {} contours", contours.len());
                }

                // Draw contours on original image
                if args.show_contours {
                    if args.verbose {
                        println!("Drawing contours...");
                    }
                    let annotated = draw_contours(&img, &contours);
                    let contours_filename = format!("{}_contours.jpg", base_name);
                    save_rgb_image(&annotated, &args.output_dir, &contours_filename)?;

                    println!("\nContour detection complete! Found {} contours.", contours.len());
                }

                // Circle detection
                if args.detect_circles {
                    if args.verbose {
                        println!("\nFiltering for circular shapes...");
                        println!("Analyzing contours (showing first 10):");
                        for (i, contour) in contours.iter().take(10).enumerate() {
                            println!("  Contour {}: radius={:.1}, circ={:.3}, aspect={:.2}, pixels={}",
                                    i + 1, contour.radius(), contour.circularity(),
                                    contour.aspect_ratio(), contour.area());
                        }
                    }

                    // Filter for circles with reasonable size and circularity
                    let circles = filter_circles(&contours, 10.0, 200.0, 2.0);

                    if args.verbose {
                        println!("Found {} circular shapes (from {} total contours)",
                                circles.len(), contours.len());
                    }

                    // Filter for white circles only
                    if args.verbose {
                        println!("\nFiltering for white circles...");
                        // Show brightness values for first few circles
                        println!("Analyzing brightness (showing first 5):");
                        for (i, circle) in circles.iter().take(5).enumerate() {
                            let brightness = circle.average_brightness(&img);
                            println!("  Circle {}: brightness={:.1}/255", i + 1, brightness);
                        }
                    }

                    let white_circles = filter_white_circles(&circles, &img, 200.0);

                    if args.verbose {
                        println!("Found {} white circles (from {} circular shapes)",
                                white_circles.len(), circles.len());

                        // Print some example details in verbose mode
                        if !white_circles.is_empty() {
                            println!("Example white circles:");
                            for (i, circle) in white_circles.iter().take(5).enumerate() {
                                println!("  Circle {}: radius={:.1}, brightness={:.1}",
                                        i + 1, circle.radius(), circle.average_brightness(&img));
                            }
                        }
                    }

                    // Draw white circles on original image
                    if args.verbose {
                        println!("Drawing detected white circles...");
                    }
                    let annotated = draw_circles(&img, &white_circles);
                    let circles_filename = format!("{}_circles.jpg", base_name);
                    save_rgb_image(&annotated, &args.output_dir, &circles_filename)?;

                    println!("\nCircle detection complete! Found {} white circles from {} circular shapes (out of {} total contours).",
                            white_circles.len(), circles.len(), contours.len());
                }
            }
        }
    }

    Ok(())
}
