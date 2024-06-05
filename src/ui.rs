use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Stylize,
    widgets::{block::Title, Block, Borders},
    Frame,
};

use crate::app::App;

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).split(f.size());

    let body_name = if let Some(body) = app.bodies.get_body_data(&app.main_body) {
        &body.data.name[..]
    } else {
        "Unknown body"
    };
    let left_block = Block::default()
        .title(Title::from(body_name.bold()).alignment(Alignment::Center))
        .borders(Borders::ALL);
    f.render_widget(left_block, chunks[0])
}
