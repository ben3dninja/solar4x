use bevy::prelude::*;

use crate::{
    core_plugin::{AppState, BodiesConfig, SystemInitSet},
    engine_plugin::{GameTime, ToggleTime},
};

use super::ClientMode;

pub struct ExplorerPlugin {
    pub config: BodiesConfig,
}

impl Plugin for ExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone()).add_systems(
            OnEnter(ClientMode::Explorer),
            start_explorer.in_set(SystemInitSet),
        );
    }
}

fn start_explorer(
    mut app_state: ResMut<NextState<AppState>>,
    mut time: Option<ResMut<GameTime>>,
    mut toggle_time: Option<ResMut<ToggleTime>>,
) {
    app_state.set(AppState::Loaded);
    if let Some(t) = time.as_mut() {
        t.0 = 0.
    }
    if let Some(t) = toggle_time.as_mut() {
        t.0 = true
    }
}
