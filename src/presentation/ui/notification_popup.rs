use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::domain::{Notification, NotificationLevel};
use crate::presentation::theme::Theme;

pub struct NotificationPopup<'a> {
    notification: &'a Notification,
    theme: &'a Theme,
}

impl<'a> NotificationPopup<'a> {
    #[must_use]
    pub fn new(notification: &'a Notification, theme: &'a Theme) -> Self {
        Self {
            notification,
            theme,
        }
    }
}

impl Widget for NotificationPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = format!(" {} ", self.notification.title);
        let message = &self.notification.message;

        let max_popup_width = 60.min(area.width.saturating_sub(2));
        let width = u16::try_from(message.width())
            .unwrap_or(u16::MAX)
            .max(u16::try_from(title.width()).unwrap_or(0))
            .saturating_add(4)
            .min(max_popup_width);

        let inner_width = width.saturating_sub(2).max(1);
        let content_width = u16::try_from(message.width()).unwrap_or(0);

        let lines = (content_width + inner_width - 1) / inner_width;

        let height = lines.saturating_add(3).min(10).max(3);

        let x = area.width.saturating_sub(width).saturating_sub(2);
        let y = 2;

        let popup_area = Rect::new(x, y, width, height);

        let intersection = area.intersection(popup_area);
        if intersection.area() == 0 {
            return;
        }

        let color = match self.notification.level {
            NotificationLevel::Info => self.theme.accent,
            NotificationLevel::Warn => Color::Yellow,
            NotificationLevel::Error => Color::Red,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(color));

        let para = Paragraph::new(message.as_str())
            .block(block)
            .wrap(Wrap { trim: true })
            .style(Style::default().add_modifier(Modifier::BOLD));

        Clear.render(intersection, buf);
        para.render(intersection, buf);
    }
}
