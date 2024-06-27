use bevy::{
    a11y::AccessibilityPlugin,
    core_pipeline::CorePipelinePlugin,
    input::InputPlugin,
    math::DVec3,
    prelude::*,
    render::{camera::ScalingMode, pipelined_rendering::PipelinedRenderingPlugin, RenderPlugin},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle, SpritePlugin},
    text::TextPlugin,
    ui::UiPlugin,
    winit::WinitPlugin,
};

use bevy::render::color::Color as GuiColor;
use ratatui::style::Color as TuiColor;

use crate::{
    bodies::body_data::BodyType,
    core_plugin::{AppState, BodyInfo},
    engine_plugin::Position,
    tui_plugin::{
        space_map_plugin::{initialize_space_map, SpaceMap},
        InitializeUiSet,
    },
    utils::algebra::project_onto_plane,
};

const MAX_WIDTH: f32 = 1000.;
const MIN_RADIUS: f32 = 1.;
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
        .add_systems(Startup, gui_setup)
        .add_systems(
            OnEnter(AppState::Game),
            (insert_display_components, update_transform)
                .chain()
                .in_set(InitializeUiSet)
                .after(initialize_space_map),
        )
        .add_systems(FixedPostUpdate, update_transform);
    }
}

fn gui_setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedHorizontal(MAX_WIDTH);
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
}

fn insert_display_components(
    mut commands: Commands,
    query: Query<(Entity, &BodyInfo)>,
    mut meshes: ResMut<Assets<Mesh>>,
    colors: ResMut<Colors>,
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
