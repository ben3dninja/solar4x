use bevy::math::{DVec2, DVec3};

pub fn mod_180(x: f64) -> f64 {
    let x = x % 360.;
    if x > 180. {
        x - 360.
    } else {
        x
    }
}

#[allow(non_snake_case)]
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

/// Transforms radial/orthoradial/perp coordinates into absolute coordinates
pub fn convert_orbital_to_global(
    thrust: DVec3,
    origin: DVec3,
    origin_speed: DVec3,
    pos: DVec3,
    speed: DVec3,
) -> DVec3 {
    let v1 = (pos - origin).normalize();
    let v2 = (speed - origin_speed).try_normalize().unwrap_or(DVec3::Z);
    let v3 = v1.cross(v2);

    thrust.x * v1 + thrust.y * v2 + thrust.z * v3
}
