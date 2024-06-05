use std::io::{stdout, Result};

use app::App;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
mod app;
mod bodies;
mod ui;
mod utils;
fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut tui = Terminal::new(CrosstermBackend::new(stdout()))?;

    let res = App::new()?.run(&mut tui);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    res
}
