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
        Block, WidgetRef,
    },
};

use crate::{
    app::{body_data::BodyType, body_id::BodyID},
    core_plugin::{BodyInfo, EntityMapping, PrimaryBody},
    engine_plugin::{EllipticalOrbit, Position},
    utils::{
        algebra::project_onto_plane,
        ui::{Direction2, Direction4},
    },
};

use super::tree_plugin::{TreeState, TreeViewEvent};

const OFFSET_STEP: f64 = 1e8;

pub struct SpaceMapPlugin;

impl Plugin for SpaceMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpaceMapEvent>()
            .add_systems(PostStartup, initialize_space_map)
            .add_systems(
                Update,
                (
                    handle_space_map_events,
                    update_selected.run_if(resource_exists::<TreeState>),
                ),
            )
            .add_systems(PostUpdate, update_space_map);
    }
}

#[derive(Debug, Event)]
pub enum SpaceMapEvent {
    Zoom(Direction2),
    MapOffset(Direction4),
    MapOffsetReset,
    FocusBody,
    Autoscale,
}

#[derive(Debug, Resource, Clone)]
pub struct FocusBody(pub BodyID);

#[derive(Resource, Default, Debug)]
pub struct SpaceMap {
    circles: Vec<Circle>,
    offset: DVec2,
    focus_body: BodyID,
    zoom_level: f64,
    selected_body: Option<BodyID>,
    system_size: f64,
}

impl WidgetRef for SpaceMap {
    fn render_ref(&self, area: Rect, buf: &mut Buffer)
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
            .render_ref(area, buf)
    }
}

fn update_selected(
    mut map: ResMut<SpaceMap>,
    mut reader: EventReader<TreeViewEvent>,
    tree: Res<TreeState>,
) {
    for event in reader.read() {
        match event {
            TreeViewEvent::SelectTree(_) => {
                map.selected_body = Some(tree.selected_body_id());
            }
            _ => continue,
        }
    }
}

fn update_space_map(
    mut map: ResMut<SpaceMap>,
    mapping: Res<EntityMapping>,
    query: Query<(&Position, &BodyInfo)>,
) {
    let mut circles = Vec::new();
    let focus = map.focus_body;
    let (&Position(focus_pos), _) = query
        .get(mapping.id_mapping[&focus])
        .unwrap_or_else(|_| panic!("Could not find focus object {}", focus));
    for (&Position(pos), BodyInfo(data)) in query.iter() {
        let proj = project_onto_plane(pos - focus_pos, (DVec3::X, DVec3::Y));
        let color = match data.body_type {
            _ if Some(data.id) == map.selected_body => Color::Red,
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
    tree: Option<Res<TreeState>>,
) {
    let system_size = positions
        .iter()
        .map(|pos| pos.0.length())
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();
    let focus_body = primary.0;
    commands.insert_resource(SpaceMap {
        circles: Vec::new(),
        offset: DVec2::ZERO,
        focus_body,
        zoom_level: 1.,
        selected_body: tree.map(|tree| tree.selected_body_id()),
        system_size,
    });
    commands.insert_resource(FocusBody(focus_body));
}

fn handle_space_map_events(
    mut reader: EventReader<SpaceMapEvent>,
    mut space_map: ResMut<SpaceMap>,
    tree: Option<Res<TreeState>>,
    mapping: Res<EntityMapping>,
    mut focus_body: ResMut<FocusBody>,
    query: Query<&BodyInfo>,
) {
    use Direction2::*;
    use Direction4::*;
    use SpaceMapEvent::*;
    for event in reader.read() {
        match event {
            Zoom(d) => match d {
                Down => space_map.zoom_level /= 1.5,
                Up => space_map.zoom_level *= 1.5,
            },
            MapOffset(d) => {
                let zoom = space_map.zoom_level;
                space_map.offset += (match d {
                    Front | Right => 1.,
                    _ => -1.,
                } * OFFSET_STEP
                    / zoom)
                    * match d {
                        Front | Back => DVec2::Y,
                        _ => DVec2::X,
                    }
            }
            MapOffsetReset => space_map.offset = DVec2::ZERO,
            FocusBody => {
                if let Some(tree) = &tree {
                    focus_body.0 = tree.selected_body_id();
                    space_map.focus_body = focus_body.0;
                }
            }
            Autoscale => {
                let focus_data = &query
                    .get(mapping.id_mapping[&space_map.focus_body])
                    .unwrap()
                    .0;
                if let Some(max_dist) = focus_data
                    .orbiting_bodies
                    .iter()
                    .filter_map(|id| {
                        mapping
                            .id_mapping
                            .get(id)
                            .and_then(|&e| query.get(e).ok())
                            .map(|body| body.0.semimajor_axis)
                    })
                    .max()
                {
                    space_map.zoom_level = space_map.system_size / (max_dist as f64);
                }
            }
        }
    }
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
        ui_plugin::space_map_plugin::{update_space_map, FocusBody, SpaceMap},
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
        let earth = "terre".into();
        app.world.resource_mut::<FocusBody>().0 = earth;
        app.update();
        assert_eq!(app.world.resource::<FocusBody>().0, earth);
    }
}
