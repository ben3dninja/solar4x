use std::{
    io::{Result, Stdout},
    time::Duration,
};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};

use crate::{
    bodies::{body::Body, body_id::BodyID, BodySystem},
    ui::ui,
};

type Tui = Terminal<CrosstermBackend<Stdout>>;

const DEFAULT_BODY: &str = "soleil";
// frame rate in fps
const FRAME_RATE: f64 = 60.;
const DEFAULT_SPEED: f64 = 10.;

pub enum AppScreen {
    Main,
    Info,
}

pub struct App {
    pub current_screen: AppScreen,
    pub main_body: BodyID,
    pub system: BodySystem,
    pub list_state: ListState,
    // 1 represents the level where all the system is seen,
    // higher values mean more zoom
    pub zoom_level: f64,
    // 1 represents 1 day / second
    pub speed: f64,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            current_screen: AppScreen::Main,
            main_body: BodyID::from(DEFAULT_BODY),
            system: BodySystem::simple_solar_system()?,
            list_state: ListState::default().with_selected(Some(1)),
            zoom_level: 1.,
            speed: DEFAULT_SPEED,
        })
    }
    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        for body in &mut self.system.bodies {
            body.set_time(20000.);
        }
        loop {
            tui.draw(|frame| ui(frame, self))?;
            if event::poll(Duration::from_secs_f64(1. / FRAME_RATE))? {
                if let event::Event::Key(event) = event::read()? {
                    if event.kind == KeyEventKind::Release {
                        continue;
                    }
                    match self.current_screen {
                        AppScreen::Main => match event.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Down => {
                                self.list_state.select(match self.list_state.selected() {
                                    Some(i) if i == self.system.number() - 1 => Some(i),
                                    Some(i) => Some(i + 1),
                                    None => Some(0),
                                })
                            }
                            KeyCode::Up => {
                                self.list_state.select(match self.list_state.selected() {
                                    Some(0) => Some(0),
                                    Some(i) => Some(i - 1),
                                    None => None,
                                })
                            }
                            KeyCode::Char('+') => {
                                self.zoom_level *= 1.5;
                            }
                            KeyCode::Char('-') => {
                                self.zoom_level /= 1.5;
                            }
                            KeyCode::Char('>') => {
                                self.speed *= 1.5;
                            }
                            KeyCode::Char('<') => {
                                self.speed /= 1.5;
                            }
                            _ => (),
                        },
                        AppScreen::Info => todo!(),
                    }
                }
            }
            for body in &mut self.system.bodies {
                body.set_time(body.time + self.speed / FRAME_RATE)
            }
        }
        Ok(())
    }

    pub fn selected_body(&self) -> &Body {
        &self.system.bodies[self.list_state.selected().unwrap_or_default()]
    }
}
