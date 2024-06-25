use bevy::app::App;
use rust_space_trading::{
    app::body_data::BodyType,
    core_plugin::CorePlugin,
    engine_plugin::EnginePlugin,
    input_plugin::InputPlugin,
    ui_plugin::{space_map_plugin::SpaceMapPlugin, tree_plugin::TreePlugin, UiPlugin},
};

fn main() {
    App::new()
        .add_plugins((
            CorePlugin {
                smallest_body_type: BodyType::Moon,
            },
            EnginePlugin,
            InputPlugin,
            UiPlugin,
            TreePlugin,
            SpaceMapPlugin,
        ))
        .run();
}
