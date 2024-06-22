use bevy::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Color,
    widgets::{
        block::Title,
        canvas::{Canvas, Circle},
        Block, Widget,
    },
};

use crate::{app::body_data::BodyType, engine_plugin::Position};

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
    offset: DVec2,
    focus_object: BodyID,
}

impl Widget for SpaceMap {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Canvas::default().paint(|ctx| {}).render(area, buf)
    }
}

fn update_space_map(mut map: ResMut<SpaceMap>, query: Query<&Position>) {
    let mut circles = Vec::new();
    map.circles = circles;
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
                let (x, refy) = (x as f64 * scale, y as f64 * scale);
                let data = self.shared_info.bodies.get(id);
                let color = match data.map(|body| body.body_type) {
                    None => Color::DarkGray,
                    Some(body_type) => match body_type {
                        _ if *id == self.selected_body_id_tree() => Color::Red,
                        BodyType::Star => Color::Yellow,
                        BodyType::Planet => Color::Blue,
                        _ => Color::DarkGray,
                    },
                };
                let radius = data.map_or(0., |d| d.radius * scale);
                ctx.draw(&Circle {
                    x,
                    y,
                    radius,
                    color,
                })
            }
        });
}
