use std::f64::consts::TAU;

use bevy::math::{DMat3, DVec2, DVec3};
use rand::Rng;

use crate::physics::G;

pub fn mod_180(x: f64) -> f64 {
    let x = x % 360.;
    if x > 180. {
        x - 360.
    } else {
        x
    }
}

#[allow(non_snake_case)]
/// Rotates u with respect to angles in radians
pub fn rotate(u: DVec2, o: f64, O: f64, I: f64) -> DVec3 {
    let (x, y) = u.into();
    let x_glob = (o.cos() * O.cos() - o.sin() * O.sin() * I.cos()) * x
        + (-o.sin() * O.cos() - o.cos() * O.sin() * I.cos()) * y;
    let y_glob = (o.cos() * O.sin() + o.sin() * O.cos() * I.cos()) * x
        + (-o.sin() * O.sin() + o.cos() * O.cos() * I.cos()) * y;
    let z_glob = (o.sin() * I.sin()) * x + (o.cos() * I.sin()) * y;
    DVec3::new(x_glob, y_glob, z_glob)
}

/// Computes the coordinates of the projection of v on the (v1, v2) plane, provided they form an orthonormal basis
pub fn project_onto_plane(v: DVec3, basis: (DVec3, DVec3)) -> DVec2 {
    DVec2::new(v.dot(basis.0), v.dot(basis.1))
}

/// Transforms relative coordinates into absolute coordinates
pub fn orbital_to_global_matrix(
    origin_pos: DVec3,
    origin_speed: DVec3,
    pos: DVec3,
    speed: DVec3,
) -> DMat3 {
    let [v1, v2, v3] = relative_axes(pos - origin_pos, speed - origin_speed);
    DMat3::from_cols(v1, v2, v3)
}

pub fn global_to_orbital_matrix(
    origin_pos: DVec3,
    origin_speed: DVec3,
    pos: DVec3,
    speed: DVec3,
) -> DMat3 {
    orbital_to_global_matrix(origin_pos, origin_speed, pos, speed).inverse()
}

/// Produces three unit vectors pointing towards:
///
/// 1 - the "forward" direction of the ship, that is its speed compared to a host body
///
/// 2 - the direction orthogonal to both the first axis and the position of the ship relative to the host (i.e. orthogonal to the "radial" direction)
///
/// 3 - the cross product of the preceding two
///
/// If the orbit is circular (orthogonal position and speed are sufficient), then those correspond to orthoradial, radial and down.
pub fn relative_axes(
    relative_pos: impl Into<DVec3>,
    relative_speed: impl Into<DVec3>,
) -> [DVec3; 3] {
    let forward = relative_speed.into().normalize_or(-DVec3::X);
    let radial = relative_pos.into().normalize_or(DVec3::Y);
    let down = forward.cross(radial).normalize_or(
        forward
            .cross(DVec3::Y)
            .normalize_or(forward.cross(DVec3::X)),
    );
    let right = down.cross(forward).normalize();
    [forward, right, down]
}

/// Position and velocity for circular orbit at given altitude around body
/// For now : in the eccliptic plane, rotating trigonometrically  (TODO: random inclination and direction too)
pub fn circular_orbit_around_body(
    altitude: f64,
    body_mass: f64,
    body_pos: DVec3,
    body_speed: DVec3,
) -> (DVec3, DVec3) {
    let angle = rand::thread_rng().gen_range(0. ..TAU);
    let unit_pos = DVec2::from_angle(angle);
    let unit_speed = unit_pos.perp();
    let ((x, y), (vx, vy)) = (unit_pos.into(), unit_speed.into());
    (
        altitude * DVec3::new(x, y, 0.) + body_pos,
        (G * body_mass / altitude).sqrt() * DVec3::new(vx, vy, 0.) + body_speed,
    )
}

#[allow(non_snake_case)]
pub fn center_to_periapsis_direction(o: f64, O: f64, I: f64) -> DVec3 {
    DVec3::new(
        O.cos() * o.cos() - O.sin() * I.cos() * o.sin(),
        O.cos() * o.cos() + O.cos() * I.cos() * o.sin(),
        I.sin() * o.sin(),
    )
}

pub fn ellipse_half_sizes(a: f64, e: f64) -> DVec2 {
    DVec2::new(1., (1. - e * e).sqrt()) * a
}
