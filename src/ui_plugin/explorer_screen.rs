use bevy::{prelude::*, utils::HashMap};
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyEvent as CKeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{StatefulWidget, StatefulWidgetRef, WidgetRef},
};

use crate::{
    bodies::body_id::BodyID,
    client_plugin::ClientMode,
    core_plugin::{
        BodiesMapping, BodyInfo, EventHandling, InputReading, LoadedSet, LoadingState, PrimaryBody,
        SystemSize,
    },
    engine_plugin::{EngineEvent, Position, ToggleTime},
    keyboard::Keymap,
    utils::{
        list::ClampedList,
        ui::{Direction2, Direction4},
    },
};

use super::{
    info_widget::InfoWidget,
    search_plugin::{SearchEvent, SearchMatcher, SearchPlugin, SearchState, SearchWidget},
    space_map_plugin::{SpaceMap, SpaceMapEvent, SpaceMapWidget},
    tree_widget::{TreeEvent, TreeState, TreeWidget},
    AppScreen, UiInit,
};

pub struct ExplorerScreenPlugin;

impl Plugin for ExplorerScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExplorerEvent>()
            .add_plugins(SearchPlugin)
            .add_systems(
                Update,
                (
                    read_input.in_set(InputReading),
                    handle_explorer_events.in_set(EventHandling),
                )
                    .in_set(LoadedSet)
                    .run_if(in_state(AppScreen::Explorer)),
            )
            .add_systems(
                Update,
                update_space_map
                    .run_if(resource_exists::<SpaceMap>)
                    .run_if(resource_equals(ToggleTime(true))),
            )
            .add_systems(
                OnEnter(LoadingState::Loaded),
                create_screen
                    .run_if(in_state(ClientMode::Explorer))
                    .in_set(UiInit),
            )
            .add_systems(
                OnExit(ClientMode::Explorer),
                clear_screen.run_if(not(in_state(AppScreen::Explorer))),
            );
    }
}

fn create_screen(
    mut commands: Commands,
    mut next_screen: ResMut<NextState<AppScreen>>,
    primary: Query<Entity, With<PrimaryBody>>,
    bodies: Query<&BodyInfo>,
    system_size: Res<SystemSize>,
) {
    let primary = primary.single();
    commands.insert_resource(SpaceMap::new(system_size.0, Some(primary), Some(primary)));
    commands.insert_resource(ExplorerContext::new(primary, &bodies));
    next_screen.set(AppScreen::Explorer);
}
fn clear_screen(mut commands: Commands) {
    commands.remove_resource::<SpaceMap>();
    commands.remove_resource::<ExplorerContext>();
}

#[derive(Default, Debug, Clone, Copy)]
pub enum SidePaneMode {
    #[default]
    Tree,
    Search,
}

#[derive(Resource)]
pub struct ExplorerContext {
    pub(super) side_pane_mode: SidePaneMode,
    pub(super) info_toggle: bool,
    pub(super) tree_state: TreeState,
    pub(super) search_state: SearchState,
    pub(super) info: InfoWidget,
    pub(super) space_map: SpaceMapWidget,
}

