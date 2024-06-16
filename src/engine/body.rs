use nalgebra::{Vector2, Vector3};

use crate::{
    app::body_data::BodyData,
    utils::algebra::{degs, mod_180, rads},
};

const E_TOLERANCE: f64 = 1e-6;

// Stores orbital parameters and calculates the position in the host body's frame
#[derive(Default)]
pub struct Body {
    // lengths in km, angles in degrees and times in days
    eccentricity: f64,
    semimajor_axis: i64,
    inclination: f64,
    long_asc_node: f64,
    arg_periapsis: f64,
    initial_mean_anomaly: f64,
    revolution_period: f64,

    mean_anomaly: f64,
    eccentric_anomaly: f64,
    orbital_position: Vector2<i64>,
    orbital_velocity: Vector2<i64>,
    pub position: Vector3<i64>,
    pub velocity: Vector3<i64>,

    pub update_state: UpdateState,
    last_update_time: f64,
}

impl From<&BodyData> for Body {
    fn from(data: &BodyData) -> Self {
        Self {
            eccentricity: data.eccentricity,
            semimajor_axis: data.semimajor_axis,
            inclination: data.inclination,
            long_asc_node: data.long_asc_node,
            arg_periapsis: data.arg_periapsis,
            initial_mean_anomaly: data.initial_mean_anomaly,
            revolution_period: data.revolution_period,
            mean_anomaly: data.initial_mean_anomaly,
            ..Default::default()
        }
    }
}

// State corresponding to which elements are up to date (for example if the state is M, only the mean anomaly is up to date while if it is Orb,
// then both the mean atand eccentric anomalies are up to date, along with the orbital coordinates)
#[derive(Default, PartialEq, PartialOrd, Clone, Debug)]
pub enum UpdateState {
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
    fn update_M(&mut self, time: f64) {
        self.mean_anomaly =
            mod_180(self.initial_mean_anomaly + 360. * time / self.revolution_period);
        self.update_state = UpdateState::M;
    }
    fn update_E(&mut self, time: f64) {
        if self.update_state < UpdateState::M {
            self.update_M(time)
        }
        let M = self.mean_anomaly;
        let e = self.eccentricity;
        let ed = degs(e);
        let mut E = M + ed * rads(M).sin();
        // TODO : change formulas to use radians instead
        let mut dM = M - (E - ed * rads(E).sin());
        let mut dE = dM / (1. - e * rads(E).cos());
        for _ in 0..10 {
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
    fn update_orb_pos(&mut self, time: f64) {
        if self.update_state < UpdateState::E {
            self.update_E(time)
        }
        let a = self.semimajor_axis as f64;
        let E = rads(self.eccentric_anomaly);
        let e = self.eccentricity;
        let x = (a * (E.cos() - e)).round() as i64;
        let y = (a * (1. - e * e).sqrt() * E.sin()).round() as i64;
        self.orbital_position = Vector2::new(x, y);
        self.update_state = UpdateState::Orb;
    }

    pub fn update_pos(&mut self, time: f64) {
        if self.update_state < UpdateState::Orb {
            self.update_orb_pos(time)
        }
        if self.update_state >= UpdateState::Glob {
            return;
        }
        let x = self.orbital_position.x as f64;
        let y = self.orbital_position.y as f64;
        let o = rads(self.arg_periapsis);
        let O = rads(self.long_asc_node);
        let I = rads(self.inclination);
        let x_glob = ((o.cos() * O.cos() - o.sin() * O.sin() * I.cos()) * x
            + (-o.sin() * O.cos() - o.cos() * O.sin() * I.cos()) * y)
            .round() as i64;
        let y_glob = ((o.cos() * O.sin() + o.sin() * O.cos() * I.cos()) * x
            + (-o.sin() * O.sin() + o.cos() * O.cos() * I.cos()) * y)
            .round() as i64;
        let z_glob = ((o.sin() * I.sin()) * x + (o.cos() * I.sin()) * y).round() as i64;
        self.position = Vector3::new(x_glob, y_glob, z_glob);
        self.update_state = UpdateState::Glob;
        self.last_update_time = time;
    }
}
