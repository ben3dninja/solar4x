use bevy::{
    a11y::AccessibilityPlugin,
    core_pipeline::CorePipelinePlugin,
    input::{mouse::MouseButtonInput, ButtonState, InputPlugin},
    math::DVec3,
    prelude::*,
    render::{camera::ScalingMode, pipelined_rendering::PipelinedRenderingPlugin, RenderPlugin},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle, SpritePlugin},
    text::TextPlugin,
    ui::UiPlugin,
    window::PrimaryWindow,
    winit::WinitPlugin,
};

use bevy::render::color::Color as GuiColor;
use ratatui::style::Color as TuiColor;

use crate::{
    bodies::body_data::BodyType,
    core_plugin::{AppState, BodyInfo},
    engine_plugin::Position,
    gravity::{Acceleration, GravityBound, Speed},
    tui_plugin::{
        space_map_plugin::{initialize_space_map, FocusBody, SpaceMap},
        InitializeUiSet,
    },
    utils::algebra::project_onto_plane,
};

const MAX_WIDTH: f32 = 1000.;
const MIN_RADIUS: f32 = 0.2;
pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TransformPlugin,
            InputPlugin,
            WindowPlugin::default(),
            AccessibilityPlugin,
            AssetPlugin::default(),
            WinitPlugin::default(),
            RenderPlugin::default(),
            ImagePlugin::default(),
            PipelinedRenderingPlugin,
            CorePipelinePlugin,
            SpritePlugin,
            TextPlugin,
            UiPlugin,
        ))
        .insert_resource(ClearColor(GuiColor::BLACK))
        .insert_resource(ShootingState::Idle)
        .add_systems(Startup, gui_setup)
        .add_systems(
            OnEnter(AppState::Game),
            (insert_display_components, update_transform)
                .chain()
                .in_set(InitializeUiSet)
                .after(initialize_space_map),
        )
        .add_systems(FixedPostUpdate, update_transform)
        .add_systems(Update, (shoot, update_camera_pos));
    }
}

fn gui_setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(MAX_WIDTH);
    commands.spawn(cam);
    let colors = Colors {
        stars: Color {
            tui: TuiColor::Yellow,
            gui: materials.add(GuiColor::GOLD),
        },
        planets: Color {
            tui: TuiColor::Blue,
            gui: materials.add(GuiColor::TEAL),
        },
        other: Color {
            tui: TuiColor::DarkGray,
            gui: materials.add(GuiColor::DARK_GRAY),
        },
        selected: Color {
            tui: TuiColor::Red,
            gui: materials.add(GuiColor::MAROON),
        },
        ships: Color {
            tui: TuiColor::Magenta,
            gui: materials.add(GuiColor::PURPLE),
        },
    };
    commands.insert_resource(colors);
}

#[derive(Clone)]
pub struct Color {
    tui: TuiColor,
    gui: Handle<ColorMaterial>,
}

#[derive(Resource)]
pub struct Colors {
    stars: Color,
    planets: Color,
    other: Color,
    selected: Color,
    ships: Color,
}

fn insert_display_components(
    mut commands: Commands,
    query: Query<(Entity, &BodyInfo)>,
    mut meshes: ResMut<Assets<Mesh>>,
    colors: Res<Colors>,
    space_map: Res<SpaceMap>,
) {
    let scale = MAX_WIDTH as f64 / space_map.system_size;
    query.iter().for_each(|(e, BodyInfo(data))| {
        let (material, z) = match data.body_type {
            BodyType::Star => (colors.stars.gui.clone(), 0.),
            BodyType::Planet => (colors.planets.gui.clone(), -1.),
            _ => (colors.other.gui.clone(), -2.),
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

fn update_camera_pos(
    mut cam: Query<(&mut Transform, &mut OrthographicProjection)>,
    focus_pos: Query<&Position, With<FocusBody>>,
    space_map: Res<SpaceMap>,
) {
    let scale = MAX_WIDTH as f64 / space_map.system_size;
    let (mut cam_pos, mut proj) = cam.single_mut();
    cam_pos.translation =
        ((focus_pos.single().0 + DVec3::new(space_map.offset.x, space_map.offset.y, 0.)) * scale)
            .as_vec3();
    proj.scale = (1. / space_map.zoom_level) as f32;
}

fn update_transform(space_map: Res<SpaceMap>, mut query: Query<(&mut Transform, &Position)>) {
    let scale = MAX_WIDTH as f64 / space_map.system_size;
    for (mut transform, Position(pos)) in query.iter_mut() {
        let (x, y) = (project_onto_plane(*pos, (DVec3::X, DVec3::Y)) * scale)
            .as_vec2()
            .into();
        transform.translation.x = x;
        transform.translation.y = y;
    }
}

#[derive(Resource)]
enum ShootingState {
    Idle,
    Loading { initial_mouse_pos: Vec2 },
}

fn shoot(
    mut commands: Commands,
    mut shooting_state: ResMut<ShootingState>,
    mut buttons: EventReader<MouseButtonInput>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    colors: Res<Colors>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    space_map: Res<SpaceMap>,
) {
    if let Some(position) = window.single().cursor_position() {
        for event in buttons.read() {
            match event.state {
                ButtonState::Pressed => {
                    *shooting_state = ShootingState::Loading {
                        initial_mouse_pos: position,
                    }
                }
                ButtonState::Released => {
                    if let ShootingState::Loading { initial_mouse_pos } = *shooting_state {
                        let (camera, camera_transform) = camera_query.single();

                        if let (Some(point1), Some(point2)) = (
                            camera.viewport_to_world_2d(camera_transform, initial_mouse_pos),
                            camera.viewport_to_world_2d(camera_transform, position),
                        ) {
                            let scale = MAX_WIDTH as f64 / space_map.system_size;
                            let d = (point1 - point2).as_dvec2();
                            let speed = DVec3::new(d.x, d.y, 0.) / (scale * 20.);
                            let pos = DVec3::new(point1.x as f64, point1.y as f64, 0.) / scale;
                            commands
                                .spawn(MaterialMesh2dBundle {
                                    mesh: Mesh2dHandle(meshes.add(Circle { radius: MIN_RADIUS })),
                                    material: colors.ships.gui.clone(),
                                    transform: Transform::from_xyz(point1.x, point1.y, 1.),
                                    ..default()
                                })
                                .insert(Speed(speed))
                                .insert(Position(pos))
                                .insert(Acceleration(DVec3::ZERO))
                                .insert(GravityBound);
                        }
                        *shooting_state = ShootingState::Idle;
                    }
                }
            }
        }
    }
}
