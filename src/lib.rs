pub mod bodies;
pub mod client_plugin;
pub mod core_plugin;
pub mod engine_plugin;
pub mod influence;
pub mod keyboard;
pub mod leapfrog;
pub mod main_game;
pub mod network;
pub mod server_plugin;
pub mod spaceship;
pub mod ui_plugin;
pub mod utils;

const MAX_ID_LENGTH: usize = 32;

/// Number of server updates (ticks) per real time second
const TPS: f64 = 1.;

/// Number of simulation updates (simticks) per real time second
const STPS: f64 = 64.;

/// Game time that is added at each client update (in days, multiplied by simstepsize)
const GAMETIME_PER_SIMTICK: f64 = 1e-2;

const SECONDS_PER_DAY: f64 = 86400.;
