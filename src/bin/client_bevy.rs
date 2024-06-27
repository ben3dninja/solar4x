use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};

use bevy::app::App;
use rust_space_trading::{
    client_plugin::{ClientNetworkInfo, ClientPlugin},
    engine_plugin::EnginePlugin,
    input_plugin::InputPlugin,
    tui_plugin::{
        search_plugin::SearchPlugin, space_map_plugin::SpaceMapPlugin, tree_plugin::TreePlugin,
        TuiPlugin,
    },
    utils::args::get_keymap,
};

fn main() {
    App::new()
        .add_plugins((
            ClientPlugin(ClientNetworkInfo {
                server_address: (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6000),
                client_address: (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            }),
            EnginePlugin,
            InputPlugin {
                keymap: get_keymap(env::args()).unwrap(),
            },
            TuiPlugin,
            TreePlugin,
            SpaceMapPlugin,
            SearchPlugin,
        ))
        .run();
}
