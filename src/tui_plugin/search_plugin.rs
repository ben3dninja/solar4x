use bevy::prelude::*;
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
    utils::{list::ClampedList, ui::Direction2},
};

#[derive(Debug)]
pub enum SearchEvent {
    Select(Direction2),
    ValidateSearch,
    WriteChar(char),
    DeleteChar,
}

pub struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SearchMatcher(SkimMatcherV2::default()));
    }
}

#[derive(Resource)]
pub struct SearchMatcher(pub SkimMatcherV2);

pub struct SearchState {
    search_entries: Vec<SearchEntry>,
    list_state: ListState,
    search_input: String,
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

impl ClampedList for SearchState {
    fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    fn len(&self) -> usize {
        self.search_entries.len()
    }
}

impl SearchState {
    pub fn new<'a>(bodies: impl Iterator<Item = &'a BodyData>) -> SearchState {
        let search_entries: Vec<_> = bodies.map(|data| data.into()).collect();
        SearchState {
            search_entries,
            search_input: String::new(),
            list_state: ListState::default(),
        }
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.search_input.push(new_char);
    }

    pub fn delete_char(&mut self) {
        self.search_input.pop();
    }

    pub fn selected_body_id(&self) -> Option<BodyID> {
        self.list_state
            .selected()
            .and_then(|i| self.search_entries.get(i))
            .map(|entry| entry.id)
    }

    pub fn reset_search(&mut self) {
        self.search_input = String::new();
        self.list_state.select(Some(0));
    }

    pub fn update_search_entries<'a>(
        &mut self,
        bodies: impl Iterator<Item = &'a BodyInfo>,
        fuzzy_matcher: &SkimMatcherV2,
    ) {
        let mut ids_score: Vec<_> = bodies
            .filter_map(|BodyInfo(body)| {
                fuzzy_matcher
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
        client_plugin::ClientPlugin,
        core_plugin::BodiesConfig,
        tui_plugin::{
            explorer_screen::ExplorerEvent, search_plugin::SearchEvent, AppScreen, TuiPlugin,
        },
    };

    #[test]
    fn test_search() {
        let mut app = App::new();
        app.add_plugins((
            ClientPlugin::testing(BodiesConfig::SmallestBodyType(BodyType::Moon)),
            TuiPlugin::testing(),
        ));
        app.update();
        app.update();
        use ExplorerEvent::Search;
        use SearchEvent::*;
        app.world.send_event_batch([
            Search(WriteChar('M')),
            Search(WriteChar('o')),
            Search(WriteChar('o')),
        ]);
        app.update();

        app.world
            .send_event(ExplorerEvent::Search(SearchEvent::ValidateSearch));
        app.update();
        if let AppScreen::Explorer(ctx) = app.world.resource_mut::<AppScreen>().as_mut() {
            let id = ctx.tree_state.selected_body_id();
            assert_eq!(id, "lune".into())
        }
    }
}
