use arrayvec::ArrayString;
use bevy::prelude::Component;

const MAX_ID_LENGTH: usize = 32;

pub type ShipID = ArrayString<MAX_ID_LENGTH>;

#[derive(Component)]
pub struct ShipInfo {
    pub id: ShipID,
}
