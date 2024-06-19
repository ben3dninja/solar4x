use std::error::Error;

use crate::{
    app::{body_data::BodyType, App, GuiApp},
    keyboard::Keymap,
    ui::UiState,
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
        self.app.run(|app| {
            app.core.engine.update();
            app.core.copy_buffer();
        })
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
