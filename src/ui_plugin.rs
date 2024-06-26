use bevy::prelude::*;
use bevy_ratatui::{error::exit_on_error, terminal::RatatuiContext, RatatuiPlugins};
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        block::Title,
        canvas::{Canvas, Circle},
        Block, Borders, Clear, List, ListState, Paragraph, Widget,
    },
    Frame,
};

use self::{
    search_plugin::{SearchViewEvent, SearchWidget},
    space_map_plugin::SpaceMap,
    tree_plugin::{TreeState, TreeWidg},
};

pub mod search_plugin;
pub mod space_map_plugin;
pub mod tree_plugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RatatuiPlugins::default())
            .add_event::<WindowEvent>()
            .insert_resource(FocusView::default())
        .add_systems(Update, (handle_window_events, handle_search_validate.run_if(resource_exists::<SearchWidget>)))
    .add_systems(PostUpdate, render.pipe(exit_on_error))
        // .add_systems(PostUpdate, render)
        ;
    }
}

#[derive(Default, Copy, Clone, Resource, PartialEq, Debug)]
pub enum FocusView {
    #[default]
    Tree,
    Search,
    Info,
}

#[derive(Debug, Event)]
pub enum WindowEvent {
    ChangeFocus(FocusView),
}

fn handle_window_events(mut focus_view: ResMut<FocusView>, mut reader: EventReader<WindowEvent>) {
    for event in reader.read() {
        match *event {
            WindowEvent::ChangeFocus(new_focus) => *focus_view = new_focus,
        }
    }
}

fn handle_search_validate(
    mut focus_view: ResMut<FocusView>,
    mut reader: EventReader<SearchViewEvent>,
) {
    for event in reader.read() {
        match event {
            SearchViewEvent::ValidateSearch => *focus_view = FocusView::Tree,
            _ => continue,
        }
    }
}

fn render(
    mut ctx: ResMut<RatatuiContext>,
    mut tree: ResMut<TreeState>,
    space_map: Res<SpaceMap>,
) -> color_eyre::Result<()> {
    ctx.draw(|f| {
        let chunks =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).split(f.size());

        f.render_stateful_widget(TreeWidg, chunks[0], tree.as_mut());
        f.render_widget(space_map.as_ref(), chunks[1]);
    })?;
    Ok(())
}
// }

// fn draw_search(&mut self, f: &mut Frame, rect: Rect) {
//     let names: Vec<_> = self
//         .search_entries
//         .iter()
//         .filter_map(|entry| {
//             self.shared_info
//                 .bodies
//                 .get(entry)
//                 .map(|body| body.name.clone())
//         })
//         .collect();
//     let texts: Vec<Text> = names
//         .into_iter()
//         .map(|s| Text::styled(s, Style::default()))
//         .collect();
//     let search_bar = Paragraph::new(&self.search_input[..]).block(Block::bordered());
//     let list = List::new(texts)
//         .block(
//             Block::bordered().title(Title::from("Search view".bold()).alignment(Alignment::Center)),
//         )
//         .highlight_symbol("> ");
//     let chunks = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(rect);
//     f.render_widget(search_bar, chunks[0]);
//     f.render_stateful_widget(list, chunks[1], &mut self.search_state);
// }

// fn draw_popup(f: &mut Frame, main_body_info: BodyInfo) {
//     let main_body = state
//         .shared_info
//         .bodies
//         .get(&self.selected_body_id_tree())
//         .unwrap();
//     let popup_block = Block::default()
//         .title(&main_body.name[..])
//         .borders(Borders::ALL)
//         .style(Style::default().bg(Color::DarkGray));
//     let area = centered_rect(25, 25, f.size());
//     Clear.render(area, f.buffer_mut());
//     let info = Paragraph::new(format!(
//         "Body type: {}\n\
//             N of orbiting bodies: {}\n\
//             Radius: {} km\n\
//             Revolution period: {} earth days",
//         main_body.body_type,
//         main_body.orbiting_bodies.len(),
//         main_body.radius,
//         main_body.revolution_period,
//     ))
//     .block(popup_block);
//     f.render_widget(info, area);
// }
