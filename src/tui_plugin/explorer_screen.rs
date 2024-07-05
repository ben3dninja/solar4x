use bevy::{prelude::*, utils::HashMap};
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyEvent as CKeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{StatefulWidget, WidgetRef},
};

use crate::{
    bodies::body_id::BodyID,
    core_plugin::{BodyInfo, EntityMapping, GameSet},
    engine_plugin::{EngineEvent, Position},
    keyboard::ExplorerKeymap,
    utils::{
        list::ClampedList,
        ui::{Direction2, Direction4},
    },
};

use super::{
    info_plugin::InfoWidget,
    search_plugin::{SearchEvent, SearchState, SearchWidget},
    space_map_plugin::{SpaceMap, SpaceMapEvent},
    tree_plugin::{TreeEvent, TreeState, TreeWidget},
    AppScreen, ChangeAppScreen, ScreenContext,
};

pub struct ExplorerPlugin;

impl Plugin for ExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExplorerEvent>()
            .insert_resource(ExplorerScreen)
            .add_systems(Update, handle_explorer_events.in_set(GameSet));
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum SidePaneMode {
    #[default]
    Tree,
    Search,
}

pub struct ExplorerContext {
    pub(super) side_pane_mode: SidePaneMode,
    pub(super) info_toggle: bool,
    pub(super) tree_state: TreeState,
    pub(super) search_state: SearchState,
    pub(super) space_map: SpaceMap,
    pub(super) info: InfoWidget,
    pub(super) focus_body: Entity,
}

impl ExplorerContext {
    pub fn new<'a, 'b>(
        primary: Entity,
        bodies: &'a Query<(&'b BodyInfo, &'b Position)>,
    ) -> ExplorerContext {
        let primary_data = &bodies.get(primary).unwrap().0 .0;
        let (infos, positions): (Vec<_>, Vec<_>) = bodies.iter().map(|(i, p)| (&i.0, p)).unzip();
        ExplorerContext {
            side_pane_mode: SidePaneMode::default(),
            info_toggle: false,
            tree_state: TreeState::new(primary_data, Some(primary_data), infos.clone().into_iter()),
            search_state: SearchState::new(infos.into_iter()),
            space_map: SpaceMap::new(positions.iter()),
            info: InfoWidget {
                body_info: primary_data.clone(),
            },
            focus_body: primary,
        }
    }
}

#[derive(Event)]
pub enum ExplorerEvent {
    Tree(TreeEvent),
    Search(SearchEvent),
    SpaceMap(SpaceMapEvent),
    View(ViewEvent),
    Engine(EngineEvent),
}

#[derive(Debug, Event)]
pub enum ViewEvent {
    ChangeSidePaneMode(SidePaneMode),
    ToggleInfo,
}

impl ScreenContext for ExplorerContext {
    type ScreenEvent = ExplorerEvent;

    type ScreenKeymap = ExplorerKeymap;

    fn read_input(
        &mut self,
        key_event: &KeyEvent,
        keymap: &Self::ScreenKeymap,
        internal_event: &mut Events<Self::ScreenEvent>,
    ) -> Option<ChangeAppScreen> {
        if key_event.kind == KeyEventKind::Release {
            return None;
        }
        use Direction2::*;
        use ExplorerEvent::*;
        use ViewEvent::*;
        internal_event.send(match self.side_pane_mode {
            SidePaneMode::Tree => {
                let codes = &keymap.tree;
                use Direction4::*;
                use EngineEvent::*;
                use SpaceMapEvent::*;
                use TreeEvent::*;
                match &key_event {
                    e if codes.select_next.matches(e) => Tree(Select(Down)),
                    e if codes.select_previous.matches(e) => Tree(Select(Up)),
                    e if codes.toggle_expand.matches(e) => Tree(ToggleTreeExpansion),
                    e if codes.zoom_in.matches(e) => SpaceMap(Zoom(Up)),
                    e if codes.zoom_out.matches(e) => SpaceMap(Zoom(Down)),
                    e if codes.map_offset_up.matches(e) => SpaceMap(MapOffset(Front)),
                    e if codes.map_offset_left.matches(e) => SpaceMap(MapOffset(Left)),
                    e if codes.map_offset_down.matches(e) => SpaceMap(MapOffset(Back)),
                    e if codes.map_offset_right.matches(e) => SpaceMap(MapOffset(Right)),
                    e if codes.map_offset_reset.matches(e) => SpaceMap(MapOffsetReset),
                    e if codes.focus.matches(e) => SpaceMap(FocusBody),
                    e if codes.autoscale.matches(e) => SpaceMap(Autoscale),
                    e if codes.enter_search.matches(e) => {
                        View(ChangeSidePaneMode(SidePaneMode::Search))
                    }
                    e if codes.toggle_info.matches(e) => View(ToggleInfo),
                    e if codes.quit.matches(e) => return Some(ChangeAppScreen::StartMenu),
                    e if codes.speed_up.matches(e) => Engine(EngineSpeed(Up)),
                    e if codes.slow_down.matches(e) => Engine(EngineSpeed(Down)),
                    e if codes.toggle_time.matches(e) => Engine(ToggleTime),
                    _ => return None,
                }
            }
            SidePaneMode::Search => {
                use SearchEvent::*;
                let codes = &keymap.search;
                match &key_event {
                    e if codes.delete_char.matches(e) => Search(DeleteChar),
                    e if codes.validate_search.matches(e) => Search(ValidateSearch),
                    e if codes.select_next.matches(e) => Search(Select(Down)),
                    e if codes.select_previous.matches(e) => Search(Select(Up)),
                    e if codes.leave_search.matches(e) => {
                        View(ChangeSidePaneMode(SidePaneMode::Tree))
                    }
                    KeyEvent(CKeyEvent {
                        code: KeyCode::Char(char),
                        ..
                    }) => Search(WriteChar(*char)),
                    _ => return None,
                }
            }
        });
        None
    }
}

