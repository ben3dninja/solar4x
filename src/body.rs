use body_data::BodyData;

use crate::utils::algebra::{degs, mod_180, rads};

pub mod body_data;
pub mod body_id;

const E_TOLERANCE: f64 = 1e-6;

#[allow(non_snake_case)]
#[derive(Default, Clone, Debug)]
pub struct Body {
    pub data: BodyData,
    pub mean_anomaly: f64,
    pub eccentric_anomaly: f64,
    x_orb: i64,
    y_orb: i64,
    x: i64,
    y: i64,
    z: i64,
    update_state: UpdateState,
    // time in days
    time: f64,
}

// State corresponding to which elements are up to date (for example if the state is M, only the mean anomaly is up to date while if it is Orb,
// then both the mean and eccentric anomalies are up to date, along with the orbital coordinates)
#[derive(Default, PartialEq, PartialOrd, Clone, Debug)]
enum UpdateState {
    #[default]
    None,
    M,
    E,
    Orb,
    Glob,
}

// see https://ssd.jpl.nasa.gov/planets/approx_pos.html
#[allow(non_snake_case)]
impl Body {
    pub fn set_time(&mut self, new_time: f64) {
        self.time = new_time;
        self.update_state = UpdateState::None;
    }

    fn update_M(&mut self) {
        self.mean_anomaly = mod_180(
            self.data.initial_mean_anomaly + 360. * self.time / self.data.revolution_period,
        );
        self.update_state = UpdateState::M;
    }
    fn update_E(&mut self) {
        if self.update_state < UpdateState::M {
            self.update_M()
        }
        let M = self.mean_anomaly;
        let e = self.data.eccentricity;
        let ed = degs(e);
        let mut E = M + ed * rads(M).sin();
        // TODO : change formulas to use radians instead
        let mut dM = M - (E - ed * rads(E).sin());
        let mut dE = dM / (1. - e * rads(E).cos());
        for _ in 0..100 {
            if dE.abs() <= E_TOLERANCE {
                break;
            }
            dM = M - (E - ed * rads(E).sin());
            dE = dM / (1. - e * rads(E).cos());
            E += dE;
        }
        self.eccentric_anomaly = E;
        self.update_state = UpdateState::E;
    }
    fn update_xy_orb(&mut self) {
        if self.update_state < UpdateState::E {
            self.update_E()
        }
        let a = self.data.semimajor_axis as f64;
        let E = rads(self.eccentric_anomaly);
        let e = self.data.eccentricity;
        self.x_orb = (a * (E.cos() - e)).round() as i64;
        self.y_orb = (a * (1. - e * e).sqrt() * E.sin()).round() as i64;
        self.update_state = UpdateState::Orb;
    }

    fn update_xyz(&mut self) {
        if self.update_state < UpdateState::Orb {
            self.update_xy_orb()
        }
        let x = self.x_orb as f64;
        let y = self.y_orb as f64;
        let o = rads(self.data.arg_periapsis);
        let O = rads(self.data.long_asc_node);
        let I = rads(self.data.inclination);
        self.x = ((o.cos() * O.cos() - o.sin() * O.sin() * I.cos()) * x
            + (-o.sin() * O.cos() - o.cos() * O.sin() * I.cos()) * y)
            .round() as i64;
        self.y = ((o.cos() * O.sin() + o.sin() * O.cos() * I.cos()) * x
            + (o.sin() * O.sin() + o.cos() * O.cos() * I.cos()) * y)
            .round() as i64;
        self.z = ((o.sin() * I.sin()) * x + (o.cos() * I.sin()) * y).round() as i64;
        self.update_state = UpdateState::Glob;
    }

    // TODO : use interior mutability
    pub fn get_xyz(&mut self) -> (i64, i64, i64) {
        if self.update_state < UpdateState::Glob {
            self.update_xyz();
        }
        (self.x, self.y, self.z)
    }
}

impl From<BodyData> for Body {
    fn from(value: BodyData) -> Self {
        let mut new = Self {
            data: value,
            ..Default::default()
        };
        new.update_xyz();
        new
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::algebra::norm;

    use super::{
        body_data::{BodyData, BodyType},
        Body,
    };

    #[test]
    fn test_moon() {
        let mut moon = Body::from(BodyData {
            id: "lune".into(),
            name: "Moon".into(),
            body_type: BodyType::Moon,
            host_body: "terre".into(),
            orbiting_bodies: Vec::new(),
            semimajor_axis: 384400,
            eccentricity: 0.0549,
            inclination: 5.145,
            long_asc_node: 0.,
            arg_periapsis: 0.,
            initial_mean_anomaly: 0.,
            periapsis: 363300,
            apoapsis: 405500,
            revolution_period: 27.32170,
            rotation_period: 655.72800,
        });
        for i in 0..27 {
            moon.set_time(i.into());
            let (x, y, z) = moon.get_xyz();
            dbg!(x, y, z, moon.mean_anomaly);
            assert!((norm(x, y, z) - 384400).abs() < 30000);
        }
    }
}
