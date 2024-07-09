use bevy::prelude::*;

use crate::core_plugin::BodiesConfig;

pub struct SingleplayerPlugin {
    pub config: BodiesConfig,
}

impl Plugin for SingleplayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone());
    }
}
