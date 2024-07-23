use crate::{
    physics::predictions::Prediction,
    prelude::*,
    ui::{
        screen::editor::{EditorContext, PREDICTIONS_NUMBER},
        widget::space_map::SpaceMap,
        RenderSet,
    },
};
use bevy::{
    color::palettes::css::{BLUE, GREEN, RED, WHITE},
    prelude::*,
};

use super::MAX_HEIGHT;

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (
            draw_predictions,
            draw_maneuver_node.run_if(resource_exists::<EditorContext>),
        )
            .in_set(RenderSet)
            .run_if(in_state(Loaded).and_then(resource_exists::<SpaceMap>)),
    );
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

fn draw_maneuver_node(
    mut gizmos: Gizmos,
    space_map: Res<SpaceMap>,
    context: Res<EditorContext>,
    positions: Query<&Transform>,
    speeds: Query<&Velocity>,
) {
    if let Some(e) = context.selected_prediction_entity() {
        let scale = MAX_HEIGHT as f64 * space_map.system_size;
        let forward = (speeds.get(e).unwrap().0 * scale).normalize().as_vec3();
        let center = positions.get(e).unwrap().translation;
        let focus = space_map
            .focus_body
            .map_or(Vec3::ZERO, |e| positions.get(e).unwrap().translation);
        let radial = (center - focus).normalize();
        let down = forward
            .cross(radial)
            .normalize_or(forward.cross(Vec3::Z).normalize_or(forward.cross(Vec3::Y)));
        let right = down.cross(forward).normalize();
        let radius = MAX_HEIGHT / (50. * space_map.zoom_level as f32);
        gizmos.circle_2d(center.xy(), radius, WHITE);
        gizmos.arrow(
            center + radius * forward,
            center + 3. * radius * forward,
            RED,
        );
        gizmos.arrow(center + radius * right, center + 3. * radius * right, GREEN);
        gizmos.arrow(center + radius * down, center + 3. * radius * down, BLUE);
    }
}
