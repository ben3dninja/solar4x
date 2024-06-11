use std::{cell::RefCell, rc::Rc};

use nalgebra::{Vector2, Vector3};

use crate::utils::algebra::{degs, mod_180, rads};

use super::{
    body_data::{BodyData, BodyType},
    body_id::BodyID,
    BodySystem,
};

const E_TOLERANCE: f64 = 1e-6;

// pub type RefBody = Rc<RefCell<Body>>;

pub struct Body {
    pub id: BodyID,
    pub orbit: Orbit,
    pub system: Rc<RefCell<BodySystem>>,
    pub info: BodyInfo,
    pub orbiting_bodies: Vec<BodyID>,
    pub host_body: Option<BodyID>,
}

// Stores orbital parameters and calculates the position in the host body's frame
#[derive(Default)]
pub struct Orbit {
    // lengths in km, angles in degrees and times in days
    pub eccentricity: f64,
    pub semimajor_axis: i64,
    pub inclination: f64,
    pub long_asc_node: f64,
    pub arg_periapsis: f64,
    pub initial_mean_anomaly: f64,

    pub revolution_period: f64,

    mean_anomaly: f64,
    eccentric_anomaly: f64,
    orbital_position: Vector2<i64>,
    orbital_velocity: Vector2<i64>,
    position: Vector3<i64>,
    velocity: Vector3<i64>,

    pub update_state: UpdateState,
    last_update_time: f64,
}

#[derive(Debug, Clone)]
pub struct BodyInfo {
    pub name: String,
    pub periapsis: i64,
    pub apoapsis: i64,
    pub body_type: BodyType,
    pub radius: f64,
    // pub mass: BigInt,
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
impl Orbit {
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

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Body {
    pub fn new(data: BodyData, system: Rc<RefCell<BodySystem>>) -> Self {
        Self {
            id: data.id.clone(),
            orbit: Orbit {
                eccentricity: data.eccentricity,
                semimajor_axis: data.semimajor_axis,
                inclination: data.inclination,
                long_asc_node: data.long_asc_node,
                arg_periapsis: data.arg_periapsis,
                initial_mean_anomaly: data.initial_mean_anomaly,
                revolution_period: data.revolution_period,
                mean_anomaly: data.initial_mean_anomaly,
                ..Default::default()
            },
            system,
            info: BodyInfo {
                name: data.name.clone(),
                apoapsis: data.apoapsis,
                periapsis: data.periapsis,
                body_type: data.body_type,
                radius: data.radius,
            },
            orbiting_bodies: data.orbiting_bodies,
            host_body: data.host_body,
        }
    }

    pub fn with_host_body(self, host_body: BodyID) -> Self {
        let mut new = self;
        new.host_body = Some(host_body);
        new
    }

    pub fn with_orbiting_bodies(self, orbiting_bodies: Vec<BodyID>) -> Self {
        let mut new = self;
        new.orbiting_bodies = orbiting_bodies;
        new
    }

    pub fn get_xyz(&self) -> (i64, i64, i64) {
        let pos = self.orbit.position;
        (pos.x, pos.y, pos.z)
    }
}
