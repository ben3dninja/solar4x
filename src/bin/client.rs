use std::env;

use bevy::app::App;
use rust_space_trading::{
    bodies::body_data::BodyType,
    client_plugin::ClientPlugin,
    core_plugin::BodiesConfig,
    engine_plugin::EnginePlugin,
    ui_plugin::{space_map_plugin::SpaceMapPlugin, TuiPlugin},
    utils::args::get_keymap,
};

fn main() {
    #[allow(unused_variables)]
    let explorer_bodies_config = BodiesConfig::SmallestBodyType(BodyType::Moon);
    #[cfg(feature = "asteroids")]
    let explorer_bodies_config = BodiesConfig::SmallestBodyType(BodyType::Comet);
    App::new()
        .add_plugins((
            ClientPlugin {
                singleplayer_bodies_config: explorer_bodies_config,
                ..Default::default()
            },
            bevy::app::ScheduleRunnerPlugin::default(),
            EnginePlugin,
            // GravityPlugin,
            TuiPlugin {
                keymap: get_keymap(env::args()).unwrap(),
                ..Default::default()
            },
            SpaceMapPlugin,
            rust_space_trading::ui_plugin::gui_plugin::GuiPlugin,
        ))
        .run();
}
