use serde::Deserialize;

use body_id::BodyID;

use crate::utils::de::deserialize_options;

mod body_id;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum BodyType {
    Star,
    Planet,
    Moon,
    Asteroid,
    #[serde(alias = "Dwarf Planet")]
    DwarfPlanet,
    Comet,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BodyData {
    #[serde(rename(deserialize = "rel"))]
    id: BodyID,
    #[serde(rename(deserialize = "englishName"))]
    name: String,
    body_type: BodyType,
    #[serde(alias = "aroundPlanet", deserialize_with = "deserialize_options")]
    host_body: BodyID,
    #[serde(alias = "moons", deserialize_with = "deserialize_options")]
    orbiting_bodies: Vec<BodyID>,

    // Orbital elements
    semimajor_axis: i64,
    eccentricity: f64,
    inclination: f64,
    long_asc_node: f64,
    arg_periapsis: f64,
    #[serde(alias = "mainAnomaly")]
    mean_anomaly: f64,

    #[serde(alias = "perihelion")]
    periapsis: i64,
    #[serde(alias = "aphelion")]
    apoapsis: i64,
}

#[cfg(test)]
mod tests {

    use crate::body::BodyType;

    use super::BodyData;
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
                host_body: "terre".into(),
                orbiting_bodies: Vec::new(),
                semimajor_axis: 384400,
                eccentricity: 0.0549,
                inclination: 5.145,
                long_asc_node: 0.,
                arg_periapsis: 0.,
                mean_anomaly: 0.,
                periapsis: 363300,
                apoapsis: 405500
            }
        );
    }
}
