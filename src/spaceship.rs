use arrayvec::ArrayString;
use bevy::{math::DVec3, prelude::*, utils::HashMap};

const MAX_ID_LENGTH: usize = 32;

pub type ShipID = ArrayString<MAX_ID_LENGTH>;

#[derive(Component, Clone)]
pub struct ShipInfo {
    pub id: ShipID,
    pub spawn_pos: DVec3,
    pub spawn_speed: DVec3,
}

#[derive(Resource, Default)]
pub struct ShipsMapping(pub HashMap<ShipID, Entity>);

#[derive(Event)]
pub enum ShipEvent {
    Create(ShipInfo),
    Remove(ShipID),
}
