use std::io::{Result, Stdout};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{body::body_id::BodyID, ui::ui};

type Tui = Terminal<CrosstermBackend<Stdout>>;

const DEFAULT_BODY: &'static str = "soleil";

enum AppScreen {
    Main { body_id: BodyID },
    Info,
}

impl Default for AppScreen {
    fn default() -> Self {
        AppScreen::Main {
            body_id: DEFAULT_BODY.into(),
        }
    }
}

#[derive(Default)]
pub struct App {
    current_screen: AppScreen,
}

impl App {
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
