mod events;
use std::{
    collections::HashMap,
    error::Error,
    io::Stdout,
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crossterm::event::{self, KeyCode, KeyEventKind};
use nalgebra::{Vector2, Vector3};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    bodies::{
        body_data::{BodyData, BodyType},
        body_id::BodyID,
        BodySystem,
    },
    engine::Engine,
    ui::{Tui, UiState},
    utils::de::read_main_bodies,
};

use self::events::AppMessage;

// frame rate in fps
const FRAME_RATE: f64 = 60.;

// Fixed update time step
pub const TIME_STEP: Duration = Duration::from_millis(12);

pub struct SystemInfo {
    pub bodies: HashMap<BodyID, BodyData>,
    pub primary_body: BodyID,
}

impl SystemInfo {
    fn new<T: IntoIterator<Item = BodyData>>(bodies: T) -> Option<Self> {
        let bodies: HashMap<BodyID, BodyData> =
            bodies.into_iter().map(|data| (data.id, data)).collect();
        if let Some(primary_body) = bodies
            .values()
            .find(|data| data.host_body.is_none())
            .map(|data| data.id)
        {
            Some(SystemInfo {
                bodies,
                primary_body,
            })
        } else {
            None
        }
    }
}

pub type GlobalMap = HashMap<BodyID, Vector3<i64>>;

pub struct App {
    pub engine: Engine,
    pub tui: Option<Tui>,
    pub ui: UiState,
    pub shared_info: Arc<SystemInfo>,
    pub current_map: Arc<Mutex<GlobalMap>>,
    pub next_map: Arc<Mutex<GlobalMap>>,
    pub time_switch: bool,
}

impl App {
    pub fn new_from_filter(
        f: impl FnMut(&BodyData) -> bool,
        headless: bool,
    ) -> std::io::Result<Self> {
        let bodies = read_main_bodies()?.into_iter().filter(f);
        let current_map = Arc::new(Mutex::new(GlobalMap::new()));
        let next_map = Arc::new(Mutex::new(GlobalMap::new()));
        let shared_info = Arc::new(
            SystemInfo::new(bodies)
                .ok_or(std::io::Error::other("no primary body found in data"))?,
        );
        let engine = Engine::new_from_data(Arc::clone(&next_map), Arc::clone(&shared_info));
        Ok(Self {
            engine,
            tui: if headless {
                None
            } else {
                Some(UiState::setup_tui()?)
            },
            ui: UiState::new(Arc::clone(&shared_info), Arc::clone(&current_map))?,
            current_map,
            next_map,
            shared_info,
            time_switch: true,
        })
    }

    pub fn new_simple(headless: bool) -> std::io::Result<Self> {
        Self::new_from_filter(
            |data| matches!(data.body_type, BodyType::Planet | BodyType::Star),
            headless,
        )
    }
    pub fn new_moons(headless: bool) -> std::io::Result<Self> {
        Self::new_from_filter(
            |data| {
                matches!(
                    data.body_type,
                    BodyType::Planet | BodyType::Star | BodyType::Moon
                )
            },
            headless,
        )
    }
    pub fn new_complete(headless: bool) -> std::io::Result<Self> {
        Self::new_from_filter(|_| true, headless)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut previous_time = Instant::now();
        let mut lag = Duration::ZERO;
        loop {
            let current_time = Instant::now();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            lag += elapsed;
            if let Ok(AppMessage::Quit) = self.handle_events() {
                break;
            }
            while lag >= TIME_STEP {
                self.engine.update();
                lag -= TIME_STEP;
            }
            self.ui.render();
        }
        Ok(())
    }

    fn toggle_time_switch(&mut self) {
        self.time_switch = !self.time_switch
    }
}

#[cfg(test)]
mod tests {
    use super::App;

    #[test]
    fn test_select_body() {
        let mut app = App::new_simple().unwrap();
        app.select_body(&"terre".into());
        assert_eq!(app.selected_body_id_tree(), "terre".into())
    }
}
