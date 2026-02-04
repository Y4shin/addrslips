#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Color {
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };

    pub fn from_hex_string(hex: &str) -> Result<Self, anyhow::Error> {
        let hex = hex.trim_start_matches('#');
        if (hex.len() != 6 && hex.len() != 3) || !hex.chars().all(|c| c.is_digit(16)) {
            return Err(anyhow::anyhow!("Invalid hex color string"));
        }
        if hex.len() == 3 {
            let r = &hex[0..1];
            let g = &hex[1..2];
            let b = &hex[2..3];
            let hex_expanded = format!("{}{}{}{}{}{}", r, r, g, g, b, b);
            Color::from_hex_string(&hex_expanded)
        } else {
            let r = &hex[0..2];
            let g = &hex[2..4];
            let b = &hex[4..6];
            let r = u8::from_str_radix(r, 16)?;
            let g = u8::from_str_radix(g, 16)?;
            let b = u8::from_str_radix(b, 16)?;
            Ok(Color { r, g, b })
        }
    }
    pub fn to_hex_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}