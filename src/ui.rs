use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{block::Title, Block, List},
    Frame,
};

use crate::app::App;

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).split(f.size());

    //     let body_name = if let Some(body) = app.bodies.get_body_data(&app.main_body) {
    //         &body.data.name[..]
    //     } else {
    //         "Unknown body"
    //     };

    //     let list = app
    //         .bodies
    //         .get_body_names()
    //         .into_iter()
    //         .skip(1)
    //         .collect::<List>()
    //         .block(Block::bordered().title(Title::from(body_name.bold()).alignment(Alignment::Center)))
    //         .highlight_style(Style::default().red());
    let names = app.bodies.get_body_names();
    let texts: Vec<Text> = vec![Text::styled(names[0], Style::default().bold())]
        .into_iter()
        .chain(
            names
                .into_iter()
                .skip(1)
                .map(|s| Text::styled(s, Style::default())),
        )
        .collect();
    let list = List::new(texts)
        .block(
            Block::bordered()
                .title(Title::from("Celestial Bodies".bold()).alignment(Alignment::Center)),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, chunks[0], &mut app.list_state);
}
