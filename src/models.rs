use image::DynamicImage;

#[derive(Debug, Clone)]
pub struct Contour {
    pub label: u32,
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
    pub pixel_count: u32,
}

impl Contour {
    pub fn width(&self) -> u32 {
        self.max_x - self.min_x + 1
    }

    pub fn height(&self) -> u32 {
        self.max_y - self.min_y + 1
    }

    pub fn area(&self) -> u32 {
        self.pixel_count
    }

    pub fn perimeter(&self) -> f32 {
        // Approximate perimeter from bounding box
        2.0 * (self.width() as f32 + self.height() as f32)
    }

    pub fn circularity(&self) -> f32 {
        let perimeter = self.perimeter();
        // Use bounding box area instead of pixel count for better circularity estimate
        let area = (self.width() * self.height()) as f32;

        if area == 0.0 {
            return 0.0;
        }

        // Circularity = perimeter² / (4π × area)
        (perimeter * perimeter) / (4.0 * std::f32::consts::PI * area)
    }

    pub fn aspect_ratio(&self) -> f32 {
        let w = self.width() as f32;
        let h = self.height() as f32;
        if h == 0.0 {
            return 0.0;
        }
        w / h
    }

    pub fn is_circular(&self, threshold: f32) -> bool {
        let circ = self.circularity();
        circ >= 0.7 && circ <= threshold
    }

    pub fn radius(&self) -> f32 {
        // Approximate radius from bounding box
        let w = self.width() as f32;
        let h = self.height() as f32;
        (w + h) / 4.0
    }

    pub fn is_reasonable_size(&self, min_radius: f32, max_radius: f32) -> bool {
        let r = self.radius();
        r >= min_radius && r <= max_radius
    }

    /// Calculate average brightness of pixels in the circle region
    pub fn average_brightness(&self, img: &DynamicImage) -> f32 {
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

    pub fn is_white(&self, img: &DynamicImage, threshold: f32) -> bool {
        self.average_brightness(img) >= threshold
    }

    /// Extract the circle region as a sub-image for OCR
    pub fn extract_roi(&self, img: &DynamicImage) -> Option<DynamicImage> {
        // Add padding around the bounding box for better OCR
        let padding = 5;
        let x = self.min_x.saturating_sub(padding);
        let y = self.min_y.saturating_sub(padding);
        let width = (self.width() + 2 * padding).min(img.width() - x);
        let height = (self.height() + 2 * padding).min(img.height() - y);

        // Ensure valid dimensions
        if width == 0 || height == 0 {
            return None;
        }

        Some(img.crop_imm(x, y, width, height))
    }

    /// Get center coordinates
    pub fn center(&self) -> (u32, u32) {
        ((self.min_x + self.max_x) / 2, (self.min_y + self.max_y) / 2)
    }
}

#[derive(Debug, Clone)]
pub struct HouseNumberDetection {
    pub number: String,
    pub x: u32,
    pub y: u32,
    pub confidence: f32,
}
