use bevy::prelude::*;
use bevy_ratatui::{error::exit_on_error, terminal::RatatuiContext, RatatuiPlugins};
use ratatui::layout::{Constraint, Layout};

use crate::core_plugin::GameSet;

use self::{
    info_plugin::{InfoToggle, InfoWidget},
    search_plugin::{SearchState, SearchViewEvent, SearchWidget},
    space_map_plugin::SpaceMap,
    tree_plugin::{TreeState, TreeWidget},
};

pub mod info_plugin;
pub mod search_plugin;
pub mod space_map_plugin;
pub mod tree_plugin;

pub struct TuiPlugin;

impl Plugin for TuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RatatuiPlugins::default())
            .add_event::<WindowEvent>()
            .insert_state(FocusView::default())
            .add_systems(
                Update,
                (
                    handle_window_events,
                    handle_search_validate.run_if(resource_exists::<SearchState>),
                )
                    .in_set(GameSet),
            )
            .add_systems(PostUpdate, render.pipe(exit_on_error).in_set(GameSet));
    }
}

#[derive(SystemSet, Debug, Clone, Hash, PartialEq, Eq)]
pub struct UiInitSet;

#[derive(Default, Copy, Clone, States, PartialEq, Eq, Debug, Hash)]
pub enum FocusView {
    #[default]
    Tree,
    Search,
}
#[derive(Debug, Event)]
pub enum WindowEvent {
    ChangeFocus(FocusView),
}

fn handle_window_events(
    mut focus_view: ResMut<NextState<FocusView>>,
    mut reader: EventReader<WindowEvent>,
) {
    for event in reader.read() {
        match *event {
            WindowEvent::ChangeFocus(new_focus) => focus_view.set(new_focus),
        }
    }
}

fn handle_search_validate(
    mut focus_view: ResMut<NextState<FocusView>>,
    mut reader: EventReader<SearchViewEvent>,
) {
    for event in reader.read() {
        match event {
            SearchViewEvent::ValidateSearch => focus_view.set(FocusView::Tree),
            _ => continue,
        }
    }
}

fn render(
    mut ctx: ResMut<RatatuiContext>,
    tree: Option<ResMut<TreeState>>,
    search: Option<ResMut<SearchState>>,
    space_map: Option<Res<SpaceMap>>,
    focus: Res<State<FocusView>>,
    is_info_toggled: Option<Res<InfoToggle>>,
    info_widget: Option<Res<InfoWidget>>,
) -> color_eyre::Result<()> {
    ctx.draw(|f| {
        let mut c = vec![Constraint::Percentage(25), Constraint::Fill(1)];
        if let Some(ref toggle) = is_info_toggled {
            if toggle.0 {
                c.push(Constraint::Percentage(25));
            }
        }
        let chunks = Layout::horizontal(c).split(f.size());

        match focus.get() {
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
        }
        if let Some(space_map) = space_map {
            f.render_widget(space_map.as_ref(), chunks[1]);
        }
        if let Some(info_widget) = info_widget {
            if is_info_toggled.unwrap().0 {
                f.render_widget(info_widget.as_ref(), chunks[2]);
            }
        }
    })?;
    Ok(())
}
