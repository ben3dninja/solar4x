use std::error::Error;

use app::App;
mod app;
mod engine;
mod ui;
mod utils;
fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(not(feature = "asteroids"))]
    let res = App::new_moons_client()?.run();
    #[cfg(feature = "asteroids")]
    let res = App::new_complete_client()?.run();

    res
}
