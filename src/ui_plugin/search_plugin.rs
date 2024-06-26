use bevy::prelude::*;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{app::body_id::BodyID, core_plugin::BodyInfo, utils::ui::Direction2};

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
                        reset_on_enter_search.run_if(resource_exists::<FocusView>),
                        handle_search_events,
                    ),
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
pub struct SearchWidget {
    search_entries: Vec<BodyID>,
    // search_state: ListState,
    selected_index: Option<usize>,
    search_character_index: usize,
    search_input: String,
    search_matcher: SkimMatcherV2,
}
// TODO : only on key event
fn update_search_entries(mut state: ResMut<SearchWidget>, query: Query<&BodyInfo>) {
    let mut ids_score: Vec<_> = query
        .iter()
        .filter_map(|BodyInfo(body)| {
            state
                .search_matcher
                .fuzzy_match(&body.name, &state.search_input)
                .map(|score| (body.id, score))
        })
        .collect();
    ids_score.sort_by(|a, b| a.0.cmp(&b.0));
    ids_score.sort_by(|a, b| a.1.cmp(&b.1).reverse());
    state.search_entries = ids_score.into_iter().map(|(id, _)| id).collect();
    if state.selected_index.is_none() && !state.search_entries.is_empty() {
        state.selected_index = Some(0);
    }
}

fn initialize_search(mut commands: Commands, query: Query<&BodyInfo>) {
    let search_entries: Vec<BodyID> = query.iter().map(|BodyInfo(data)| data.id).collect();
    commands.insert_resource(SearchWidget {
        search_entries,
        search_character_index: 0,
        search_input: String::new(),
        search_matcher: SkimMatcherV2::default(),
        selected_index: None,
    });
}

fn reset_on_enter_search(
    mut search_state: ResMut<SearchWidget>,
    mut reader: EventReader<WindowEvent>,
) {
    reader
        .read()
        .find(|event| matches!(event, WindowEvent::ChangeFocus(FocusView::Search)))
        .inspect(|_| search_state.reset_search());
}

fn handle_search_events(
    mut search: ResMut<SearchWidget>,
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
impl SearchWidget {
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
        self.selected_index
            .and_then(|i| self.search_entries.get(i).cloned())
    }

    pub fn select_next_search(&mut self) {
        self.selected_index = if let Some(index) = self.selected_index {
            Some((index + 1).min(self.search_entries.len() - 1))
        } else if !self.search_entries.is_empty() {
            Some(0)
        } else {
            None
        }
    }

    pub fn select_previous_search(&mut self) {
        self.selected_index = if let Some(index) = self.selected_index {
            Some(index.saturating_sub(1))
        } else if !self.search_entries.is_empty() {
            Some(0)
        } else {
            None
        }
    }

    pub fn reset_search(&mut self) {
        self.search_character_index = 0;
        self.search_input = String::new();
        self.selected_index = Some(0);
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{
        app::body_data::BodyType,
        core_plugin::CorePlugin,
        ui_plugin::{
            search_plugin::{SearchPlugin, SearchViewEvent, SearchWidget},
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
        let mut search = app.world.resource_mut::<SearchWidget>();
        search.search_input = "Moo".into();
        app.update();
        app.world.send_event(SearchViewEvent::ValidateSearch);
        app.update();
        let id = app.world.resource::<TreeState>().selected_body_id();
        assert_eq!(id, "lune".into())
    }
}