impl ExplorerContext {
    pub fn new(primary: Entity, bodies: &Query<&BodyInfo>) -> ExplorerContext {
        let primary_data = &bodies.get(primary).unwrap().0;
        let infos: Vec<_> = bodies.iter().map(|i| &i.0).collect();
        ExplorerContext {
            side_pane_mode: SidePaneMode::default(),
            info_toggle: false,
            tree_state: TreeState::new(primary_data, Some(primary_data), infos.clone().into_iter()),
            search_state: SearchState::new(infos.into_iter()),
            info: InfoWidget {
                body_info: primary_data.clone(),
            },
            space_map: SpaceMapWidget::default(),
        }
    }
    fn update_info(&mut self, mapping: &HashMap<BodyID, Entity>, bodies: &Query<&BodyInfo>) {
        let id = self.tree_state.selected_body_id();
        if let Ok(body_info) = bodies.get(mapping[&id]) {
            self.info.body_info = body_info.0.clone();
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
    Back,
}

fn read_input(
    context: Res<ExplorerContext>,
    mut key_event: EventReader<KeyEvent>,
    keymap: Res<Keymap>,
    mut internal_event: EventWriter<ExplorerEvent>,
) {
    use Direction2::*;
    use ExplorerEvent::*;
    use ViewEvent::*;
    for KeyEvent(event) in key_event.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        let keymap = &keymap.explorer;
        internal_event.send(match context.side_pane_mode {
            SidePaneMode::Tree => {
                let codes = &keymap.tree;
                use Direction4::*;
                use EngineEvent::*;
                use SpaceMapEvent::*;
                use TreeEvent::*;
                match event {
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
                    e if codes.back.matches(e) => View(ViewEvent::Back),
                    e if codes.speed_up.matches(e) => Engine(EngineSpeed(Up)),
                    e if codes.slow_down.matches(e) => Engine(EngineSpeed(Down)),
                    e if codes.toggle_time.matches(e) => Engine(ToggleTime),
                    _ => return,
                }
            }
            SidePaneMode::Search => {
                use SearchEvent::*;
                let codes = &keymap.search;
                match event {
                    e if codes.delete_char.matches(e) => Search(DeleteChar),
                    e if codes.validate_search.matches(e) => Search(ValidateSearch),
                    e if codes.select_next.matches(e) => Search(Select(Down)),
                    e if codes.select_previous.matches(e) => Search(Select(Up)),
                    e if codes.leave_search.matches(e) => {
                        View(ChangeSidePaneMode(SidePaneMode::Tree))
                    }
                    CKeyEvent {
                        code: KeyCode::Char(char),
                        ..
                    } => Search(WriteChar(*char)),
                    _ => return,
                }
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_explorer_events(
    mut ctx: ResMut<ExplorerContext>,
    mut space_map: ResMut<SpaceMap>,
    mut next_mode: ResMut<NextState<ClientMode>>,
    mut events: EventReader<ExplorerEvent>,
    mapping: Res<BodiesMapping>,
    bodies: Query<&BodyInfo>,
    mut engine_events: Option<ResMut<Events<EngineEvent>>>,
    fuzzy_matcher: Res<SearchMatcher>,
) {
    for event in events.read() {
        match event {
            ExplorerEvent::Tree(event) => {
                use TreeEvent::*;
                match event {
                    Select(d) => {
                        ctx.tree_state.select_adjacent(*d);
                        ctx.update_info(&mapping.0, &bodies);
                        space_map.selected = Some(mapping.0[&ctx.tree_state.selected_body_id()]);
                    }
                    ToggleTreeExpansion => ctx.tree_state.toggle_selection_expansion(),
                }
            }
            ExplorerEvent::Search(event) => {
                use SearchEvent::*;
                match event {
                    DeleteChar => {
                        ctx.search_state.delete_char();
                        ctx.search_state
                            .update_search_entries(bodies.iter(), &fuzzy_matcher.0);
                    }
                    Select(d) => ctx.search_state.select_adjacent(*d),
                    ValidateSearch => {
                        if let Some(id) = ctx.search_state.selected_body_id() {
                            ctx.tree_state.select_body(id);
                            space_map.selected =
                                Some(mapping.0[&ctx.tree_state.selected_body_id()]);
                            ctx.update_info(&mapping.0, &bodies);
                        }
                        ctx.side_pane_mode = SidePaneMode::Tree;
                    }
                    WriteChar(char) => {
                        ctx.search_state.enter_char(*char);
                        ctx.search_state
                            .update_search_entries(bodies.iter(), &fuzzy_matcher.0);
                    }
                }
            }
            ExplorerEvent::SpaceMap(event) => {
                use SpaceMapEvent::*;
                match event {
                    Zoom(d) => space_map.zoom(*d),
                    MapOffset(d) => space_map.offset(*d),
                    MapOffsetReset => space_map.reset_offset(),
                    FocusBody => {
                        if let Some(entity) = mapping.0.get(&ctx.tree_state.selected_body_id()) {
                            space_map.focus_body = Some(*entity);
                            ctx.tree_state.focus_body = Some(bodies.get(*entity).unwrap().0.id)
                        }
                    }
                    Autoscale => space_map.autoscale(&mapping.0, &bodies),
                }
            }
            ExplorerEvent::View(event) => match *event {
                ViewEvent::ChangeSidePaneMode(new_focus) => {
                    ctx.search_state.reset_search();
                    ctx.side_pane_mode = new_focus
                }

                ViewEvent::ToggleInfo => ctx.info_toggle = !ctx.info_toggle,
                ViewEvent::Back => next_mode.set(ClientMode::None),
            },
            ExplorerEvent::Engine(event) => {
                engine_events.as_mut().map(|e| e.send(*event));
            }
        }
    }
}

fn update_space_map(
    mut ctx: ResMut<ExplorerContext>,
    space_map: Res<SpaceMap>,
    query: Query<(Entity, &Position, &BodyInfo)>,
) {
    ctx.space_map.update_map(space_map.as_ref(), &query);
}

pub struct ExplorerScreen<'a> {
    pub map: &'a mut SpaceMap,
}

impl StatefulWidget for ExplorerScreen<'_> {
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
        state.space_map.render_ref(chunks[1], buf, self.map);
        if state.info_toggle {
            state.info.render_ref(chunks[2], buf);
        }
    }
}
