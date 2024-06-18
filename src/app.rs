pub mod body_data;
pub mod body_id;
pub mod info;
mod input;

use std::{
    collections::HashMap,
    error::Error,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use nalgebra::Vector3;

use crate::{
    app::body_data::BodyType,
    engine::Engine,
    ui::{events::UiEvent, Tui, UiState},
    utils::de::read_main_bodies,
};

use self::{body_data::BodyData, body_id::BodyID, info::SystemInfo};

// frame rate in fps
const FRAME_RATE: f64 = 60.;

// Fixed update time step
pub const TIME_STEP: Duration = Duration::from_millis(12);

pub type GlobalMap = HashMap<BodyID, Vector3<i64>>;

pub struct App {
    pub engine: Engine,
    pub tui: Option<Tui>,
    pub ui: UiState,
    pub shared_info: Arc<SystemInfo>,
    pub current_map: Arc<Mutex<GlobalMap>>,
    pub next_map: Arc<Mutex<GlobalMap>>,
    pub time_switch: bool,
    ui_event_sender: Sender<UiEvent>,
}

pub enum AppMessage {
    Quit,
    Idle,
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
        let (ui_event_sender, ui_event_receiver) = mpsc::channel();
        Ok(Self {
            engine,
            tui: if headless {
                None
            } else {
                Some(UiState::setup_tui()?)
            },
            ui: UiState::new(
                Arc::clone(&shared_info),
                Arc::clone(&current_map),
                ui_event_receiver,
            )?,
            current_map,
            next_map,
            shared_info,
            time_switch: true,
            ui_event_sender,
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
            if let Ok(AppMessage::Quit) = self.handle_input() {
                break;
            }
            let current_time = Instant::now();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            if self.time_switch {
                lag += elapsed;
                while lag >= TIME_STEP {
                    self.engine.update();
                    self.swap_buffers();
                    lag -= TIME_STEP;
                }
            }
            if let Some(tui) = &mut self.tui {
                self.ui.handle_events()?;
                self.ui.render(tui)?;
            }
        }
        Ok(())
    }

    fn toggle_time_switch(&mut self) {
        self.time_switch = !self.time_switch
    }

    fn swap_buffers(&mut self) {
        let buf = Arc::clone(&self.current_map);
        self.current_map = Arc::clone(&self.next_map);
        self.next_map = buf;
        self.engine.global_map = Arc::clone(&self.next_map);
        self.ui.global_map = Arc::clone(&self.current_map);
    }
}
impl Drop for App {
    fn drop(&mut self) {
        if self.tui.is_some() {
            UiState::reset_tui().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::algebra::inorm;

    use super::App;

    #[test]
    fn test_swap_buffers() {
        let mut app = App::new_moons(true).unwrap();
        app.engine.update();
        app.swap_buffers();

        let global = app.ui.global_map.lock().unwrap();
        let local = &app.engine.bodies;
        let moon = "lune".into();
        assert!(
            (inorm(global[&moon]) - inorm(local[&"terre".into()].position)).abs()
                <= inorm(local[&moon].position)
        )
    }
}
