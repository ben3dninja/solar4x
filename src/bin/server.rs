use std::net::{IpAddr, Ipv4Addr};

use bevy::app::{App, ScheduleRunnerPlugin};
use rust_space_trading::{
    core_plugin::BodiesConfig,
    engine_plugin::EnginePlugin,
    server_plugin::{ServerNetworkInfo, ServerPlugin},
};

fn main() {
    App::new()
        .add_plugins((
            ServerPlugin {
                server_address: ServerNetworkInfo(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6000),
                config: BodiesConfig::default(),
            },
            EnginePlugin,
            ScheduleRunnerPlugin::default(),
        ))
        .run();
}