impl ExplorerContext {
    fn update_info(&mut self, mapping: &HashMap<BodyID, Entity>, bodies: &Query<&BodyInfo>) {
        let id = self.tree_state.selected_body_id();
        if let Ok(body_info) = bodies.get(mapping[&id]) {
            self.info.body_info = body_info.0.clone();
        }
    }
}

pub fn handle_explorer_events(
    mut screen: ResMut<AppScreen>,
    mut events: EventReader<ExplorerEvent>,
    mapping: Res<EntityMapping>,
    bodies: Query<&BodyInfo>,
    mut engine_events: Option<ResMut<Events<EngineEvent>>>,
) {
    if let AppScreen::Explorer(context) = screen.as_mut() {
        for event in events.read() {
            match event {
                ExplorerEvent::Tree(event) => {
                    use TreeEvent::*;
                    match event {
                        Select(d) => {
                            context.tree_state.select_adjacent(*d);
                            context.update_info(&mapping.id_mapping, &bodies);
                        }
                        ToggleTreeExpansion => context.tree_state.toggle_selection_expansion(),
                    }
                }
                ExplorerEvent::Search(event) => {
                    use SearchEvent::*;
                    match event {
                        DeleteChar => {
                            context.search_state.delete_char();
                            context.search_state.update_search_entries(bodies.iter());
                        }
                        Select(d) => context.search_state.select_adjacent(*d),
                        ValidateSearch => {
                            if let Some(id) = context.search_state.selected_body_id() {
                                context.tree_state.select_body(id);
                                context.update_info(&mapping.id_mapping, &bodies);
                            }
                            context.side_pane_mode = SidePaneMode::Tree;
                        }
                        WriteChar(char) => {
                            context.search_state.enter_char(*char);
                            context.search_state.update_search_entries(bodies.iter());
                        }
                    }
                }
                ExplorerEvent::SpaceMap(event) => {
                    use SpaceMapEvent::*;
                    match event {
                        Zoom(d) => context.space_map.zoom(*d),
                        MapOffset(d) => context.space_map.offset(*d),
                        MapOffsetReset => context.space_map.reset_offset(),
                        FocusBody => {
                            if let Some(entity) = mapping
                                .id_mapping
                                .get(&context.tree_state.selected_body_id())
                            {
                                context.focus_body = *entity;
                                println!("{entity:?}");
                                context.tree_state.focus_body =
                                    Some(bodies.get(*entity).unwrap().0.id)
                            }
                        }
                        Autoscale => {
                            let focus_data = &bodies.get(context.focus_body).unwrap().0;
                            if let Some(max_dist) = focus_data
                                .orbiting_bodies
                                .iter()
                                .filter_map(|id| {
                                    mapping
                                        .id_mapping
                                        .get(id)
                                        .and_then(|&e| bodies.get(e).ok())
                                        .map(|body| body.0.semimajor_axis)
                                })
                                .max_by(|a, b| a.total_cmp(b))
                            {
                                context.space_map.zoom_level =
                                    context.space_map.system_size / max_dist;
                            }
                        }
                    }
                }
                ExplorerEvent::View(event) => match *event {
                    ViewEvent::ChangeSidePaneMode(new_focus) => {
                        context.search_state.reset_search();
                        context.side_pane_mode = new_focus
                    }

                    ViewEvent::ToggleInfo => context.info_toggle = !context.info_toggle,
                },
                ExplorerEvent::Engine(event) => {
                    engine_events.as_mut().map(|e| e.send(*event));
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct ExplorerScreen;

impl StatefulWidget for &ExplorerScreen {
    type State = ExplorerContext;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let mut c = vec![Constraint::Percentage(25), Constraint::Fill(1)];
        if state.info_toggle {
            c.push(Constraint::Percentage(25));
        }
        let chunks = Layout::horizontal(c).split(area);

        match state.side_pane_mode {
            SidePaneMode::Tree => TreeWidget.render(chunks[0], buf, &mut state.tree_state),
            SidePaneMode::Search => {
                SearchWidget.render(chunks[0], buf, &mut state.search_state);
            }
        }
        state.space_map.render_ref(chunks[1], buf);
        if state.info_toggle {
            state.info.render_ref(chunks[2], buf);
        }
    }
}