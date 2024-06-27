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
    bodies::{body_data::BodyType, body_id::BodyID},
    core_plugin::{AppState, BodyInfo, EntityMapping, GameSet, PrimaryBody},
    engine_plugin::{EllipticalOrbit, Position},
    utils::{
        algebra::project_onto_plane,
        ui::{Direction2, Direction4},
    },
};

use super::{
    search_plugin::SearchViewEvent,
    tree_plugin::{TreeState, TreeViewEvent},
    InitializeUiSet,
};

const OFFSET_STEP: f64 = 1e8;

pub struct SpaceMapPlugin;

impl Plugin for SpaceMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpaceMapEvent>()
            .add_systems(
                OnEnter(AppState::Game),
                initialize_space_map.in_set(InitializeUiSet),
            )
            .add_systems(
                Update,
                (
                    handle_space_map_events,
                    // update_selected.run_if(resource_exists::<TreeState>),
                    update_space_map,
                )
                    .in_set(GameSet)
                    .chain(),
            );
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

#[derive(Component)]
pub struct FocusBody;

#[derive(Resource, Default, Debug)]
pub struct SpaceMap {
    circles: Vec<Circle>,
    pub offset: DVec2,
    pub zoom_level: f64,
    pub system_size: f64,
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

// fn update_selected(
//     mut map: ResMut<SpaceMap>,
//     mut reader: EventReader<TreeViewEvent>,
//     mut search_reader: EventReader<SearchViewEvent>,
//     tree: Res<TreeState>,
// ) {
//     for event in reader.read() {
//         match event {
//             TreeViewEvent::SelectTree(_) => {
//                 map.selected_body = Some(tree.selected_body_id());
//             }
//             _ => continue,
//         }
//     }

//     for event in search_reader.read() {
//         match event {
//             SearchViewEvent::ValidateSearch => {
//                 map.selected_body = Some(tree.selected_body_id());
//             }
//             _ => continue,
//         }
//     }
// }

fn update_space_map(
    mut map: ResMut<SpaceMap>,
    query: Query<(&Position, &BodyInfo)>,
    tree: Option<Res<TreeState>>,
    focus_pos: Query<&Position, With<FocusBody>>,
) {
    let mut circles = Vec::new();
    let &Position(focus_pos) = focus_pos.single();
    let selected = tree.as_ref().map(|t| t.selected_body_id());
    for (&Position(pos), BodyInfo(data)) in query.iter() {
        let proj = project_onto_plane(pos - focus_pos, (DVec3::X, DVec3::Y)) - map.offset;
        let color = match data.body_type {
            _ if Some(data.id) == selected => Color::Red,
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

pub fn initialize_space_map(
    mut commands: Commands,
    positions: Query<&Position, With<EllipticalOrbit>>,
    primary: Query<Entity, With<PrimaryBody>>,
) {
    let system_size = positions
        .iter()
        .map(|pos| pos.0.length())
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();
    commands.insert_resource(SpaceMap {
        circles: Vec::new(),
        offset: DVec2::ZERO,
        zoom_level: 1.,
        system_size,
    });
    commands.entity(primary.single()).insert(FocusBody);
}

fn handle_space_map_events(
    mut commands: Commands,
    mut reader: EventReader<SpaceMapEvent>,
    mut space_map: ResMut<SpaceMap>,
    tree: Option<Res<TreeState>>,
    mapping: Res<EntityMapping>,
    focus_body: Query<(Entity, &BodyInfo), With<FocusBody>>,
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
                    commands
                        .entity(focus_body.single().0)
                        .remove::<self::FocusBody>();
                    if let Some(entity) = mapping.id_mapping.get(&tree.selected_body_id()) {
                        commands.entity(*entity).insert(self::FocusBody);
                    }
                }
            }
            Autoscale => {
                let focus_data = &focus_body.single().1 .0;
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
        ecs::{query::With, schedule::IntoSystemConfigs},
    };

    use crate::{
        bodies::body_data::BodyType,
        core_plugin::{BodiesConfig, BodyInfo, GameSet},
        engine_plugin::{update_global, update_local, update_time, EnginePlugin},
        standalone_plugin::StandalonePlugin,
        tui_plugin::{
            space_map_plugin::{
                handle_space_map_events, update_space_map, FocusBody, SpaceMap, SpaceMapEvent,
            },
            tree_plugin::{TreePlugin, TreeState},
        },
    };

    use super::SpaceMapPlugin;

    #[test]
    fn test_update_space_map() {
        let mut app = App::new();
        app.add_plugins((
            StandalonePlugin(BodiesConfig::SmallestBodyType(BodyType::Planet)),
            EnginePlugin,
            SpaceMapPlugin,
        ))
        .add_systems(
            Update,
            (update_time, update_local, update_global, update_space_map)
                .in_set(GameSet)
                .chain(),
        );
        app.update();
        let map = app.world.get_resource::<SpaceMap>().unwrap();
        assert_eq!(map.circles.len(), 9);
        dbg!(map);
        assert!(4459753056. < map.system_size);
        assert!(map.system_size < 4537039826.);
    }

    #[test]
    fn test_change_focus_body() {
        let mut app = App::new();
        app.add_plugins((
            StandalonePlugin(BodiesConfig::SmallestBodyType(BodyType::Planet)),
            EnginePlugin,
            TreePlugin,
            SpaceMapPlugin,
        ))
        .add_systems(Update, handle_space_map_events.in_set(GameSet));
        app.update();
        let earth = "terre".into();

        app.world.resource_mut::<TreeState>().select_body(earth);
        app.world.send_event(SpaceMapEvent::FocusBody);
        app.update();
        let world = &mut app.world;
        assert_eq!(
            world
                .query_filtered::<&BodyInfo, With<FocusBody>>()
                .single(world)
                .0
                .id,
            earth
        );
    }
}
