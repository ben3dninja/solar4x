use std::{error::Error, num::ParseFloatError};

use arrayvec::CapacityError;
use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Stylize,
    widgets::{Block, Clear, List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{
    keyboard::FleetScreenKeymap,
    main_game::{GameStage, InGame, ShipEvent},
    spaceship::{ShipID, ShipInfo},
    utils::{
        ecs::exit_on_error_if_app,
        list::{ClampedList, OptionsList},
        ui::{centered_rect, Direction2},
    },
    MAX_ID_LENGTH,
};

use super::{AppScreen, ChangeAppScreen, ContextUpdate, ScreenContext};
pub struct FleetScreenPlugin;

impl Plugin for FleetScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FleetScreenEvent>()
            .add_systems(Update, handle_fleet_events.pipe(exit_on_error_if_app))
            .add_systems(
                PostUpdate,
                update_fleet_context
                    .in_set(ContextUpdate)
                    .run_if(state_changed::<GameStage>.or_else(on_event::<ShipEvent>())),
            )
            .add_systems(OnEnter(InGame), change_screen_to_fleet);
    }
}

#[derive(Event, Clone)]
pub enum FleetScreenEvent {
    Select(Direction2),
    TryNewShip(Result<ShipInfo, ShipCreationError>),
}

#[derive(Clone, Debug)]
pub enum ShipCreationError {
    ParseError(ParseFloatError),
    IDTooLong,
    ShipAlreadyExists(ShipID),
}

impl From<ParseFloatError> for ShipCreationError {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseError(value)
    }
}

impl From<CapacityError> for ShipCreationError {
    fn from(_value: CapacityError) -> Self {
        Self::IDTooLong
    }
}

impl Error for ShipCreationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ShipCreationError::ParseError(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for ShipCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipCreationError::ParseError(e) => {
                write!(f, "Parsing error while creating ship: {}", e)
            }
            ShipCreationError::ShipAlreadyExists(id) => write!(
                f,
                "Couldn't create ship with id \"{}\" because it already exists",
                id
            ),
            ShipCreationError::IDTooLong => write!(
                f,
                "Couldn't create ship because id is too long (max length = {})",
                MAX_ID_LENGTH
            ),
        }
    }
}

#[derive(Default)]
pub struct FleetContext {
    list_state: ListState,
    ships: Vec<ShipInfo>,
    popup_context: Option<CreateShipContext>,
    stage: GameStage,
}

impl ClampedList for FleetContext {
    fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    fn len(&self) -> usize {
        self.ships.len()
    }
}

#[derive(Default)]
struct CreateShipContext {
    id_text: String,
    pos_x: String,
    pos_y: String,
    pos_z: String,
    speed_x: String,
    speed_y: String,
    speed_z: String,
    selected: usize,
}

impl OptionsList<7> for CreateShipContext {
    fn current_index(&mut self) -> &mut usize {
        &mut self.selected
    }

    fn fields_list(&mut self) -> [(&mut String, String); 7] {
        [
            (&mut self.id_text, "Ship ID".into()),
            (&mut self.pos_x, "Spawn x".into()),
            (&mut self.pos_y, "Spawn y".into()),
            (&mut self.pos_z, "Spawn z".into()),
            (&mut self.speed_x, "Velocity x".into()),
            (&mut self.speed_y, "Velocity y".into()),
            (&mut self.speed_z, "Velocity z".into()),
        ]
    }
}

impl CreateShipContext {
    fn to_info<'a>(
        &self,
        mut ships: impl Iterator<Item = &'a ShipInfo>,
    ) -> Result<ShipInfo, ShipCreationError> {
        let CreateShipContext {
            id_text,
            pos_x,
            pos_y,
            pos_z,
            speed_x,
            speed_y,
            speed_z,
            ..
        } = self;
        let spawn_pos = (pos_x.parse()?, pos_y.parse()?, pos_z.parse()?).into();
        let spawn_speed = (speed_x.parse()?, speed_y.parse()?, speed_z.parse()?).into();
        let id = ShipID::from(id_text).map_err(CapacityError::simplify)?;
        if ships.any(|s| s.id == id) {
            Err(ShipCreationError::ShipAlreadyExists(id))
        } else {
            Ok(ShipInfo {
                id,
                spawn_pos,
                spawn_speed,
            })
        }
    }
}

impl FleetContext {
    fn new(ships: impl Iterator<Item = ShipInfo>) -> Self {
        Self {
            ships: ships.collect(),
            ..Default::default()
        }
    }
    fn selected_ship_id(&self) -> Option<ShipID> {
        self.list_state.selected().map(|i| self.ships[i].id)
    }
}

pub struct FleetScreen;

impl ScreenContext for FleetContext {
    type ScreenEvent = FleetScreenEvent;

    type ScreenKeymap = FleetScreenKeymap;

