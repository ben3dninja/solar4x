use std::env;

use bevy::app::{App, ScheduleRunnerPlugin};
use rust_space_trading::{
    bodies::body_data::BodyType,
    client_plugin::ClientPlugin,
    core_plugin::BodiesConfig,
    engine_plugin::EnginePlugin,
    tui_plugin::{
        // search_plugin::SearchPlugin,
        space_map_plugin::SpaceMapPlugin,
        // tree_plugin::TreePlugin,
        TuiPlugin,
    },
    utils::args::get_keymap, // utils::args::get_keymap,
};

fn main() {
    #[allow(unused_variables)]
    let explorer_bodies_config = BodiesConfig::SmallestBodyType(BodyType::Moon);
    #[cfg(feature = "asteroids")]
    let explorer_bodies_config = BodiesConfig::SmallestBodyType(BodyType::Comet);
    App::new()
        .add_plugins((
            ClientPlugin {
                explorer_bodies_config,
                ..Default::default()
            },
            ScheduleRunnerPlugin::default(),
            EnginePlugin,
            // GravityPlugin,
            // InputPlugin {
            //     keymap: get_keymap(env::args()).unwrap(),
            // },
            TuiPlugin {
                keymap: get_keymap(env::args()).unwrap(),
                ..Default::default()
            },
            // TreePlugin,
            SpaceMapPlugin,
            // SearchPlugin,
            // InfoPlugin,
            // GuiPlugin,
        ))
        .run();
}
