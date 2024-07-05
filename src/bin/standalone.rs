// use std::env;

use std::env;

use bevy::{
    app::{App, ScheduleRunnerPlugin},
    prelude::default,
};
use rust_space_trading::{
    bodies::body_data::BodyType,
    core_plugin::BodiesConfig,
    engine_plugin::EnginePlugin,
    // gui_plugin::GuiPlugin,
    // input_plugin::InputPlugin,
    standalone_plugin::StandalonePlugin,
    tui_plugin::TuiPlugin,
    utils::args::get_keymap,
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
            // },
            TuiPlugin {
                keymap: get_keymap(env::args()).unwrap(),
                ..default()
            },
            // GuiPlugin,
        ))
        .run();
}
