use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::application::services::autocomplete_service::AutocompleteState;
use crate::application::services::identity_resolver::IdentityResolver;

pub struct MentionPopup {
    resolver: IdentityResolver,
}

impl Default for MentionPopup {
    fn default() -> Self {
        Self::new(IdentityResolver::with_preference(false))
    }
}

impl MentionPopup {
    #[must_use]
    pub fn new(resolver: IdentityResolver) -> Self {
        Self { resolver }
    }
}

impl StatefulWidget for MentionPopup {
    type State = AutocompleteState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.active || state.results.is_empty() {
            return;
        }

        Widget::render(Clear, area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Mention User ")
            .title_style(Style::default().add_modifier(Modifier::BOLD));

        let items: Vec<ListItem> = state
            .results
            .iter()
            .map(|user| ListItem::new(Span::raw(self.resolver.resolve(user))))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_index));

        StatefulWidget::render(list, area, buf, &mut list_state);
    }
}
