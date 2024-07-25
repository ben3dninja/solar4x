use std::env;

use bevy::app::App;
use rust_space_trading::{prelude::*, ui::gui::GuiPlugin, utils::args::get_keymap};

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
            TuiPlugin {
                keymap: get_keymap(env::args()).unwrap(),
                ..Default::default()
            },
            GuiPlugin,
        ))
        .run();
}
