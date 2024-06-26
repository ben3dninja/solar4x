use bevy::prelude::*;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{block::Title, Block, List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::{body_data::BodyData, body_id::BodyID},
    core_plugin::BodyInfo,
    utils::{
        list::{select_next_clamp, select_previous_clamp},
        ui::Direction2,
    },
};

use super::{tree_plugin::TreeState, FocusView, WindowEvent};

pub struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SearchViewEvent>()
            .add_systems(PostStartup, initialize_search)
            .add_systems(
                Update,
                (
                    (
                        handle_search_events,
                        reset_on_enter_search.run_if(resource_exists::<FocusView>),
                    )
                        .chain(),
                    update_search_entries,
                )
                    .chain(),
            );
    }
}

#[derive(Debug, Event)]
pub enum SearchViewEvent {
    MoveCursor(Direction2),
    SelectSearch(Direction2),
    ValidateSearch,
    WriteChar(char),
    DeleteChar,
}

#[derive(Resource)]
pub struct SearchState {
    search_entries: Vec<SearchEntry>,
    // search_state: ListState,
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

// TODO : only on key event q
fn update_search_entries(mut state: ResMut<SearchState>, query: Query<&BodyInfo>) {
    let mut ids_score: Vec<_> = query
        .iter()
        .filter_map(|BodyInfo(body)| {
            state
                .search_matcher
                .fuzzy_match(&body.name, &state.search_input)
                .map(|score| (body, score))
        })
        .collect();
    ids_score.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    ids_score.sort_by(|a, b| a.1.cmp(&b.1).reverse());
    state.search_entries = ids_score.into_iter().map(|(data, _)| data.into()).collect();
    if state.list_state.selected().is_none() && !state.search_entries.is_empty() {
        state.list_state.select(Some(0));
    }
}

fn initialize_search(mut commands: Commands, query: Query<&BodyInfo>) {
    let search_entries: Vec<_> = query.iter().map(|BodyInfo(data)| data.into()).collect();
    commands.insert_resource(SearchState {
        search_entries,
        search_character_index: 0,
        search_input: String::new(),
        search_matcher: SkimMatcherV2::default(),
        list_state: ListState::default(),
    });
}

fn reset_on_enter_search(
    mut search_state: ResMut<SearchState>,
    mut reader: EventReader<WindowEvent>,
) {
    reader
        .read()
        .find(|event| matches!(event, WindowEvent::ChangeFocus(FocusView::Search)))
        .inspect(|_| search_state.reset_search());
}

fn handle_search_events(
    mut search: ResMut<SearchState>,
    mut reader: EventReader<SearchViewEvent>,
    mut tree: Option<ResMut<TreeState>>,
) {
    use Direction2::*;
    use SearchViewEvent::*;
    for event in reader.read() {
        match event {
            DeleteChar => search.delete_char(),
            MoveCursor(d) => match d {
                Down => search.move_cursor_left(),
                Up => search.move_cursor_right(),
            },
            SelectSearch(d) => match d {
                Down => search.select_next_search(),
                Up => search.select_previous_search(),
            },
            ValidateSearch => {
                if let Some(ref mut tree) = tree {
                    if let Some(id) = search.selected_body_id() {
                        tree.select_body(id);
                    }
                }
            }
            WriteChar(char) => search.enter_char(*char),
        }
    }
}

// Code from https://ratatui.rs/examples/apps/user_input/
impl SearchState {
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
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{
        app::body_data::BodyType,
        core_plugin::CorePlugin,
        ui_plugin::{
            search_plugin::{SearchPlugin, SearchState, SearchViewEvent},
            tree_plugin::{TreePlugin, TreeState},
        },
    };

    #[test]
    fn test_search() {
        let mut app = App::new();
        app.add_plugins((
            CorePlugin {
                smallest_body_type: BodyType::Moon,
            },
            SearchPlugin,
            TreePlugin,
        ));
        app.update();
        let mut search = app.world.resource_mut::<SearchState>();
        search.search_input = "Moo".into();
        app.update();
        app.world.send_event(SearchViewEvent::ValidateSearch);
        app.update();
        let id = app.world.resource::<TreeState>().selected_body_id();
        assert_eq!(id, "lune".into())
    }
}