    fn read_input(
        &mut self,
        key_event: &bevy_ratatui::event::KeyEvent,
        keymap: &Self::ScreenKeymap,
        internal_event: &mut Events<Self::ScreenEvent>,
    ) -> Option<ChangeAppScreen> {
        if key_event.kind == KeyEventKind::Release {
            return None;
        }
        use Direction2::*;
        use FleetScreenEvent::*;
        match &mut self.popup_context {
            None => match key_event {
                e if keymap.select_next.matches(e) => {
                    internal_event.send(Select(Down));
                }
                e if keymap.select_previous.matches(e) => {
                    internal_event.send(Select(Up));
                }
                e if keymap.edit_trajectory.matches(e) => {
                    if let Some(id) = self.selected_ship_id() {
                        return Some(ChangeAppScreen::TrajectoryEditor(id));
                    }
                }
                e if keymap.new_ship.matches(e) => {
                    self.popup_context = Some(CreateShipContext::default())
                }
                e if keymap.back.matches(e) => return Some(ChangeAppScreen::StartMenu),
                _ => {}
            },
            Some(ctx) => match key_event {
                e if keymap.cycle_create_options.matches(e) => ctx.select_next(),
                e if keymap.back.matches(e) => self.popup_context = None,
                e if keymap.validate_new_ship.matches(e) => {
                    internal_event.send(TryNewShip(ctx.to_info(self.ships.iter())));
                }
                e if keymap.delete_char.matches(e) => {
                    ctx.selected_field().pop();
                }
                KeyEvent(crossterm::event::KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => ctx.selected_field().push(*c),

                _ => {}
            },
        }
        None
    }
}

fn change_screen_to_fleet(mut screen: ResMut<AppScreen>, ships: Query<&ShipInfo>) {
    *screen = AppScreen::Fleet(FleetContext::new(ships.iter().cloned()));
}

fn handle_fleet_events(
    mut screen: ResMut<AppScreen>,
    mut events: EventReader<FleetScreenEvent>,
    mut ship_events: EventWriter<ShipEvent>,
) -> color_eyre::eyre::Result<()> {
    if let AppScreen::Fleet(context) = screen.as_mut() {
        for event in events.read() {
            match event {
                FleetScreenEvent::Select(d) => context.select_adjacent(*d),
                FleetScreenEvent::TryNewShip(info) => {
                    let info = info.clone()?;
                    context.ships.push(info.clone());
                    ship_events.send(ShipEvent::Create(info.clone()));
                    context.popup_context = None;
                }
            }
        }
    }
    Ok(())
}

fn update_fleet_context(
    stage: Res<State<GameStage>>,
    ships: Query<&ShipInfo>,
    mut screen: ResMut<AppScreen>,
) {
    if let AppScreen::Fleet(ctx) = screen.as_mut() {
        ctx.stage = stage.get().clone();
        ctx.ships.retain(|i| ships.iter().any(|j| i == j));
        ctx.ships.extend(
            ships
                .iter()
                .find(|i| !ctx.ships.iter().any(|j| *i == j))
                .cloned(),
        );
    }
}

impl StatefulWidget for FleetScreen {
    type State = FleetContext;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let entries = state.ships.iter().map(|s| s.id.to_string());
        let list = List::new(entries).highlight_symbol(">").block(
            Block::bordered()
                .title_top("Ships")
                .title_bottom(format!("Current stage: {}", state.stage)),
        );
        <List as StatefulWidget>::render(list, area, buf, &mut state.list_state);
        if let Some(ctx) = &mut state.popup_context {
            let popup = centered_rect(60, 60, area);
            Clear.render(popup, buf);
            let chunks =
                Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(popup);
            Paragraph::new("Create ship".bold())
                .alignment(Alignment::Center)
                .render(chunks[0], buf);
            let body = Layout::horizontal([Constraint::Percentage(50), Constraint::Fill(1)])
                .split(chunks[1]);
            ctx.paragraph(0).render(body[0], buf);
            let mut constraints = [Constraint::Percentage(100 / 6)].repeat(6);
            constraints.push(Constraint::Fill(1));
            let coords = Layout::vertical(constraints).split(body[1]);
            for i in 1..7 {
                ctx.paragraph(i).render(coords[i - 1], buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{app::App, prelude::default, state::state::NextState};

    use crate::{
        bodies::body_id::id_from,
        client_plugin::{ClientMode, ClientPlugin},
        main_game::{GameStage, ShipEvent},
        spaceship::{ShipInfo, ShipsMapping},
        ui_plugin::{AppScreen, TuiPlugin},
    };

    use super::{CreateShipContext, FleetScreenEvent};

    fn new_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            ClientPlugin::testing().in_mode(ClientMode::Singleplayer),
            TuiPlugin::testing(),
        ));
        app.update();
        app
    }

    #[test]
    fn test_create_ship() {
        let mut app = new_app();
        let popup = CreateShipContext {
            id_text: "s".into(),
            pos_x: "1000".into(),
            pos_y: "0".into(),
            pos_z: "0".into(),
            speed_x: "0".into(),
            speed_y: "1000".into(),
            speed_z: "0".into(),
            selected: 0,
        };
        app.world_mut().send_event(FleetScreenEvent::TryNewShip(
            popup.to_info(Vec::new().iter()),
        ));
        app.update();
        app.update();
        assert_eq!(app.world().resource::<ShipsMapping>().0.len(), 1)
    }

    #[test]
    fn test_update_context() {
        let mut app = new_app();
        if let AppScreen::Fleet(ctx) = app.world().resource::<AppScreen>() {
            assert_eq!(ctx.ships.len(), 0);
            assert_eq!(ctx.stage, GameStage::Preparation);
        } else {
            unreachable!()
        }
        app.world_mut().send_event(ShipEvent::Create(ShipInfo {
            id: id_from("s"),
            ..default()
        }));
        app.world_mut()
            .resource_mut::<NextState<GameStage>>()
            .set(GameStage::Action);
        app.update();
        if let AppScreen::Fleet(ctx) = app.world().resource::<AppScreen>() {
            assert_eq!(ctx.ships.len(), 1);
            assert_eq!(ctx.stage, GameStage::Action);
        } else {
            unreachable!()
        }
    }
}
