// use std::{
//     cell::RefCell,
//     rc::{Rc, Weak},
// };

use body_data::BodyData;
// use nalgebra::{Vector2, Vector3};
// use num_bigint::BigInt;

use crate::utils::algebra::{degs, mod_180, rads};

use super::{
    body_data, // , body_id::BodyID, BodySystem
};

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
    pub update_state: UpdateState,
    // time in days
    pub time: f64,
}

// type ChildBody<'a> = Rc<RefCell<NewBody<'a>>>;
// type ParentBody<'a> = Weak<RefCell<NewBody<'a>>>;

// pub struct NewBody<'a> {
//     id: BodyID,
//     system: &'a BodySystem,
//     orbital_elements: OrbitalElements,
//     info: BodyInfo,
//     orbiting_bodies: Vec<ChildBody<'a>>,
//     host_body: ParentBody<'a>,
//     coordinates: BodyCoordinates,
// }

// struct OrbitalElements {
//     eccentricity: f64,
//     semimajor_axis: i64,
//     inclination: f64,
//     long_asc_node: f64,
//     arg_periapsis: f64,
//     mean_anomaly: f64,
// }

// struct BodyInfo {
//     name: String,
//     mass: BigInt,
// }

// #[allow(non_snake_case)]
// struct BodyCoordinates {
//     update_state: UpdateState,
//     last_update_time: f64,
//     M: f64,
//     E: f64,
//     orbital_position: Vector2<i64>,
//     orbital_velocity: Vector2<i64>,
//     position: Vector3<i64>,
//     velocity: Vector3<i64>,
// }

// impl BodyCoordinates {
//     fn update_M(&mut self) {
//         self.M = mod_180(
//             self.data.initial_mean_anomaly + 360. * self.time / self.data.revolution_period,
//         );
//         self.update_state = UpdateState::M;
//     }
//     fn update_E(&mut self) {
//         if self.update_state < UpdateState::M {
//             self.update_M()
//         }
//         let M = self.M;
//         let e = self.data.eccentricity;
//         let ed = degs(e);
//         let mut E = M + ed * rads(M).sin();
//         // TODO : change formulas to use radians instead
//         let mut dM = M - (E - ed * rads(E).sin());
//         let mut dE = dM / (1. - e * rads(E).cos());
//         for _ in 0..100 {
//             if dE.abs() <= E_TOLERANCE {
//                 break;
//             }
//             dM = M - (E - ed * rads(E).sin());
//             dE = dM / (1. - e * rads(E).cos());
//             E += dE;
//         }
//         self.eccentric_anomaly = E;
//         self.update_state = UpdateState::E;
//     }
//     fn update_orb(&mut self, elements: OrbitalElements) {
//         if self.update_state < UpdateState::E {
//             self.update_E()
//         }
//         let a = elements.semimajor_axis as f64;
//         let E = rads(self.E);
//         let e = elements.eccentricity;
//         self.orbital_position = Vector2::new(
//             (a * (E.cos() - e)).round() as i64,
//             (a * (1. - e * e).sqrt() * E.sin()).round() as i64,
//         );
//         self.update_state = UpdateState::Orb;
//     }

//     fn update_pos(&mut self) {
//         if self.update_state < UpdateState::Orb {
//             self.update_orb()
//         }
//         let (x, y) = (self.orbital_position.x, self.orbital_position.y);
//         let o = rads(self.data.arg_periapsis);
//         let O = rads(self.data.long_asc_node);
//         let I = rads(self.data.inclination);
//         self.x = ((o.cos() * O.cos() - o.sin() * O.sin() * I.cos()) * x
//             + (-o.sin() * O.cos() - o.cos() * O.sin() * I.cos()) * y)
//             .round() as i64;
//         self.y = ((o.cos() * O.sin() + o.sin() * O.cos() * I.cos()) * x
//             + (o.sin() * O.sin() + o.cos() * O.cos() * I.cos()) * y)
//             .round() as i64;
//         self.z = ((o.sin() * I.sin()) * x + (o.cos() * I.sin()) * y).round() as i64;
//         self.update_state = UpdateState::Glob;
//     }
// }

// State corresponding to which elements are up to date (for example if the state is M, only the mean anomaly is up to date while if it is Orb,
// then both the mean and eccentric anomalies are up to date, along with the orbital coordinates)
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

    pub fn update_xyz(&mut self) {
        if self.update_state < UpdateState::Orb {
            self.update_xy_orb()
        }
        if self.update_state >= UpdateState::Glob {
            return;
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
            + (-o.sin() * O.sin() + o.cos() * O.cos() * I.cos()) * y)
            .round() as i64;
        self.z = ((o.sin() * I.sin()) * x + (o.cos() * I.sin()) * y).round() as i64;
        self.update_state = UpdateState::Glob;
    }

    // TODO : use interior mutability
    pub fn get_updated_xyz(&mut self) -> (i64, i64, i64) {
        if self.update_state < UpdateState::Glob {
            self.update_xyz();
        }
        self.get_raw_xyz()
    }

    pub fn get_raw_xyz(&self) -> (i64, i64, i64) {
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

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.data.id == other.data.id
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
            let (x, y, z) = moon.get_updated_xyz();
            dbg!(x, y, z, moon.mean_anomaly);
            assert!((norm(x, y, z) - 384400).abs() < 30000);
        }
    }
}
