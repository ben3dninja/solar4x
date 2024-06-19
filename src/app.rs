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
    keyboard::Keymap,
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
}

pub enum AppMessage {
    Quit,
    Idle,
}

pub enum ApplicationSide {
    Client,
    Server,
}

impl App {
    pub fn new_from_filter(f: impl FnMut(&BodyData) -> bool) -> IoResult<Self> {
        let bodies = read_main_bodies()?.into_iter().filter(f);
        Self::new_from_bodies(bodies)
    }

    pub fn new_from_bodies(bodies: impl IntoIterator<Item = BodyData>) -> IoResult<Self> {
        let current_map = Arc::new(Mutex::new(GlobalMap::new()));
        let next_map = Arc::new(Mutex::new(GlobalMap::new()));
        let shared_info = Arc::new(
            SystemInfo::new(bodies)
                .ok_or(std::io::Error::other("no primary body found in data"))?,
        );
        let engine = Engine::new_from_data(Arc::clone(&next_map), Arc::clone(&shared_info));
        Ok(Self {
            engine,
            current_map,
            next_map,
            shared_info,
            time_switch: true,
        })
    }

    pub fn toggle_time_switch(&mut self) {
        self.time_switch = !self.time_switch
    }

    pub fn copy_buffer(&mut self) {
        self.current_map
            .lock()
            .unwrap()
            .clone_from(&*self.next_map.lock().unwrap());
    }
}

pub struct GuiApp {
    pub core: App,
    pub ui_event_sender: Sender<UiEvent>,
    pub ui_handle: Option<JoinHandle<()>>,
    pub ui_context: Arc<Mutex<UiContext>>,
    pub error_receiver: Receiver<AppError>,
    pub keymap: Keymap,
}

impl GuiApp {
    pub fn new_from_filter(
        f: impl FnMut(&BodyData) -> bool,
        testing: bool,
    ) -> IoResult<(Self, Option<UiState>)> {
        let bodies = read_main_bodies()?.into_iter().filter(f);
        Self::new_from_bodies(bodies, testing)
    }

    pub fn new_from_bodies(
        bodies: impl IntoIterator<Item = BodyData>,
        testing: bool,
    ) -> IoResult<(Self, Option<UiState>)> {
        let core = App::new_from_bodies(bodies)?;
        let (ui_event_sender, ui_event_receiver) = mpsc::channel();
        let (error_sender, error_receiver) = mpsc::channel();
        let ui_context = Arc::new(Mutex::new(UiContext::default()));
        let mut ui = Some(UiState::new(
            Arc::clone(&core.shared_info),
            Arc::clone(&core.current_map),
            ui_event_receiver,
            error_sender,
            Arc::clone(&ui_context),
        )?);
        let ui_handle = if testing {
            None
        } else {
            let tui = Some(UiState::setup_tui()?);
            let handle = Some(thread::spawn(move || ui.unwrap().run(tui)));
            ui = None;
            handle
        };
        Ok((
            Self {
                core,
                ui_event_sender,
                ui_handle,
                ui_context,
                keymap: Keymap::default(),
                error_receiver,
            },
            ui,
        ))
    }

    pub fn new_smallest_type(
        smallest_body_type: BodyType,
        testing: bool,
    ) -> IoResult<(Self, Option<UiState>)> {
        Self::new_from_filter(|data| data.body_type <= smallest_body_type, testing)
    }

    pub fn get_current_screen(&self) -> AppScreen {
        self.ui_context.lock().unwrap().current_screen
    }
    pub fn get_explorer_mode(&self) -> ExplorerMode {
        self.ui_context.lock().unwrap().explorer_mode
    }
    pub fn run(&mut self, fixed_update: impl Fn(&mut Self)) -> Result<(), Box<dyn Error>> {
        let mut previous_time = Instant::now();
        let mut lag = Duration::ZERO;
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
            if self.core.time_switch {
                lag += elapsed;
                while lag >= TIME_STEP {
                    fixed_update(self);
                    lag -= TIME_STEP;
                }
            }
        }
        Ok(())
    }
    pub fn set_keymap(&mut self, keymap: Keymap) {
        self.keymap = keymap;
    }

    pub fn with_keymap(mut self, keymap: Keymap) -> Self {
        self.set_keymap(keymap);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{app::body_data::BodyType, standalone::Standalone, utils::algebra::inorm};

    #[test]
    fn test_copy_buffers() {
        let (mut app, ui) = Standalone::new_testing(BodyType::Moon).unwrap();
        app.core_mut().engine.update();
        app.core_mut().copy_buffer();

        let global = ui.global_map.lock().unwrap();
        let local = &app.core().engine.bodies;
        let moon = "lune".into();
        assert!(
            (inorm(global[&moon]) - inorm(local[&"terre".into()].position)).abs()
                <= inorm(local[&moon].position)
        )
    }
}
