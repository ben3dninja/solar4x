use bevy::{math::DVec3, prelude::*};
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, List, ListState, StatefulWidget},
};

use crate::{
    core_plugin::{BodyInfo, PrimaryBody},
    engine_plugin::{Position, Velocity},
    gravity::Influenced,
    keyboard::Keymap,
    main_game::trajectory::ManeuverNode,
    spaceship::{ShipID, ShipsMapping},
    utils::{list::ClampedList, ui::Direction2},
    GAMETIME_PER_UPDATE,
};

use super::AppScreen;
pub struct EditorPlugin;

#[derive(Event)]
pub enum EditorEvent {
    Select(Direction2),
    NewNode,
    Back,
}

#[derive(Resource, Default)]
pub struct EditorContext {
    ship: ShipID,
    pos: DVec3,
    speed: DVec3,
    list_state: ListState,
    nodes: Vec<ManeuverNode>,
}

impl EditorContext {
    pub fn new(ship: ShipID, &Position(pos): &Position, &Velocity(speed): &Velocity) -> Self {
        Self {
            ship,
            pos,
            speed,
            ..Default::default()
        }
    }
}

pub struct EditorScreen;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EditorEvent>()
            .add_computed_state::<InEditor>()
            .add_systems(
                Update,
                (read_input, handle_editor_events)
                    .chain()
                    .run_if(in_state(InEditor)),
            )
            .add_systems(OnEnter(InEditor), create_screen)
            .add_systems(OnExit(InEditor), clear_screen);
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InEditor;

impl ComputedStates for InEditor {
    type SourceStates = AppScreen;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            AppScreen::Editor(_) => Some(Self),
            _ => None,
        }
    }
}

fn create_screen(
    mut commands: Commands,
    screen: Res<State<AppScreen>>,
    coords: Query<(&Position, &Velocity)>,
    mapping: Res<ShipsMapping>,
) {
    if let AppScreen::Editor(id) = screen.get() {
        if let Some(e) = mapping.0.get(id) {
            let (pos, speed) = coords.get(*e).unwrap();
            commands.insert_resource(EditorContext::new(*id, pos, speed));
        }
    }
}

fn clear_screen(mut commands: Commands) {
    commands.remove_resource::<EditorContext>();
}

fn read_input(
    mut key_event: EventReader<KeyEvent>,
    keymap: Res<Keymap>,
    mut internal_event: EventWriter<EditorEvent>,
) {
    use Direction2::*;
    use EditorEvent::*;
    let keymap = &keymap.editor;
    for event in key_event.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        internal_event.send(match event {
            e if keymap.select_next.matches(e) => Select(Down),
            e if keymap.select_previous.matches(e) => Select(Up),
            e if keymap.back.matches(e) => Back,
            e if keymap.new_node.matches(e) => NewNode,
            _ => return,
        });
    }
}

impl ClampedList for EditorContext {
    fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

pub fn handle_editor_events(
    mut screen: ResMut<NextState<AppScreen>>,
    mut context: ResMut<EditorContext>,
    mut events: EventReader<EditorEvent>,
    influencer: Query<&Influenced>,
    bodies: Query<&BodyInfo>,
    primary: Query<&BodyInfo, With<PrimaryBody>>,
    mapping: Res<ShipsMapping>,
) {
    for event in events.read() {
        match event {
            EditorEvent::Select(d) => context.select_adjacent(*d),
            EditorEvent::NewNode => {
                let origin = influencer
                    .get(mapping.0[&context.ship])
                    .unwrap()
                    .main_influencer
                    .map(|e| bodies.get(e).unwrap().0.id)
                    .unwrap_or(primary.single().0.id);
                let time = context
                    .nodes
                    .last()
                    .map(|n| n.time + GAMETIME_PER_UPDATE)
                    .unwrap_or_default();
                context.nodes.push(ManeuverNode {
                    name: "Node".into(),
                    time,
                    thrust: DVec3::ZERO,
                    origin,
                });
            }
            EditorEvent::Back => screen.set(AppScreen::Fleet),
        }
    }
}

impl StatefulWidget for EditorScreen {
    type State = EditorContext;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let chunks =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]).split(area);
        List::new(state.nodes.iter().map(|n| &n.name[..]))
            .highlight_symbol(">")
            .block(Block::bordered().title_top("Maneuver nodes"))
            .render(chunks[0], buf, &mut state.list_state);
    }
}
