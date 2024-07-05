// use std::env;

use bevy::app::{App, ScheduleRunnerPlugin};
use rust_space_trading::{
    bodies::body_data::BodyType,
    core_plugin::BodiesConfig,
    engine_plugin::EnginePlugin,
    // gui_plugin::GuiPlugin,
    // input_plugin::InputPlugin,
    standalone_plugin::StandalonePlugin,
    tui_plugin::{
        // info_plugin::InfoPlugin, search_plugin::SearchPlugin,
        space_map_plugin::SpaceMapPlugin,
        TuiPlugin,
        // tree_plugin::TreePlugin, TuiPlugin,
    },
};

fn main() {
    let config = BodiesConfig::SmallestBodyType(BodyType::Moon);
    #[cfg(feature = "asteroids")]
    let config = BodiesConfig::SmallestBodyType(BodyType::Comet);
    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::default(),
            StandalonePlugin(config),
            EnginePlugin,
            // GravityPlugin,
            // InputPlugin {
            //     keymap: get_keymap(env::args()).unwrap(),
            // },
            TuiPlugin::default(),
            // TreePlugin,
            SpaceMapPlugin,
            // SearchPlugin,
            // InfoPlugin,
            // GuiPlugin,
        ))
        .run();
}
