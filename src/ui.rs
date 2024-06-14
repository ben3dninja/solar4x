use std::rc::Rc;

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        block::Title,
        canvas::{Canvas, Circle},
        Block, Borders, Clear, List, Paragraph, Widget,
    },
    Frame,
};

use crate::{
    app::{App, AppScreen, ExplorerMode},
    bodies::body_data::BodyType,
    utils::ui::centered_rect,
};

impl App {
    pub fn draw_ui(&mut self, f: &mut Frame) {
        let chunks =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).split(f.size());
        match self.explorer_mode {
            ExplorerMode::Tree => self.draw_tree(f, chunks[0]),
            ExplorerMode::Search => self.draw_search(f, chunks[0]),
        }
        self.draw_canvas(f, chunks[1]);
        if matches!(self.current_screen, AppScreen::Info) {
            self.draw_popup(f)
        }
    }

    fn draw_tree(&mut self, f: &mut Frame, rect: Rect) {
        let texts: Vec<Line<'_>> = self
            .tree_entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                self.system.borrow().bodies.get(&entry.id).map(|body| {
                    let style = if body.id == self.focus_body {
                        Style::default().bold()
                    } else {
                        Style::default()
                    };
                    let deepness_marker = Span::from(if entry.deepness == 0 {
                        String::new()
                    } else {
                        "│ ".repeat(entry.deepness.saturating_sub(1))
                            + if self.entry_is_last_child(index).unwrap() {
                                "└─"
                            } else {
                                "├─"
                            }
                    });
                    vec![deepness_marker, Span::styled(body.info.name.clone(), style)].into()
                })
            })
            .collect();
        let list = List::new(texts)
            .block(
                Block::bordered()
                    .title(Title::from("Tree view".bold()).alignment(Alignment::Center)),
            )
            .highlight_symbol("> ");
        f.render_stateful_widget(list, rect, &mut self.tree_state);
    }

    fn draw_search(&mut self, f: &mut Frame, rect: Rect) {
        let names: Vec<_> = self
            .search_entries
            .iter()
            .filter_map(|entry| {
                self.system
                    .borrow()
                    .bodies
                    .get(entry)
                    .map(|body| body.info.name.clone())
            })
            .collect();
        let texts: Vec<Text> = names
            .into_iter()
            .map(|s| Text::styled(s, Style::default()))
            .collect();
        let search_bar = Paragraph::new(&self.search_input[..]).block(Block::bordered());
        let list = List::new(texts)
            .block(
                Block::bordered()
                    .title(Title::from("Search view".bold()).alignment(Alignment::Center)),
            )
            .highlight_symbol("> ");
        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(rect);
        f.render_widget(search_bar, chunks[0]);
        f.render_stateful_widget(list, chunks[1], &mut self.search_state);
    }

    fn draw_canvas(&self, f: &mut Frame, rect: Rect) {
        let max_dist = self.system.borrow().get_max_distance() as f64;
        let (width, height) = (rect.width as f64, rect.height as f64);
        let min_dim = width.min(height);
        let scale = self.zoom_level * 0.9 * min_dim / max_dist;
        let system = &self.system;
        let (focusx, focusy, _) =
            system.borrow().bodies[&self.focus_body].get_absolute_xyz(Rc::clone(system));
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
                    let (x, y) = (x - self.offset.x - focusx, y - self.offset.y - focusy);
                    let (x, y) = (x as f64 * scale, y as f64 * scale);
                    let color = match body.info.body_type {
                        _ if body.id == self.selected_body_id_tree() => Color::White,
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
        let main_body = system.bodies.get(&self.selected_body_id_tree()).unwrap();
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
