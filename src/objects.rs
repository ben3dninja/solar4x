use bevy::prelude::SystemSet;

pub mod bodies;
pub mod id;
pub mod ships;

pub(crate) mod prelude {

    pub(crate) use super::bodies::{
        bodies_config::BodiesConfig,
        body_data::{BodyData, BodyType},
        BodiesMapping, BodyID, BodyInfo, PrimaryBody,
    };
    pub(crate) use super::id::id_from;
    pub(crate) use super::ships::{ShipEvent, ShipID, ShipInfo, ShipsMapping};
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ObjectsUpdate;
