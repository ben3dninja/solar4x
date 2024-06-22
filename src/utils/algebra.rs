use std::f64::consts::PI;

use bevy::math::{DVec2, DVec3};
use nalgebra::Vector3;

pub fn inorm(v: Vector3<i64>) -> i64 {
    ((v.x.pow(2) + v.y.pow(2) + v.z.pow(2)) as f64)
        .sqrt()
        .round() as i64
}

pub fn rads(x: f64) -> f64 {
    x * PI / 180.
}

pub fn degs(x: f64) -> f64 {
    x * 180. / PI
}
pub fn mod_180(x: f64) -> f64 {
    let x = x % 360.;
    if x > 180. {
        x - 360.
    } else {
        x
    }
}

/// Computes the coordinates of the projection of v on the (v1, v2) plane, provided they form an orthonormal basis
pub fn project_onto_plane(v: DVec3, basis: (DVec3, DVec3)) -> DVec2 {
    DVec2::new(v.dot(basis.0), v.dot(basis.1))
}
