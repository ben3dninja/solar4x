use arrayvec::ArrayString;
use bevy::prelude::Component;

mod body;
mod ship;
mod id;

const MAX_OBJECT_NAME_LENGTH: usize = 32;

pub enum ObjectType {
    /// A "Body" is a celestial body whose position is entirely determined by the
    /// current simtick, following orbital mechanics.
    Body,
    /// A "Ship" is an object whose movement is governed by the gravitationnal
    /// attraction of the celestial bodies, along with custom trajectories
    Ship
}

/// Each space object is given an IDentifier, which is unique in a specific game.
#[derive(Component)]
pub struct ID {
    object_type: ObjectType,
    name: ArrayString<MAX_OBJECT_NAME_LENGTH>,
}