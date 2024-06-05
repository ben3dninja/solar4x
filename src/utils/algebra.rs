use std::f64::consts::PI;

pub fn norm(x: i64, y: i64, z: i64) -> i64 {
    ((x * x + y * y + z * z) as f64).sqrt().round() as i64
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
