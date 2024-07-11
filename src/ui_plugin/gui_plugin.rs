use bevy::{
    a11y::AccessibilityPlugin,
    color::palettes::css::{BLACK, DARK_GRAY, GOLD, TEAL},
    core_pipeline::CorePipelinePlugin,
    input::InputPlugin,
    math::DVec3,
    prelude::*,
    render::{camera::ScalingMode, pipelined_rendering::PipelinedRenderingPlugin, RenderPlugin},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle, SpritePlugin},
    text::TextPlugin,
    ui::UiPlugin,
    winit::{WakeUp, WinitPlugin},
};

use crate::{
    bodies::body_data::BodyType,
    core_plugin::{BodyInfo, LoadingState},
    engine_plugin::Position,
    utils::algebra::project_onto_plane,
};

use super::{AppScreen, UiInit};

const MAX_WIDTH: f32 = 1000.;
const MIN_RADIUS: f32 = 0.01;
pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TransformPlugin,
            InputPlugin,
            WindowPlugin::default(),
            AccessibilityPlugin,
            AssetPlugin::default(),
            WinitPlugin::<WakeUp>::default(),
            RenderPlugin::default(),
            ImagePlugin::default(),
            PipelinedRenderingPlugin,
            CorePipelinePlugin,
            SpritePlugin,
            TextPlugin,
            UiPlugin,
        ))
        .insert_resource(ClearColor(Color::Srgba(BLACK)))
        // .insert_resource(ShootingState::Idle)
        .add_systems(Startup, gui_setup)
        .add_systems(
            OnEnter(LoadingState::Loaded),
            (insert_display_components, update_transform)
                .chain()
                .after(UiInit),
        )
        .add_systems(
            FixedPreUpdate,
            (update_transform, update_camera_pos).chain(),
        );
    }
}

fn gui_setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(MAX_WIDTH);
    commands.spawn(cam);
    let colors = Colors {
        stars: materials.add(Color::Srgba(GOLD)),
        planets: materials.add(Color::Srgba(TEAL)),
        other: materials.add(Color::Srgba(DARK_GRAY)),
    };
    commands.insert_resource(colors);
}

#[derive(Resource)]
pub struct Colors {
    stars: Handle<ColorMaterial>,
    planets: Handle<ColorMaterial>,
    other: Handle<ColorMaterial>,
}

fn insert_display_components(
    mut commands: Commands,
    query: Query<(Entity, &BodyInfo)>,
    mut meshes: ResMut<Assets<Mesh>>,
    colors: Res<Colors>,
    screen: Res<AppScreen>,
) {
    if let AppScreen::Explorer(ctx) = screen.as_ref() {
        let scale = MAX_WIDTH as f64 / ctx.space_map.system_size;
        query.iter().for_each(|(e, BodyInfo(data))| {
            let (material, z) = match data.body_type {
                BodyType::Star => (colors.stars.clone(), 0.),
                BodyType::Planet => (colors.planets.clone(), -1.),
                _ => (colors.other.clone(), -2.),
            };
            commands.entity(e).insert(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle {
                    radius: MIN_RADIUS.max((data.radius * scale) as f32),
                })),
                material,
                transform: Transform::from_xyz(0., 0., z),
                ..default()
            });
        });
    }
}

fn update_camera_pos(
    mut cam: Query<(&mut Transform, &mut OrthographicProjection)>,
    screen: Res<AppScreen>,
    positions: Query<&Position>,
) {
    if let AppScreen::Explorer(ctx) = screen.as_ref() {
        let space_map = &ctx.space_map;
        let scale = MAX_WIDTH as f64 / space_map.system_size;
        let (mut cam_pos, mut proj) = cam.single_mut();
        let focus_pos = positions.get(ctx.focus_body).unwrap().0;
        cam_pos.translation = ((focus_pos
            + DVec3::new(space_map.offset_amount.x, space_map.offset_amount.y, 0.))
            * scale)
            .as_vec3();
        proj.scale = (1. / space_map.zoom_level) as f32;
    }
}

fn update_transform(screen: Res<AppScreen>, mut query: Query<(&mut Transform, &Position)>) {
    if let AppScreen::Explorer(ctx) = screen.as_ref() {
        let scale = MAX_WIDTH as f64 / ctx.space_map.system_size;
        for (mut transform, Position(pos)) in query.iter_mut() {
            let (x, y) = (project_onto_plane(*pos, (DVec3::X, DVec3::Y)) * scale)
                .as_vec2()
                .into();
            transform.translation.x = x;
            transform.translation.y = y;
        }
    }
}

// #[derive(Resource)]
// enum ShootingState {
//     Idle,
//     Loading { launch_mouse_position: Vec2 },
// }

// fn shoot(
//     mut commands: Commands,
//     mut shooting_state: ResMut<ShootingState>,
//     mut buttons: EventReader<MouseButtonInput>,
//     window: Query<&Window, With<PrimaryWindow>>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     colors: Res<Colors>,
//     camera_query: Query<(&Camera, &GlobalTransform)>,
//     space_map: Res<SpaceMap>,
//     focus: Query<&Velocity, With<FocusBody>>,
//     time: Res<Time>,
//     game_speed: Res<GameSpeed>,
// ) {
//     if let Some(mouse_position) = window.single().cursor_position() {
//         let &Velocity(focus_speed) = focus.single();
//         let (camera, camera_transform) = camera_query.single();
//         let scale = MAX_WIDTH as f64 / space_map.system_size;
//         for event in buttons.read() {
//             match event.state {
//                 ButtonState::Pressed => {
//                     *shooting_state = ShootingState::Loading {
//                         launch_mouse_position: mouse_position,
//                     }
//                 }
//                 ButtonState::Released => {
//                     if let ShootingState::Loading {
//                         launch_mouse_position,
//                     } = *shooting_state
//                     {
//                         if let Some(shooting_transform) =
//                             camera.viewport_to_world_2d(camera_transform, launch_mouse_position)
//                         {
//                             if let Some(release_transform) =
//                                 camera.viewport_to_world_2d(camera_transform, mouse_position)
//                             {
//                                 let d = (shooting_transform - release_transform).as_dvec2();
//                                 let speed = DVec3::new(d.x, d.y, 0.)
//                                     / (scale * time.delta_seconds_f64() * game_speed.0 * 20.)
//                                     + focus_speed;
//                                 let pos = DVec3::new(
//                                     shooting_transform.x as f64,
//                                     shooting_transform.y as f64,
//                                     0.,
//                                 ) / scale;

//                                 let (x, y, _) = (pos * scale).as_vec3().into();
//                                 commands
//                                     .spawn(MaterialMesh2dBundle {
//                                         mesh: Mesh2dHandle(
//                                             meshes.add(Circle { radius: MIN_RADIUS }),
//                                         ),
//                                         material: colors.ships.gui.clone(),
//                                         transform: Transform::from_xyz(x, y, 1.),
//                                         ..default()
//                                     })
//                                     .insert(Velocity(speed))
//                                     .insert(Position(pos))
//                                     .insert(Acceleration(DVec3::ZERO))
//                                     .insert(GravityBound);
//                             }
//                         }
//                         *shooting_state = ShootingState::Idle;
//                     }
//                 }
//             }
//         }
//     }
// }
