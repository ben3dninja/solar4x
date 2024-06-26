use bevy::app::App;
use rust_space_trading::{
    app::body_data::BodyType,
    core_plugin::BodiesConfig,
    engine_plugin::EnginePlugin,
    input_plugin::InputPlugin,
    standalone_plugin::StandalonePlugin,
    ui_plugin::{
        search_plugin::SearchPlugin, space_map_plugin::SpaceMapPlugin, tree_plugin::TreePlugin,
        UiPlugin,
    },
};

fn main() {
    App::new()
        .add_plugins((
            StandalonePlugin(BodiesConfig::SmallestBodyType(BodyType::Moon)),
            EnginePlugin,
            InputPlugin,
            UiPlugin,
            TreePlugin,
            SpaceMapPlugin,
            SearchPlugin,
        ))
        .run();
}
