use image::{RgbImage, Rgb};

fn main() {
    let mut img = RgbImage::new(800, 600);

    // Fill with a gradient
    for y in 0..600 {
        for x in 0..800 {
            let r = (x * 255 / 800) as u8;
            let g = (y * 255 / 600) as u8;
            let b = 128;
            img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    img.save("test_image.jpg").unwrap();
    println!("Created test_image.jpg (800x600 RGB gradient)");
}
