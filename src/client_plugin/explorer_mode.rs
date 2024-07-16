use bevy::prelude::*;

use crate::{
    bodies::bodies_config::BodiesConfig,
    core_plugin::LoadingState,
    engine_plugin::{GameTime, ToggleTime},
};

use super::ClientMode;

/// This plugin's role is to handle the game logic behind the client explorer mode
pub struct ExplorerPlugin(pub BodiesConfig);

impl Plugin for ExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone())
            .add_systems(OnEnter(ClientMode::Explorer), start_explorer);
    }
}

fn start_explorer(
    mut app_state: ResMut<NextState<LoadingState>>,
    time: Option<ResMut<GameTime>>,
    toggle_time: Option<ResMut<ToggleTime>>,
) {
    app_state.set(LoadingState::Loading);
    if let Some(mut t) = time {
        *t = GameTime::default()
    }
    if let Some(mut t) = toggle_time {
        t.0 = true
    }
}
