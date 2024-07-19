use bevy::ecs::system::Resource;
use serde::{Deserialize, Serialize};

use super::{
    body_data::{BodyData, BodyType},
    BodyID,
};

#[derive(Resource, Clone, Serialize, Deserialize)]
pub enum BodiesConfig {
    SmallestBodyType(BodyType),
    IDs(Vec<BodyID>),
}

impl Default for BodiesConfig {
    fn default() -> Self {
        BodiesConfig::SmallestBodyType(BodyType::Planet)
    }
}

impl BodiesConfig {
    pub fn into_filter(self) -> Box<dyn FnMut(&BodyData) -> bool> {
        match self {
            BodiesConfig::SmallestBodyType(body_type) => {
                Box::new(move |data: &BodyData| data.body_type <= body_type)
            }
            BodiesConfig::IDs(v) => Box::new(move |data: &BodyData| v.contains(&data.id)),
        }
    }
}
