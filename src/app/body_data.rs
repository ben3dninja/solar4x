use std::fmt::Display;

use serde::Deserialize;

use crate::utils::de::deserialize_options;

use super::body_id::BodyID;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub enum BodyType {
    Star,
    #[default]
    Planet,
    Moon,
    Asteroid,
    #[serde(alias = "Dwarf Planet")]
    DwarfPlanet,
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

#[derive(Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BodyData {
    #[serde(rename(deserialize = "rel"))]
    pub id: BodyID,
    #[serde(rename(deserialize = "englishName"))]
    pub name: String,
    pub body_type: BodyType,
    #[serde(alias = "aroundPlanet")]
    pub host_body: Option<BodyID>,
    #[serde(alias = "moons", deserialize_with = "deserialize_options")]
    pub orbiting_bodies: Vec<BodyID>,

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
    pub radius: f64,
}

#[cfg(test)]
mod tests {

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
        let body_data: BodyData = from_str(data).unwrap();
        assert_eq!(
            body_data,
            BodyData {
                id: "lune".into(),
                name: "Moon".into(),
                body_type: BodyType::Moon,
                host_body: Some("terre".into()),
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
                radius: 1737.
            }
        );
    }
}
