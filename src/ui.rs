use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{
        block::Title,
        canvas::{Canvas, Circle},
        Block, Borders, Clear, List, Paragraph, Widget,
    },
    Frame,
};

use crate::{
    app::{App, AppScreen},
    bodies::body_data::BodyType,
    utils::ui::centered_rect,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).split(f.size());
    let names = app.system.get_body_names();
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
    let max_dist = app.system.get_max_distance() as f64;
    let (width, height) = (chunks[1].width as f64, chunks[1].height as f64);
    let min_dim = width.min(height);
    let scale = app.zoom_level * 0.9 * min_dim / max_dist;
    for body in &mut app.system.bodies {
        body.update_xyz();
    }
    let canvas = Canvas::default()
        .block(
            Block::bordered().title(Title::from("Space map".bold()).alignment(Alignment::Center)),
        )
        .x_bounds([-width / 2., width / 2.])
        .y_bounds([-height, height])
        .paint(|ctx| {
            for body in &app.system.bodies {
                let (x, y, _) = body.get_raw_xyz();
                let (x, y) = (x as f64 * scale, y as f64 * scale);
                let color = match body.data.body_type {
                    _ if body == app.selected_body() => Color::White,
                    BodyType::Star => Color::Yellow,
                    BodyType::Planet => {
                        if body.data.apoapsis < 800000000 {
                            Color::Blue
                        } else {
                            Color::Red
                        }
                    }
                    _ => Color::Gray,
                };
                let radius = body.data.radius * scale;
                #[cfg(feature = "radius")]
                let radius = scale
                    * match body.data.body_type {
                        BodyType::Star => 20000000.,
                        BodyType::Planet => {
                            if body.data.apoapsis < 800000000 {
                                10000000.
                            } else {
                                100000000.
                            }
                        }
                        _ => 500000.,
                    };
                ctx.draw(&Circle {
                    x,
                    y,
                    radius,
                    color,
                })
            }
        });
    f.render_widget(canvas, chunks[1]);
    if matches!(app.current_screen, AppScreen::Info) {
        let data = &app.selected_body().data;
        let popup_block = Block::default()
            .title(&data.name[..])
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        let area = centered_rect(25, 25, f.size());
        Clear.render(area, f.buffer_mut());
        let info = Paragraph::new(format!(
            "Body type: {}\n\
            N of orbiting bodies: {}\n\
            Radius: {} km\n\
            Revolution period: {} earth days",
            data.body_type,
            data.orbiting_bodies.len(),
            data.radius,
            data.revolution_period,
        ))
        .block(popup_block);
        f.render_widget(info, area);
    }
}
