use addrslips::core::db::{Color, NewAddress, NewArea, Point, ProjectDb};
use image::{ImageBuffer, Rgb};
use tempfile::NamedTempFile;

/// Creates a 100x100 red test image and returns the temp file.
/// The file will be automatically cleaned up when dropped.
pub fn create_test_image() -> NamedTempFile {
    let img = ImageBuffer::from_fn(100, 100, |_, _| Rgb([255u8, 0u8, 0u8]));
    let file = tempfile::Builder::new()
        .suffix(".png")
        .tempfile()
        .expect("Failed to create temp image file");
    img.save_with_format(file.path(), image::ImageFormat::Png)
        .expect("Failed to save test image");
    file
}

/// Creates a ProjectDb with a temporary tar.zst file.
/// Returns both the project and the temp directory (which must be kept alive).
pub async fn create_test_project() -> (ProjectDb, tempfile::TempDir) {
    let dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let path = dir.path().join("test.addrslips");
    let project = ProjectDb::new(&path)
        .await
        .expect("Failed to create test project");
    (project, dir)
}

/// Creates a NewArea with the given name and color, using a test image.
/// Returns the NewArea and the temp image file (keep alive until area is created).
pub fn make_new_area(name: &str, color: Color) -> (NewArea, NamedTempFile) {
    let img_file = create_test_image();
    let new_area = NewArea {
        name: name.to_string(),
        color,
        image_path: img_file.path().to_path_buf(),
    };
    (new_area, img_file)
}

/// Color constants for tests
pub const TEST_RED: Color = Color { r: 255, g: 0, b: 0 };
pub const TEST_BLUE: Color = Color { r: 0, g: 0, b: 255 };
pub const TEST_GREEN: Color = Color { r: 0, g: 255, b: 0 };

/// Creates a NewAddress with test data
pub fn make_test_address(house_number: &str, x: u32, y: u32) -> NewAddress {
    NewAddress {
        house_number: house_number.to_string(),
        position: Point { x, y },
        confidence: 0.95,
        estimated_flats: Some(4),
        circle_radius: 10,
        assigned_street_id: None,
    }
}
