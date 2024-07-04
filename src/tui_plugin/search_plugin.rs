use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{block::Title, Block, List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{
    bodies::{body_data::BodyData, body_id::BodyID},
    core_plugin::BodyInfo,
    utils::{
        list::{select_next_clamp, select_previous_clamp},
        ui::Direction2,
    },
};

#[derive(Debug)]
pub enum SearchViewEvent {
    MoveCursor(Direction2),
    SelectSearch(Direction2),
    ValidateSearch,
    WriteChar(char),
    DeleteChar,
}

pub struct SearchState {
    search_entries: Vec<SearchEntry>,
    search_character_index: usize,
    list_state: ListState,
    search_input: String,
    search_matcher: SkimMatcherV2,
}

struct SearchEntry {
    id: BodyID,
    name: String,
}

impl From<&BodyData> for SearchEntry {
    fn from(value: &BodyData) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
        }
    }
}

pub struct SearchWidget;

impl StatefulWidget for SearchWidget {
    type State = SearchState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let names: Vec<_> = state
            .search_entries
            .iter()
            .map(|entry| entry.name.clone())
            .collect();
        let texts: Vec<Text> = names
            .into_iter()
            .map(|s| Text::styled(s, Style::default()))
            .collect();
        let search_bar = Paragraph::new(&state.search_input[..]).block(Block::bordered());
        let list = List::new(texts)
            .block(
                Block::bordered()
                    .title(Title::from("Search view".bold()).alignment(Alignment::Center)),
            )
            .highlight_symbol("> ");
        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(area);
        search_bar.render(chunks[0], buf);

        <List as StatefulWidget>::render(list, chunks[1], buf, &mut state.list_state);
    }
}

// Code from https://ratatui.rs/examples/apps/user_input/
impl SearchState {
    pub fn new<'a>(bodies: impl Iterator<Item = &'a BodyData>) -> SearchState {
        let search_entries: Vec<_> = bodies.map(|data| data.into()).collect();
        SearchState {
            search_entries,
            search_character_index: 0,
            search_input: String::new(),
            search_matcher: SkimMatcherV2::default(),
            list_state: ListState::default(),
        }
    }
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.search_character_index.saturating_sub(1);
        self.search_character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.search_character_index.saturating_add(1);
        self.search_character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.search_input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&mut self) -> usize {
        self.search_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.search_character_index)
            .unwrap_or(self.search_input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.search_character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.search_character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.search_input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.search_input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.search_input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.search_input.chars().count())
    }

    pub fn selected_body_id(&self) -> Option<BodyID> {
        self.list_state
            .selected()
            .and_then(|i| self.search_entries.get(i))
            .map(|entry| entry.id)
    }

    pub fn select_next_search(&mut self) {
        select_next_clamp(&mut self.list_state, self.search_entries.len() - 1);
    }

    pub fn select_previous_search(&mut self) {
        select_previous_clamp(&mut self.list_state, 0);
    }

    pub fn reset_search(&mut self) {
        self.search_character_index = 0;
        self.search_input = String::new();
        self.list_state.select(Some(0));
    }

    pub fn update_search_entries<'a>(&mut self, bodies: impl Iterator<Item = &'a BodyInfo>) {
        let mut ids_score: Vec<_> = bodies
            .filter_map(|BodyInfo(body)| {
                self.search_matcher
                    .fuzzy_match(&body.name, &self.search_input)
                    .map(|score| (body, score))
            })
            .collect();
        ids_score.sort_by(|a, b| a.0.name.cmp(&b.0.name));
        ids_score.sort_by(|a, b| a.1.cmp(&b.1).reverse());
        self.search_entries = ids_score.into_iter().map(|(data, _)| data.into()).collect();
        if self.list_state.selected().is_none() && !self.search_entries.is_empty() {
            self.list_state.select(Some(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{
        bodies::body_data::BodyType,
        core_plugin::{BodiesConfig, BodyInfo},
        standalone_plugin::StandalonePlugin,
        tui_plugin::{
            explorer_screen::ExplorerEvent, search_plugin::SearchViewEvent, AppScreen, TuiPlugin,
        },
    };

    #[test]
    fn test_search() {
        let mut app = App::new();
        app.add_plugins((
            StandalonePlugin(BodiesConfig::SmallestBodyType(BodyType::Moon)),
            TuiPlugin::testing(),
        ));
        app.update();
        app.update();
        let bodies: Vec<_> = app
            .world
            .query::<&BodyInfo>()
            .iter(&app.world)
            .cloned()
            .collect();
        if let AppScreen::Explorer(ctx) = app.world.resource_mut::<AppScreen>().as_mut() {
            ctx.search.search_input = "Moo".into();
            ctx.search.update_search_entries(bodies.iter());
        }
        app.update();

        app.world
            .send_event(ExplorerEvent::Search(SearchViewEvent::ValidateSearch));
        app.update();
        if let AppScreen::Explorer(ctx) = app.world.resource_mut::<AppScreen>().as_mut() {
            let id = ctx.tree.selected_body_id();
            assert_eq!(id, "lune".into())
        }
    }
}
