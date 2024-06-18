pub mod body_data;
pub mod body_id;
pub mod info;
mod input;

use std::{
    collections::HashMap,
    error::Error,
    io::Result as IoResult,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use nalgebra::Vector3;

use crate::{
    app::body_data::BodyType,
    engine::Engine,
    ui::{events::UiEvent, AppScreen, ExplorerMode, UiContext, UiState},
    utils::de::read_main_bodies,
};

use self::{body_data::BodyData, body_id::BodyID, info::SystemInfo};

// Fixed update time step
pub const TIME_STEP: Duration = Duration::from_millis(12);

pub type GlobalMap = HashMap<BodyID, Vector3<i64>>;

pub type AppError = Box<dyn Error + Send>;

pub struct App {
    pub engine: Engine,
    pub shared_info: Arc<SystemInfo>,
    pub current_map: Arc<Mutex<GlobalMap>>,
    pub next_map: Arc<Mutex<GlobalMap>>,
    pub time_switch: bool,
    ui_event_sender: Sender<UiEvent>,
    ui_handle: Option<JoinHandle<()>>,
    error_receiver: Receiver<AppError>,
    ui_context: Arc<Mutex<UiContext>>,
}

pub enum AppMessage {
    Quit,
    Idle,
}

impl App {
    fn new_from_filter(
        f: impl FnMut(&BodyData) -> bool,
        headless: bool,
        manual: bool,
    ) -> IoResult<(Self, Option<UiState>)> {
        let bodies = read_main_bodies()?.into_iter().filter(f);
        let current_map = Arc::new(Mutex::new(GlobalMap::new()));
        let next_map = Arc::new(Mutex::new(GlobalMap::new()));
        let shared_info = Arc::new(
            SystemInfo::new(bodies)
                .ok_or(std::io::Error::other("no primary body found in data"))?,
        );
        let engine = Engine::new_from_data(Arc::clone(&next_map), Arc::clone(&shared_info));
        let (ui_event_sender, ui_event_receiver) = mpsc::channel();
        let (error_sender, error_receiver) = mpsc::channel();
        let ui_context = Arc::new(Mutex::new(UiContext::default()));
        let tui = if headless {
            None
        } else {
            Some(UiState::setup_tui()?)
        };
        let mut ui = Some(UiState::new(
            Arc::clone(&shared_info),
            Arc::clone(&current_map),
            ui_event_receiver,
            error_sender,
            Arc::clone(&ui_context),
        )?);
        let ui_handle = if manual {
            None
        } else {
            let handle = Some(thread::spawn(move || ui.unwrap().run(tui)));
            ui = None;
            handle
        };
        Ok((
            Self {
                engine,
                current_map,
                next_map,
                shared_info,
                time_switch: true,
                ui_event_sender,
                ui_handle,
                error_receiver,
                ui_context,
            },
            ui,
        ))
    }

    fn new_simple(headless: bool, manual: bool) -> IoResult<(Self, Option<UiState>)> {
        Self::new_from_filter(
            |data| matches!(data.body_type, BodyType::Planet | BodyType::Star),
            headless,
            manual,
        )
    }

    fn new_moons(headless: bool, manual: bool) -> IoResult<(Self, Option<UiState>)> {
        Self::new_from_filter(
            |data| {
                matches!(
                    data.body_type,
                    BodyType::Planet | BodyType::Star | BodyType::Moon
                )
            },
            headless,
            manual,
        )
    }
    fn new_complete(headless: bool, manual: bool) -> IoResult<(Self, Option<UiState>)> {
        Self::new_from_filter(|_| true, headless, manual)
    }

    pub fn new_simple_client() -> IoResult<Self> {
        Self::new_simple(false, false).map(|a| a.0)
    }
    pub fn new_moons_client() -> IoResult<Self> {
        Self::new_moons(false, false).map(|a| a.0)
    }
    pub fn new_complete_client() -> IoResult<Self> {
        Self::new_moons(false, false).map(|a| a.0)
    }

    pub fn new_simple_testing() -> IoResult<(Self, UiState)> {
        Self::new_simple(true, true).map(|a| (a.0, a.1.unwrap()))
    }
    pub fn new_moons_testing() -> IoResult<(Self, UiState)> {
        Self::new_moons(true, true).map(|a| (a.0, a.1.unwrap()))
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut previous_time = Instant::now();
        let mut lag = Duration::ZERO;
        // eprintln!(
        //     "Starting app at {}",
        //     std::time::SystemTime::now()
        //         .duration_since(std::time::UNIX_EPOCH)
        //         .unwrap()
        //         .as_secs()
        // );
        loop {
            if let Ok(AppMessage::Quit) = self.handle_input() {
                self.ui_event_sender.send(UiEvent::Quit)?;
                if let Some(handle) = self.ui_handle.take() {
                    handle.join().unwrap();
                }
                break;
            }
            if let Ok(err) = self.error_receiver.try_recv() {
                if let Some(handle) = self.ui_handle.take() {
                    handle.join().unwrap();
                }
                return Err(err);
            }
            let current_time = Instant::now();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            if self.time_switch {
                lag += elapsed;
                while lag >= TIME_STEP {
                    self.engine.update();
                    self.copy_buffer();
                    lag -= TIME_STEP;
                }
            }
        }
        Ok(())
    }

    fn toggle_time_switch(&mut self) {
        self.time_switch = !self.time_switch
    }

    fn copy_buffer(&mut self) {
        self.current_map
            .lock()
            .unwrap()
            .clone_from(&*self.next_map.lock().unwrap());
    }

    pub fn get_current_screen(&self) -> AppScreen {
        self.ui_context.lock().unwrap().current_screen
    }
    pub fn get_explorer_mode(&self) -> ExplorerMode {
        self.ui_context.lock().unwrap().explorer_mode
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::algebra::inorm;

    use super::App;

    #[test]
    fn test_copy_buffers() {
        let (mut app, ui) = App::new_moons_testing().unwrap();
        app.engine.update();
        app.copy_buffer();

        let global = ui.global_map.lock().unwrap();
        let local = &app.engine.bodies;
        let moon = "lune".into();
        assert!(
            (inorm(global[&moon]) - inorm(local[&"terre".into()].position)).abs()
                <= inorm(local[&moon].position)
        )
    }
}
