use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::utils::de::deserialize_options;

use super::body_id::{BodyID, MainBodyID};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default, PartialOrd)]
pub enum BodyType {
    Star,
    #[default]
    Planet,
    Moon,
    #[serde(alias = "Dwarf Planet")]
    DwarfPlanet,
    Asteroid,
    Comet,
}

impl Display for BodyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Star => "Star",
            Self::Planet => "Planet",
            Self::Moon => "Moon",
            Self::Asteroid => "Asteroid",
            Self::DwarfPlanet => "Dwarf Planet",
            Self::Comet => "Comet",
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MainBodyData {
    #[serde(rename(deserialize = "rel"))]
    pub id: MainBodyID,
    #[serde(rename(deserialize = "englishName"))]
    pub name: String,
    pub body_type: BodyType,
    #[serde(alias = "aroundPlanet")]
    pub host_body: Option<MainBodyID>,
    #[serde(alias = "moons", deserialize_with = "deserialize_options")]
    pub orbiting_bodies: Vec<MainBodyID>,

    // Orbital elements
    pub semimajor_axis: i64,
    pub eccentricity: f64,
    pub inclination: f64,
    pub long_asc_node: f64,
    pub arg_periapsis: f64,
    #[serde(alias = "mainAnomaly")]
    pub initial_mean_anomaly: f64,

    #[serde(alias = "perihelion")]
    pub periapsis: i64,
    #[serde(alias = "aphelion")]
    pub apoapsis: i64,

    // Time
    // Time required to complete a cycle around the host body (in earth days)
    #[serde(alias = "sideralOrbit")]
    pub revolution_period: f64,
    // Time required to rotate around itself (in earth hours)
    #[serde(alias = "sideralRotation")]
    pub rotation_period: f64,

    #[serde(alias = "meanRadius")]
    radius: f64,
    #[serde(deserialize_with = "deserialize_options")]
    mass: Mass,
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct BodyData {
    pub id: BodyID,
    pub name: String,
    pub body_type: BodyType,
    pub host_body: Option<BodyID>,
    pub orbiting_bodies: Vec<BodyID>,

    // Orbital elements
    pub semimajor_axis: f64,
    pub eccentricity: f64,
    pub inclination: f64,
    pub long_asc_node: f64,
    pub arg_periapsis: f64,
    pub initial_mean_anomaly: f64,

    pub periapsis: f64,
    pub apoapsis: f64,

    // Time
    // Time required to complete a cycle around the host body (in earth days)
    pub revolution_period: f64,
    // Time required to rotate around itself (in earth hours)
    pub rotation_period: f64,

    pub radius: f64,
    pub mass: f64,
}

impl From<MainBodyData> for BodyData {
    fn from(value: MainBodyData) -> Self {
        Self {
            id: value.id.into(),
            name: value.name,
            body_type: value.body_type,
            host_body: value.host_body.map(Into::<BodyID>::into),
            orbiting_bodies: value
                .orbiting_bodies
                .into_iter()
                .map(Into::<BodyID>::into)
                .collect(),
            semimajor_axis: value.semimajor_axis as f64,
            eccentricity: value.eccentricity,
            inclination: value.inclination,
            long_asc_node: value.long_asc_node,
            arg_periapsis: value.arg_periapsis,
            initial_mean_anomaly: value.initial_mean_anomaly,
            periapsis: value.periapsis as f64,
            apoapsis: value.apoapsis as f64,
            revolution_period: value.revolution_period,
            rotation_period: value.rotation_period,
            radius: value.radius,
            mass: value.mass.into(),
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Mass {
    mass_value: f64,
    mass_exponent: i32,
}

impl From<Mass> for f64 {
    fn from(value: Mass) -> Self {
        value.mass_value * 10f64.powi(value.mass_exponent)
    }
}

#[cfg(test)]
mod tests {

    use crate::bodies::{body_data::MainBodyData, body_id::id_from};

    use super::{BodyData, BodyType};
    use serde_json::from_str;

    #[test]
    fn test_de() {
        let data = r#"
        {
            "id": "lune",
            "name": "La Lune",
            "englishName": "Moon",
            "isPlanet": false,
            "moons": null,
            "semimajorAxis": 384400,
            "perihelion": 363300,
            "aphelion": 405500,
            "eccentricity": 0.05490,
            "inclination": 5.14500,
            "mass": {
                "massValue": 7.34600,
                "massExponent": 22
            },
            "vol": {
                "volValue": 2.19680,
                "volExponent": 10
            },
            "density": 3.34400,
            "gravity": 1.62000,
            "escape": 2380.00000,
            "meanRadius": 1737.00000,
            "equaRadius": 1738.10000,
            "polarRadius": 1736.00000,
            "flattening": 0.00120,
            "dimension": "",
            "sideralOrbit": 27.32170,
            "sideralRotation": 655.72800,
            "aroundPlanet": {
                "planet": "terre",
                "rel": "https://api.le-systeme-solaire.net/rest/bodies/terre"
            },
            "discoveredBy": "",
            "discoveryDate": "",
            "alternativeName": "",
            "axialTilt": 6.68,
            "avgTemp": 0,
            "mainAnomaly": 0.00000,
            "argPeriapsis": 0.00000,
            "longAscNode": 0.00000,
            "bodyType": "Moon",
            "rel": "https://api.le-systeme-solaire.net/rest/bodies/lune"
        }
        "#;
        let body_data: MainBodyData = from_str(data).unwrap();
        assert_eq!(
            BodyData::from(body_data),
            BodyData {
                id: id_from("lune"),
                name: "Moon".into(),
                body_type: BodyType::Moon,
                host_body: Some(id_from("terre")),
                orbiting_bodies: Vec::new(),
                semimajor_axis: 384400.,
                eccentricity: 0.0549,
                inclination: 5.145,
                long_asc_node: 0.,
                arg_periapsis: 0.,
                initial_mean_anomaly: 0.,
                periapsis: 363300.,
                apoapsis: 405500.,
                revolution_period: 27.32170,
                rotation_period: 655.72800,
                radius: 1737.,
                mass: 7.346e22
            }
        );
    }
}
