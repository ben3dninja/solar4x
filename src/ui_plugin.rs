use bevy::prelude::*;
use bevy_ratatui::{error::exit_on_error, terminal::RatatuiContext, RatatuiPlugins};
use ratatui::layout::{Constraint, Layout};

use self::{
    search_plugin::{SearchState, SearchViewEvent, SearchWidget},
    space_map_plugin::SpaceMap,
    tree_plugin::{TreeState, TreeWidget},
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
        .add_systems(Update, (handle_window_events, handle_search_validate.run_if(resource_exists::<SearchState>)))
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
    Switching,
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
    tree: Option<ResMut<TreeState>>,
    search: Option<ResMut<SearchState>>,
    space_map: Option<Res<SpaceMap>>,
    focus: Res<FocusView>,
) -> color_eyre::Result<()> {
    ctx.draw(|f| {
        let chunks =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).split(f.size());

        match *focus {
            FocusView::Tree => {
                if let Some(mut tree) = tree {
                    f.render_stateful_widget(TreeWidget, chunks[0], tree.as_mut());
                }
            }
            FocusView::Search => {
                if let Some(mut search) = search {
                    f.render_stateful_widget(SearchWidget, chunks[0], search.as_mut());
                }
            }
            _ => {}
        }
        if let Some(space_map) = space_map {
            f.render_widget(space_map.as_ref(), chunks[1]);
        }
    })?;
    Ok(())
}
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
