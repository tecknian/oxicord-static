use super::adapter::ColorConverter;
use ratatui::style::{Color, Style};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub accent: Color,
    pub mention_style: Style,
    pub selection_style: Style,
    pub dimmed_style: Style,
    pub base_style: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::new("Yellow", None)
    }
}

impl Theme {
    pub fn new(accent_color_str: &str, mention_color_str: Option<&str>) -> Self {
        let accent = parse_color(accent_color_str);
        let mention = mention_color_str.map(parse_color);
        Self::from_color(accent, mention)
    }

    #[must_use]
    pub fn from_color(accent: Color, mention_color: Option<Color>) -> Self {
        let accent_hsl = ColorConverter::to_hsl(accent);

        let mention_base = mention_color.unwrap_or(Color::Blue);
        let mut mention_bg_hsl = ColorConverter::to_hsl(mention_base);
        mention_bg_hsl.l = 0.1;
        mention_bg_hsl.s = 0.5;
        let mention_bg = ColorConverter::to_ratatui(mention_bg_hsl);

        let mention_style = Style::default().bg(mention_bg).fg(Color::White);

        let mut selection_bg_hsl = accent_hsl;
        selection_bg_hsl.l = 0.2;
        selection_bg_hsl.s = 0.3;
        let selection_bg = ColorConverter::to_ratatui(selection_bg_hsl);

        let selection_style = Style::default().bg(selection_bg).fg(Color::White);

        let dimmed_style = Style::default().fg(Color::DarkGray);

        Self {
            accent,
            mention_style,
            selection_style,
            dimmed_style,
            base_style: Style::default().fg(Color::Reset),
        }
    }
}

fn parse_color(s: &str) -> Color {
    if let Ok(c) = Color::from_str(s) {
        return c;
    }

    if s.starts_with('#')
        && let Ok((r, g, b)) = parse_hex_color(s)
    {
        return Color::Rgb(r, g, b);
    }

    match s.to_lowercase().as_str() {
        "orange" => Color::Indexed(208),
        _ => Color::Yellow,
    }
}

fn parse_hex_color(s: &str) -> Result<(u8, u8, u8), ()> {
    let s = s.trim_start_matches('#');

    if !s.is_ascii() {
        return Err(());
    }

    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| ())?;
        let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| ())?;
        let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| ())?;
        Ok((r, g, b))
    } else if s.len() == 3 {
        let r = u8::from_str_radix(&format!("{}{}", &s[0..1], &s[0..1]), 16).map_err(|_| ())?;
        let g = u8::from_str_radix(&format!("{}{}", &s[1..2], &s[1..2]), 16).map_err(|_| ())?;
        let b = u8::from_str_radix(&format!("{}{}", &s[2..3], &s[2..3]), 16).map_err(|_| ())?;
        Ok((r, g, b))
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("Red"), Color::Red);
        assert_eq!(parse_color("blue"), Color::Blue);
        assert_eq!(parse_color("#FF0000"), Color::Rgb(255, 0, 0));
        assert_eq!(parse_color("#0f0"), Color::Rgb(0, 255, 0));
        assert_eq!(parse_color("Orange"), Color::Indexed(208));
        assert_eq!(parse_color("Invalid"), Color::Yellow);
    }
}
