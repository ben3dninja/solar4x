use serde::{de::Visitor, Deserialize, Deserializer};

use crate::{
    objects::id::{id_from, MAX_ID_LENGTH},
    utils::de::deserialize_options,
};

use super::{
    body_data::{BodyData, BodyType},
    BodyID,
};

const ID_PREFIX: &str = "https://api.le-systeme-solaire.net/rest/bodies/";

#[derive(PartialEq, Debug, Clone)]
pub struct MainBodyID(pub String);

impl From<&str> for MainBodyID {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<MainBodyID> for BodyID {
    fn from(value: MainBodyID) -> Self {
        id_from(&value.0)
    }
}

struct IDVisitor;

impl<'de> Visitor<'de> for IDVisitor {
    type Value = MainBodyID;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a body id")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut id = None;
        while let Some((key, value)) = map.next_entry::<&str, &str>()? {
            if key == "rel" {
                id = strip_id_prefix(value);
            }
        }
        id.ok_or(serde::de::Error::custom(
            "id could not be deserialized from map",
        ))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        strip_id_prefix(v).ok_or(serde::de::Error::custom(
            "id could not be deserialized from string",
        ))
    }
}

fn strip_id_prefix(s: &str) -> Option<MainBodyID> {
    s.strip_prefix(ID_PREFIX)
        .map(MainBodyID::from)
        .filter(|s| s.0.len() < MAX_ID_LENGTH)
}

impl<'de> Deserialize<'de> for MainBodyID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IDVisitor)
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

    use super::*;
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

    #[test]
    fn test_id_single() {
        let id: MainBodyID = from_str(
            r#"
            "https://api.le-systeme-solaire.net/rest/bodies/terre"
        "#,
        )
        .unwrap();
        assert_eq!(id, "terre".into());
    }

    #[test]
    fn test_id_map() {
        let id: MainBodyID = from_str(
            r#"
        {
            "planet": "terre",
            "rel": "https://api.le-systeme-solaire.net/rest/bodies/terre"
        }"#,
        )
        .unwrap();
        assert_eq!(id, "terre".into());
    }
}
