use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{
        block::Title,
        canvas::{Canvas, Circle},
        Block, List,
    },
    Frame,
};

use crate::{app::App, bodies::body_data::BodyType};

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
        // dbg!(body.time, body.update_state.clone());
    }
    let canvas = Canvas::default()
        .block(Block::bordered().title("Space map"))
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
                // let (radius, mut color) = match body.data.body_type {
                //     BodyType::Star => (min_dim / 30., Color::Yellow),
                //     BodyType::Planet => (
                //         min_dim / 70.,
                //         if body.data.apoapsis < 800000000 {
                //             Color::Blue
                //         } else {
                //             Color::Red
                //         },
                //     ),
                //     _ => (min_dim / 150., Color::Gray),
                // };
                // if body == app.selected_body() {
                //     color = Color::White
                // }
                ctx.draw(&Circle {
                    x,
                    y,
                    radius,
                    color,
                })
            }
        });
    f.render_widget(canvas, chunks[1]);
}
