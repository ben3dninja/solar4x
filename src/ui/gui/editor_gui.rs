use crate::{
    physics::predictions::Prediction,
    prelude::*,
    ui::{
        gui::SelectionRadius,
        screen::editor::{
            ClearOnEditorExit, ConfirmThrust, EditorContext, SelectNode, UpdateThrust,
            PREDICTIONS_NUMBER,
        },
        widget::space_map::SpaceMap,
        RenderSet,
    },
    utils::ui::maneuver_gizmos_directions,
};
use bevy::{
    color::palettes::css::{BLUE, DARK_BLUE, DARK_GREEN, DARK_RED, GREEN, RED, WHITE},
    input::{
        common_conditions::{input_just_released, input_pressed},
        mouse::{MouseButtonInput, MouseMotion},
        ButtonState,
    },
    prelude::*,
    window::PrimaryWindow,
};

use super::{SelectObjectEvent, MAX_HEIGHT};

const GIZMO_COLORS: [Color; 6] = [
    Color::Srgba(RED),
    Color::Srgba(DARK_RED),
    Color::Srgba(GREEN),
    Color::Srgba(DARK_GREEN),
    Color::Srgba(BLUE),
    Color::Srgba(DARK_BLUE),
];

pub fn plugin(app: &mut App) {
    app.init_resource::<CurrentGizmo>()
        .add_systems(
            PostUpdate,
            (draw_predictions, draw_maneuver_node)
                .in_set(RenderSet)
                .run_if(in_state(Loaded).and_then(resource_exists::<EditorContext>)),
        )
        .add_systems(
            Update,
            (despawn_arrows, spawn_arrows)
                .chain()
                .after(EventHandling)
                .run_if(on_event::<SelectNode>()),
        )
        .add_systems(
            Update,
            (
                handle_drag_gizmo
                    .run_if(input_pressed(MouseButton::Left).and_then(on_event::<MouseMotion>())),
                handle_click_gizmo.run_if(on_event::<SelectObjectEvent>()),
                handle_release_gizmo.run_if(input_just_released(MouseButton::Left)),
            )
                .run_if(in_state(Loaded).and_then(resource_exists::<EditorContext>)),
        );
}
#[derive(Resource, PartialEq, Default)]
/// Contains the direction of the gizmo (unit 3vector) and the initial position of the mouse
pub(super) struct CurrentGizmo(pub Option<GizmoDraggingState>);

#[derive(PartialEq)]
pub(super) struct GizmoDraggingState {
    gizmo: Entity,
    direction: Vec3,
    initial_mouse_pos: Vec2,
}

#[derive(Component, Clone, Debug)]
struct ArrowGizmo {
    color: Color,
    /// The unit vector pointing in the direction of the arrow
    direction: Vec3,
}

fn draw_predictions(
    mut gizmos: Gizmos,
    predictions: Query<(&Transform, &Prediction)>,
    space_map: Res<SpaceMap>,
) {
    for (t, p) in predictions.iter() {
        gizmos.circle_2d(
            t.translation.xy(),
            (1. - p.index as f32 / PREDICTIONS_NUMBER as f32) * MAX_HEIGHT
                / (500. * space_map.zoom_level as f32),
            Color::srgba(1., 1., 1., 0.1),
        );
    }
}

fn spawn_arrows(
    context: Res<EditorContext>,
    space_map: Res<SpaceMap>,
    mut commands: Commands,
    positions: Query<&Transform>,
    speeds: Query<&Velocity>,
) {
    if let Some(e) = context.selected_prediction_entity() {
        let scale = MAX_HEIGHT as f64 / space_map.system_size;
        let speed = (speeds.get(e).unwrap().0 * scale).as_vec3();
        let pos = positions.get(e).unwrap().translation;
        let focus = space_map
            .focus_body
            .map_or(Vec3::ZERO, |e| positions.get(e).unwrap().translation);
        eprintln!("Pos: {}, speed: {}, focus: {}", pos, speed, focus);
        let directions = maneuver_gizmos_directions(pos, speed, focus);
        for (i, d) in directions.iter().enumerate() {
            eprintln!("Spawning gizmo with direction {:?}", d);
            commands.spawn((
                ArrowGizmo {
                    color: GIZMO_COLORS[i],
                    direction: *d,
                },
                ClearOnEditorExit,
                SelectionRadius {
                    min_radius: MAX_HEIGHT / 50.,
                    ..default()
                },
            ));
        }
    }
}

fn despawn_arrows(mut commands: Commands, entities: Query<Entity, With<ArrowGizmo>>) {
    entities.iter().for_each(|e| commands.entity(e).despawn());
}

fn draw_maneuver_node(
    mut gizmos: Gizmos,
    space_map: Res<SpaceMap>,
    context: Res<EditorContext>,
    positions: Query<&Transform>,
    current_gizmo: Res<CurrentGizmo>,
    arrows: Query<(Entity, &ArrowGizmo)>,
) {
    if let Some(e) = context.selected_prediction_entity() {
        let pos = positions.get(e).unwrap().translation;
        let radius = MAX_HEIGHT / (50. * space_map.zoom_level as f32);
        gizmos.circle_2d(pos.xy(), radius, WHITE);
        for (gizmo, arrow) in arrows.iter() {
            let mut color = arrow.color;
            if current_gizmo.0.as_ref().is_some_and(|g| g.gizmo == gizmo) {
                color = color.mix(&Color::WHITE, 0.5);
            }
            gizmos.arrow(
                pos + arrow.direction * radius,
                pos + arrow.direction * 3. * radius,
                color,
            );
        }
    }
}

fn handle_click_gizmo(
    mut select_event: EventReader<SelectObjectEvent>,
    arrows: Query<&ArrowGizmo>,
    mut current_gizmo: ResMut<CurrentGizmo>,
) {
    for event in select_event.read() {
        if let Ok(arrow) = arrows.get(event.entity) {
            current_gizmo.0 = Some(GizmoDraggingState {
                gizmo: event.entity,
                direction: arrow.direction,
                initial_mouse_pos: event.cursor_pos,
            })
        }
    }
}

fn handle_release_gizmo(
    mut input: EventReader<MouseButtonInput>,
    mut current_gizmo: ResMut<CurrentGizmo>,
    mut confirm_thrust: EventWriter<ConfirmThrust>,
) {
    for event in input.read() {
        if matches!(event.state, ButtonState::Released) && current_gizmo.0.is_some() {
            current_gizmo.0 = None;
            confirm_thrust.send(ConfirmThrust);
        }
    }
}

fn handle_drag_gizmo(
    window: Query<&Window, With<PrimaryWindow>>,
    current_gizmo: Res<CurrentGizmo>,
    space_map: Res<SpaceMap>,
    mut update_thrust: EventWriter<UpdateThrust>,
) {
    if let Some(cursor_pos) = window.single().cursor_position() {
        if let Some(GizmoDraggingState {
            direction,
            initial_mouse_pos,
            ..
        }) = current_gizmo.0
        {
            let zoom = space_map.zoom_level;
            let scale = MAX_HEIGHT as f64 * space_map.system_size;
            update_thrust.send(UpdateThrust(
                direction.as_dvec3() * (cursor_pos - initial_mouse_pos).length() as f64
                    / (zoom * scale),
            ));
        }
    }
}
