use std::env;

use bevy::app::App;
use rust_space_trading::{
    bodies::{bodies_config::BodiesConfig, body_data::BodyType},
    client_plugin::ClientPlugin,
    ui_plugin::TuiPlugin,
    utils::args::get_keymap,
};

fn main() {
    #[allow(unused_variables)]
    let singleplayer_bodies_config = BodiesConfig::SmallestBodyType(BodyType::Moon);
    #[cfg(feature = "asteroids")]
    let singleplayer_bodies_config = BodiesConfig::SmallestBodyType(BodyType::Comet);
    App::new()
        .add_plugins((
            ClientPlugin {
                singleplayer_bodies_config,
                ..Default::default()
            },
            bevy::app::ScheduleRunnerPlugin::default(),
            // GravityPlugin,
            TuiPlugin {
                keymap: get_keymap(env::args()).unwrap(),
                ..Default::default()
            },
            rust_space_trading::ui_plugin::gui_plugin::GuiPlugin,
        ))
        .run();
}
