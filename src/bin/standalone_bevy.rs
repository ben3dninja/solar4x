use std::env;

use bevy::app::App;
use rust_space_trading::{
    bodies::body_data::BodyType,
    core_plugin::BodiesConfig,
    engine_plugin::EnginePlugin,
    gui_plugin::GuiPlugin,
    input_plugin::InputPlugin,
    standalone_plugin::StandalonePlugin,
    tui_plugin::{
        search_plugin::SearchPlugin, space_map_plugin::SpaceMapPlugin, tree_plugin::TreePlugin,
        TuiPlugin,
    },
    utils::args::get_keymap,
};

fn main() {
    let config = BodiesConfig::SmallestBodyType(BodyType::Moon);
    #[cfg(feature = "asteroids")]
    let config = BodiesConfig::SmallestBodyType(BodyType::Comet);
    App::new()
        .add_plugins((
            StandalonePlugin(config),
            EnginePlugin,
            InputPlugin {
                keymap: get_keymap(env::args()).unwrap(),
            },
            TuiPlugin,
            TreePlugin,
            SpaceMapPlugin,
            SearchPlugin,
            GuiPlugin,
        ))
        .run();
}
