use bevy::prelude::*;

use crate::core_plugin::{start_game, BodiesConfig, CorePlugin};

#[derive(Default)]
pub struct StandalonePlugin(pub BodiesConfig);

impl Plugin for StandalonePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CorePlugin)
            .insert_resource(self.0.clone())
            .add_systems(Startup, start_game);
    }
}
