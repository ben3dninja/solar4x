use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    widgets::{
        block::Title,
        canvas::{Canvas, Circle},
        Block, Widget,
    },
};

use crate::{
    app::{body_data::BodyType, body_id::BodyID},
    core_plugin::{BodyInfo, EntityMapping, PrimaryBody},
    engine_plugin::{EllipticalOrbit, Position},
    utils::algebra::project_onto_plane,
};

pub struct SpaceMapPlugin;

impl Plugin for SpaceMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, initialize_space_map)
            .add_systems(PostUpdate, update_space_map);
    }
}

#[derive(Resource, Default, Debug)]
pub struct SpaceMap {
    circles: Vec<Circle>,
    pub offset: DVec2,
    pub focus_object: BodyID,
    pub zoom_level: f64,
    pub selected_body: BodyID,
    pub system_size: f64,
}

impl Widget for SpaceMap {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (width, height) = (area.width as f64, area.height as f64);
        let scale = self.system_size / (width.min(height) * self.zoom_level);
        let (width, height) = (width * scale, height * scale);
        Canvas::default()
            .block(
                Block::bordered()
                    .title(Title::from("Space map".bold()).alignment(Alignment::Center)),
            )
            .x_bounds([-width / 2., width / 2.])
            .y_bounds([-height, height])
            .paint(|ctx| {
                for circle in &self.circles {
                    ctx.draw(circle);
                }
            })
            .render(area, buf)
    }
}

fn update_space_map(
    mut map: ResMut<SpaceMap>,
    mapping: Res<EntityMapping>,
    query: Query<(&Position, &BodyInfo)>,
) {
    let mut circles = Vec::new();
    let focus = map.focus_object;
    let (&Position(focus_pos), _) = query
        .get(mapping.id_mapping[&focus])
        .unwrap_or_else(|_| panic!("Could not find focus object {}", focus));
    for (&Position(pos), BodyInfo(data)) in query.iter() {
        let proj = project_onto_plane(pos - focus_pos, (DVec3::X, DVec3::Y));
        let color = match data.body_type {
            _ if data.id == map.selected_body => Color::Red,
            BodyType::Star => Color::Yellow,
            BodyType::Planet => Color::Blue,
            _ => Color::DarkGray,
        };
        let radius = data.radius;
        circles.push(Circle {
            x: proj.x,
            y: proj.y,
            radius,
            color,
        });
    }
    map.circles = circles;
}

fn initialize_space_map(
    mut commands: Commands,
    positions: Query<&Position, With<EllipticalOrbit>>,
    primary: Res<PrimaryBody>,
) {
    let system_size = positions
        .iter()
        .map(|pos| pos.0.length())
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();
    commands.insert_resource(SpaceMap {
        circles: Vec::new(),
        offset: DVec2::ZERO,
        focus_object: primary.0,
        zoom_level: 1.,
        selected_body: primary.0,
        system_size,
    });
}

#[cfg(test)]
mod tests {
    use bevy::{
        app::{App, Update},
        ecs::schedule::IntoSystemConfigs,
    };

    use crate::{
        app::body_data::BodyType,
        core_plugin::CorePlugin,
        engine_plugin::{update_global, update_local, update_time, EnginePlugin},
        ui_plugin::space_map_plugin::{update_space_map, SpaceMap},
    };

    use super::SpaceMapPlugin;

    #[test]
    fn test_update_space_map() {
        let mut app = App::new();
        app.add_plugins((
            CorePlugin {
                smallest_body_type: BodyType::Planet,
            },
            EnginePlugin,
            SpaceMapPlugin,
        ))
        .add_systems(
            Update,
            (update_time, update_local, update_global, update_space_map).chain(),
        );

        app.update();
        let map = app.world.get_resource::<SpaceMap>().unwrap();
        assert_eq!(map.circles.len(), 9);
        dbg!(map);
        assert!(4459753056. < map.system_size);
        assert!(map.system_size < 4537039826.);
    }
}
