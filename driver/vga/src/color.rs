use crate::PixelType;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Color {
    Black,
    White,
    Red,
    Green,
    Blue,
    Other(PixelType),
}

impl graphics::Color for Color {}

impl From<Color> for PixelType {
    fn from(value: Color) -> Self {
        match value {
            Color::Black => 0x00_00_00_00,
            Color::White => 0x00_ff_ff_ff,
            Color::Red => 0x00_ff_00_00,
            Color::Green => 0x00_00_ff_00,
            Color::Blue => 0x00_00_00_ff,
            Color::Other(pt) => pt,
        }
    }
}
