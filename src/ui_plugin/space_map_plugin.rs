use bevy::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{
        canvas::{Canvas, Circle},
        Widget,
    },
};

use crate::engine_plugin::Position;

pub struct SpaceMapPlugin;

impl Plugin for SpaceMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpaceMap::default())
            .add_systems(Update, update_space_map);
    }
}

#[derive(Resource, Default)]
pub struct SpaceMap {
    circles: Vec<Circle>,
}

impl Widget for SpaceMap {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Canvas::default().paint(|ctx| {}).render(area, buf)
    }
}

fn update_space_map(map: ResMut<SpaceMap>, query: Query<&Position>) {}

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
            Block::bordered().title(Title::from("Space map".bold()).alignment(Alignment::Center)),
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
