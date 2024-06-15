use std::{error::Error, io::stdout};

use app::App;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
mod app;
mod bodies;
mod engine;
mod ui;
mod utils;
fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(not(feature = "asteroids"))]
    let res = App::new_moons(false)?.run();
    #[cfg(feature = "asteroids")]
    let res = App::new_complete(false)?.run();

    res
}
