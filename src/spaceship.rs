use arrayvec::ArrayString;
use bevy::{math::DVec3, prelude::*, utils::HashMap};

use crate::MAX_ID_LENGTH;

pub type ShipID = ArrayString<MAX_ID_LENGTH>;

#[derive(Component, Clone, Default, PartialEq)]
pub struct ShipInfo {
    pub id: ShipID,
    pub spawn_pos: DVec3,
    pub spawn_speed: DVec3,
}

#[derive(Resource, Default)]
pub struct ShipsMapping(pub HashMap<ShipID, Entity>);
