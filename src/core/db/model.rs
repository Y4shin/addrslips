#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl TryFrom<i64> for Color {
    type Error = anyhow::Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let r = ((value >> 16) & 0xFF) as u8;
        let g = ((value >> 8) & 0xFF) as u8;
        let b = (value & 0xFF) as u8;
        Ok(Color { r, g, b })
    }
}


impl From<Color> for i64 {
    fn from(color: Color) -> Self {
        ((color.r as i64) << 16) | ((color.g as i64) << 8) | (color.b as i64)
    }
}