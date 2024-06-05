use std::io::{Result, Stdout};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    bodies::{body_id::BodyID, BodySystem},
    ui::ui,
};

type Tui = Terminal<CrosstermBackend<Stdout>>;

const DEFAULT_BODY: &'static str = "soleil";

pub enum AppScreen {
    Main,
    Info,
}

pub struct App {
    pub current_screen: AppScreen,
    pub main_body: BodyID,
    pub bodies: BodySystem,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            current_screen: AppScreen::Main,
            main_body: BodyID::from(DEFAULT_BODY),
            bodies: BodySystem::simple_solar_system()?,
        })
    }
    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        loop {
            tui.draw(|frame| ui(frame, self))?;
            if let event::Event::Key(event) = event::read()? {
                if event.kind == KeyEventKind::Release {
                    continue;
                }
                match self.current_screen {
                    AppScreen::Main { .. } => match event.code {
                        KeyCode::Char('q') => break,
                        _ => (),
                    },
                    AppScreen::Info => todo!(),
                }
            }
        }
        Ok(())
    }
}
