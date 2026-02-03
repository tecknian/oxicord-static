use coolor::{Hsl, Rgb};
use ratatui::style::Color;

pub struct ColorConverter;

impl ColorConverter {
    #[must_use]
    pub fn to_hsl(color: Color) -> Hsl {
        let (r, g, b) = match color {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Black => (0, 0, 0),
            Color::Red => (170, 0, 0),
            Color::Green => (0, 170, 0),
            Color::Yellow => (170, 85, 0),
            Color::Blue => (0, 0, 170),
            Color::Magenta => (170, 0, 170),
            Color::Cyan => (0, 170, 170),
            Color::Gray => (170, 170, 170),
            Color::DarkGray => (85, 85, 85),
            Color::LightRed => (255, 85, 85),
            Color::LightGreen => (85, 255, 85),
            Color::LightYellow => (255, 255, 85),
            Color::LightBlue => (85, 85, 255),
            Color::LightMagenta => (255, 85, 255),
            Color::LightCyan => (85, 255, 255),
            Color::Indexed(i) => ansi_to_rgb(i),
            _ => (255, 255, 255),
        };

        Rgb::new(r, g, b).to_hsl()
    }

    #[must_use]
    pub fn to_ratatui(hsl: Hsl) -> Color {
        let rgb: Rgb = hsl.to_rgb();
        Color::Rgb(rgb.r, rgb.g, rgb.b)
    }
}

fn ansi_to_rgb(i: u8) -> (u8, u8, u8) {
    match i {
        0 => (0, 0, 0),
        1 => (170, 0, 0),
        2 => (0, 170, 0),
        3 => (170, 85, 0),
        4 => (0, 0, 170),
        5 => (170, 0, 170),
        6 => (0, 170, 170),
        7 => (170, 170, 170),
        8 => (85, 85, 85),
        9 => (255, 85, 85),
        10 => (85, 255, 85),
        11 => (255, 255, 85),
        12 => (85, 85, 255),
        13 => (255, 85, 255),
        14 => (85, 255, 255),
        15 => (255, 255, 255),

        i if (16..=231).contains(&i) => {
            let i = i - 16;
            let r = (i / 36) % 6;
            let g = (i / 6) % 6;
            let b = i % 6;

            let map = |c| if c == 0 { 0 } else { c * 40 + 55 };
            (map(r), map(g), map(b))
        }

        i if (232..=255).contains(&i) => {
            let v = (i - 232) * 10 + 8;
            (v, v, v)
        }

        _ => (255, 255, 255),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_round_trip() {
        let original = Color::Rgb(100, 150, 200);
        let hsl = ColorConverter::to_hsl(original);
        let back = ColorConverter::to_ratatui(hsl);

        if let Color::Rgb(r, g, b) = back {
            assert!((i16::from(r) - 100).abs() <= 1);
            assert!((i16::from(g) - 150).abs() <= 1);
            assert!((i16::from(b) - 200).abs() <= 1);
        } else {
            panic!("Expected RGB color");
        }
    }

    #[test]
    fn test_ansi_conversion() {
        let red = Color::Red;
        let hsl = ColorConverter::to_hsl(red);
        let rgb: Rgb = hsl.to_rgb();
        assert!((i16::from(rgb.r) - 170).abs() <= 1);
        assert!((i16::from(rgb.g)).abs() <= 1);
        assert!((i16::from(rgb.b)).abs() <= 1);
    }

    #[test]
    fn test_ansi_256_cube() {
        let (r, g, b) = ansi_to_rgb(208);
        assert_eq!((r, g, b), (255, 135, 0));

        let (r, g, b) = ansi_to_rgb(16);
        assert_eq!((r, g, b), (0, 0, 0));

        let (r, g, b) = ansi_to_rgb(231);
        assert_eq!((r, g, b), (255, 255, 255));
    }
}
