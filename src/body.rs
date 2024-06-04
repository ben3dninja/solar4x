use serde::Deserialize;

use body_id::BodyID;

mod body_id;

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct BodyData {
    #[serde(rename(deserialize = "rel"))]
    body_id: BodyID,
    #[serde(alias = "englishName")]
    body_name: String,
    is_planet: bool,
    #[serde(alias = "aroundPlanet")]
    host_body: BodyID,
    #[serde(alias = "moons")]
    orbiting_bodies: Option<Vec<BodyID>>,

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
    use crate::body::body_id::BodyID;

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
        assert_eq!(body_data.host_body, BodyID(String::from("terre")));
    }
}
