use bevy::prelude::*;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::widgets::ListState;

use crate::{
    app::body_id::BodyID,
    core_plugin::BodyInfo,
    utils::{
        list::{select_next_clamp, select_previous_clamp},
        ui::Direction2,
    },
};

use super::{FocusView, WindowEvent};

pub struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SearchViewEvent>()
        // .add_systems(PostStartup, initialize_search)
        // .add_systems(
        //     Update,
        //     (reset_on_enter_search, update_search_entries).chain(),
        // )
        ;
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
pub struct UiSearchState {
    search_entries: Vec<BodyID>,
    search_state: ListState,
    search_character_index: usize,
    search_input: String,
    search_matcher: SkimMatcherV2,
}
// // TODO : only on key event
// fn update_search_entries(mut state: ResMut<UiSearchState>) {
//     state.search_entries = state.search(&state.search_input);
//     if state.search_state.selected().is_none() && !state.search_entries.is_empty() {
//         state.search_state.select(Some(0));
//     }
// }

// fn initialize_search(mut commands: Commands, query: Query<&BodyInfo>) {
//     let search_entries: Vec<BodyID> = query.iter().map(|BodyInfo(data)| data.id).collect();
//     commands.insert_resource(UiSearchState {
//         search_entries,
//         search_character_index: 0,
//         search_input: String::new(),
//         search_matcher: SkimMatcherV2::default(),
//         search_state: ListState::default(),
//     });
// }

// fn reset_on_enter_search(
//     mut search_state: ResMut<UiSearchState>,
//     mut reader: EventReader<WindowEvent>,
// ) {
//     reader
//         .read()
//         .find(|event| matches!(event, WindowEvent::ChangeFocus(FocusView::Search)))
//         .inspect(|_| search_state.reset_search());
// }

// Code from https://ratatui.rs/examples/apps/user_input/
// impl UiSearchState {
//     pub fn move_cursor_left(&mut self) {
//         let cursor_moved_left = self.search_character_index.saturating_sub(1);
//         self.search_character_index = self.clamp_cursor(cursor_moved_left);
//     }

//     pub fn move_cursor_right(&mut self) {
//         let cursor_moved_right = self.search_character_index.saturating_add(1);
//         self.search_character_index = self.clamp_cursor(cursor_moved_right);
//     }

//     pub fn enter_char(&mut self, new_char: char) {
//         let index = self.byte_index();
//         self.search_input.insert(index, new_char);
//         self.move_cursor_right();
//     }

//     /// Returns the byte index based on the character position.
//     ///
//     /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
//     /// the byte index based on the index of the character.
//     fn byte_index(&mut self) -> usize {
//         self.search_input
//             .char_indices()
//             .map(|(i, _)| i)
//             .nth(self.search_character_index)
//             .unwrap_or(self.search_input.len())
//     }

//     pub fn delete_char(&mut self) {
//         let is_not_cursor_leftmost = self.search_character_index != 0;
//         if is_not_cursor_leftmost {
//             // Method "remove" is not used on the saved text for deleting the selected char.
//             // Reason: Using remove on String works on bytes instead of the chars.
//             // Using remove would require special care because of char boundaries.

//             let current_index = self.search_character_index;
//             let from_left_to_current_index = current_index - 1;

//             // Getting all characters before the selected character.
//             let before_char_to_delete = self.search_input.chars().take(from_left_to_current_index);
//             // Getting all characters after selected character.
//             let after_char_to_delete = self.search_input.chars().skip(current_index);

//             // Put all characters together except the selected one.
//             // By leaving the selected one out, it is forgotten and therefore deleted.
//             self.search_input = before_char_to_delete.chain(after_char_to_delete).collect();
//             self.move_cursor_left();
//         }
//     }

//     fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
//         new_cursor_pos.clamp(0, self.search_input.chars().count())
//     }

//     pub fn validate_search(&mut self) {
//         self.set_explorer_mode(ExplorerMode::Tree);
//         if let Some(id) = self.selected_body_id_search() {
//             self.select_body(id)
//         }
//     }

//     pub fn search(&self, pattern: &str) -> Vec<BodyID> {
//         let mut ids_score: Vec<_> = self
//             .shared_info
//             .bodies
//             .values()
//             .filter_map(|body| {
//                 self.search_matcher
//                     .fuzzy_match(&body.name, pattern)
//                     .map(|score| (body.id, score))
//             })
//             .collect();
//         ids_score.sort_by(|a, b| a.0.cmp(&b.0));
//         ids_score.sort_by(|a, b| a.1.cmp(&b.1).reverse());
//         ids_score.into_iter().map(|(id, _)| id).collect()
//     }

//     pub fn selected_body_id_search(&self) -> Option<BodyID> {
//         self.search_entries
//             .get(self.search_state.selected().unwrap_or_default())
//             .cloned()
//     }

//     pub fn select_next_search(&mut self) {
//         select_next_clamp(&mut self.search_state, self.search_entries.len() - 1)
//     }

//     pub fn select_previous_search(&mut self) {
//         select_previous_clamp(&mut self.search_state, 0)
//     }

//     pub fn reset_search(&mut self) {
//         self.search_character_index = 0;
//         self.search_input = String::new();
//         self.search_state.select(Some(0));
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::{app::body_data::BodyType, standalone::Standalone, ui::ExplorerMode};

//     #[test]
//     fn test_search() {
//         let (_, mut ui) = Standalone::new_testing(BodyType::Moon).unwrap();
//         ui.toggle_selection_expansion();
//         ui.select_next_tree();
//         ui.set_explorer_mode(ExplorerMode::Search);
//         ui.search_input = "Moo".into();
//         ui.update_search_selection();
//         ui.validate_search();
//         assert_eq!(ui.selected_body_id_tree(), "lune".into())
//     }
// }