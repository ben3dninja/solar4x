use std::rc::Rc;

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
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

impl App {
    pub fn draw_ui(&mut self, f: &mut Frame) {
        let chunks =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).split(f.size());
        self.draw_explorer(f, chunks[0]);
        self.draw_canvas(f, chunks[1]);
        if matches!(self.current_screen, AppScreen::Info) {
            self.draw_popup(f)
        }
    }

    fn draw_explorer(&mut self, f: &mut Frame, rect: Rect) {
        let names: Vec<_> = self
            .listed_bodies
            .iter()
            .filter_map(|entry| {
                self.system
                    .borrow()
                    .bodies
                    .get(&entry.id)
                    .map(|body| body.info.name.clone())
            })
            .collect();
        let texts: Vec<Text> = vec![Text::styled(names[0].clone(), Style::default().bold())]
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
        f.render_stateful_widget(list, rect, &mut self.list_state);
    }

    fn draw_canvas(&self, f: &mut Frame, rect: Rect) {
        let max_dist = self.system.borrow().get_max_distance() as f64;
        let (width, height) = (rect.width as f64, rect.height as f64);
        let min_dim = width.min(height);
        let scale = self.zoom_level * 0.9 * min_dim / max_dist;

        let canvas = Canvas::default()
            .block(
                Block::bordered()
                    .title(Title::from("Space map".bold()).alignment(Alignment::Center)),
            )
            .x_bounds([-width / 2., width / 2.])
            .y_bounds([-height, height])
            .paint(move |ctx| {
                for body in self.system.borrow().bodies.values() {
                    let (x, y, _) = body.get_absolute_xyz(Rc::clone(&self.system));
                    let (x, y) = (x - self.offset.x, y - self.offset.y);
                    let (x, y) = (x as f64 * scale, y as f64 * scale);
                    let color = match body.info.body_type {
                        _ if body.id == self.selected_body_id() => Color::White,
                        BodyType::Star => Color::Yellow,
                        BodyType::Planet => {
                            if body.info.apoapsis < 800000000 {
                                Color::Blue
                            } else {
                                Color::Red
                            }
                        }
                        _ => Color::Gray,
                    };
                    let radius = body.info.radius * scale;
                    #[cfg(feature = "radius")]
                    let radius = scale
                        * match body.info.body_type {
                            BodyType::Star => 20000000.,
                            BodyType::Planet => {
                                if body.info.apoapsis < 800000000 {
                                    10000000.
                                } else {
                                    50000000.
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
        f.render_widget(canvas, rect);
    }

    fn draw_popup(&self, f: &mut Frame) {
        let system = self.system.borrow();
        let main_body = system.bodies.get(&self.selected_body_id()).unwrap();
        let popup_block = Block::default()
            .title(&main_body.info.name[..])
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        let area = centered_rect(25, 25, f.size());
        Clear.render(area, f.buffer_mut());
        let info = Paragraph::new(format!(
            "Body type: {}\n\
            N of orbiting bodies: {}\n\
            Radius: {} km\n\
            Revolution period: {} earth days",
            main_body.info.body_type,
            main_body.orbiting_bodies.len(),
            main_body.info.radius,
            main_body.orbit.revolution_period,
        ))
        .block(popup_block);
        f.render_widget(info, area);
    }
}
