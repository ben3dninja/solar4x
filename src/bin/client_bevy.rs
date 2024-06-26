use std::net::{IpAddr, Ipv4Addr};

use bevy::app::App;
use rust_space_trading::{
    client_plugin::{ClientNetworkInfo, ClientPlugin},
    engine_plugin::EnginePlugin,
    input_plugin::InputPlugin,
    ui_plugin::{
        search_plugin::SearchPlugin, space_map_plugin::SpaceMapPlugin, tree_plugin::TreePlugin,
        UiPlugin,
    },
};

fn main() {
    App::new()
        .add_plugins((
            ClientPlugin(ClientNetworkInfo {
                server_address: (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6000),
                client_address: (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            }),
            EnginePlugin,
            InputPlugin,
            UiPlugin,
            TreePlugin,
            SpaceMapPlugin,
            SearchPlugin,
        ))
        .run();
}
