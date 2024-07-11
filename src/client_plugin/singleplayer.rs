use bevy::prelude::*;

use crate::{
    bodies::bodies_config::BodiesConfig,
    core_plugin::LoadingState,
    main_game::{
        trajectory::{dispatch_trajectories, handle_trajectory_event},
        GameStage,
    },
    utils::ecs::exit_on_error_if_app,
};

use super::ClientMode;

/// This plugin's role is to handle everything that is done by the client in singleplayer mode but not in multiplayer, like reading and dispatching trajectories
#[derive(Default)]
pub struct SingleplayerPlugin(pub BodiesConfig);

impl Plugin for SingleplayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone())
            .add_systems(
                OnEnter(GameStage::Action),
                dispatch_trajectories.pipe(exit_on_error_if_app),
            )
            .add_systems(
                Update,
                handle_trajectory_event
                    .pipe(exit_on_error_if_app)
                    .run_if(in_state(GameStage::Preparation)),
            )
            .add_systems(OnEnter(ClientMode::Singleplayer), start_singleplayer);
    }
}

fn start_singleplayer(mut loading_state: ResMut<NextState<LoadingState>>) {
    loading_state.set(LoadingState::Loading);
}
