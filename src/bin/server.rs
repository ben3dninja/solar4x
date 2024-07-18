use std::net::{IpAddr, Ipv4Addr};

use bevy::app::{App, ScheduleRunnerPlugin};
use rust_space_trading::{
    bodies::bodies_config::BodiesConfig,
    orbit::OrbitPlugin,
    server::{ServerNetworkInfo, ServerPlugin},
};

fn main() {
    App::new()
        .add_plugins((
            ServerPlugin {
                server_address: ServerNetworkInfo(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6000),
                config: BodiesConfig::default(),
            },
            OrbitPlugin,
            ScheduleRunnerPlugin::default(),
        ))
        .run();
}
