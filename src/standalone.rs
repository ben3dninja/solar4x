use std::{
    error::Error,
    time::{Duration, Instant},
};

use crate::{
    app::{body_data::BodyType, App, AppMessage, GuiApp, TIME_STEP},
    keyboard::Keymap,
    ui::{events::UiEvent, UiState},
};

pub struct Standalone {
    pub app: GuiApp,
}

impl Standalone {
    pub fn new_testing(min_body_type: BodyType) -> std::io::Result<(Self, UiState)> {
        let (app, ui) = GuiApp::new_smallest_type(min_body_type, true)?;
        Ok((Standalone { app }, ui.unwrap()))
    }
    pub fn new(min_body_type: BodyType) -> std::io::Result<Self> {
        let (app, _) = GuiApp::new_smallest_type(min_body_type, false)?;
        Ok(Standalone { app })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut previous_time = Instant::now();
        let mut lag = Duration::ZERO;
        loop {
            let app = &mut self.app;
            if let Ok(AppMessage::Quit) = app.handle_input() {
                app.ui_event_sender.send(UiEvent::Quit)?;
                if let Some(handle) = app.ui_handle.take() {
                    handle.join().unwrap();
                }
                break;
            }
            if let Ok(err) = app.error_receiver.try_recv() {
                if let Some(handle) = app.ui_handle.take() {
                    handle.join().unwrap();
                }
                return Err(err);
            }
            let current_time = Instant::now();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            if app.core.time_switch {
                lag += elapsed;
                while lag >= TIME_STEP {
                    app.core.engine.update();
                    app.core.copy_buffer();
                    lag -= TIME_STEP;
                }
            }
        }
        Ok(())
    }

    pub fn core_mut(&mut self) -> &mut App {
        &mut self.app.core
    }

    pub fn core(&self) -> &App {
        &self.app.core
    }

    pub fn set_keymap(&mut self, keymap: Keymap) {
        self.app.keymap = keymap;
    }

    pub fn with_keymap(mut self, keymap: Keymap) -> Self {
        self.set_keymap(keymap);
        self
    }
}
