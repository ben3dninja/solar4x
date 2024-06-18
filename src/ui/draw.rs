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

use crate::{app::body_data::BodyType, utils::ui::centered_rect};

use super::{AppScreen, ExplorerMode, UiState};

impl UiState {
    pub fn draw_ui(&mut self, f: &mut Frame) {
        let chunks =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).split(f.size());
        match self.get_explorer_mode() {
            ExplorerMode::Tree => self.draw_tree(f, chunks[0]),
            ExplorerMode::Search => self.draw_search(f, chunks[0]),
        }
        self.draw_canvas(f, chunks[1]);
        if matches!(self.get_current_screen(), AppScreen::Info) {
            self.draw_popup(f)
        }
    }

    fn draw_tree(&mut self, f: &mut Frame, rect: Rect) {
        let texts: Vec<Line<'_>> = self
            .tree_entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                self.shared_info.bodies.get(&entry.id).map(|body| {
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
                    vec![deepness_marker, Span::styled(body.name.clone(), style)].into()
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
                self.shared_info
                    .bodies
                    .get(entry)
                    .map(|body| body.name.clone())
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
        let max_dist = self.shared_info.get_max_distance() as f64;
        let (width, height) = (rect.width as f64, rect.height as f64);
        let min_dim = width.min(height);
        let scale = self.zoom_level * 0.9 * min_dim / max_dist;
        let positions = self.global_map.lock().unwrap();
        let (focusx, focusy) = positions
            .get(&self.focus_body)
            .map_or((0, 0), |pos| (pos.x, pos.y));
        let canvas = Canvas::default()
            .block(
                Block::bordered()
                    .title(Title::from("Space map".bold()).alignment(Alignment::Center)),
            )
            .x_bounds([-width / 2., width / 2.])
            .y_bounds([-height, height])
            .paint(move |ctx| {
                for (id, pos) in positions.iter() {
                    let (x, y) = (pos.x, pos.y);
                    let (x, y) = (x - self.offset.x - focusx, y - self.offset.y - focusy);
                    let (x, y) = (x as f64 * scale, y as f64 * scale);
                    let data = self.shared_info.bodies.get(id);
                    let color = match data.map(|body| body.body_type) {
                        None => Color::DarkGray,
                        Some(body_type) => match body_type {
                            _ if *id == self.selected_body_id_tree() => Color::Red,
                            BodyType::Star => Color::Yellow,
                            BodyType::Planet => Color::Blue,
                            _ => Color::Gray,
                        },
                    };
                    let radius = data.map_or(0., |d| d.radius * scale);
                    #[cfg(feature = "radius")]
                    if let Some(data) = data {
                        let radius = scale
                            * match data.body_type {
                                BodyType::Star => 20000000.,
                                BodyType::Planet => {
                                    if data.apoapsis < 800000000 {
                                        10000000.
                                    } else {
                                        50000000.
                                    }
                                }
                                _ => 500000.,
                            };
                    }
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
        let main_body = self
            .shared_info
            .bodies
            .get(&self.selected_body_id_tree())
            .unwrap();
        let popup_block = Block::default()
            .title(&main_body.name[..])
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        let area = centered_rect(25, 25, f.size());
        Clear.render(area, f.buffer_mut());
        let info = Paragraph::new(format!(
            "Body type: {}\n\
            N of orbiting bodies: {}\n\
            Radius: {} km\n\
            Revolution period: {} earth days",
            main_body.body_type,
            main_body.orbiting_bodies.len(),
            main_body.radius,
            main_body.revolution_period,
        ))
        .block(popup_block);
        f.render_widget(info, area);
    }
}
