use std::error::Error;

use rust_space_trading::standalone::Standalone;

use crate::app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let body_type = BodyType::Moon;
    // #[cfg(feature = "asteroids")]
    let body_type = BodyType::Asteroid;
    let res = Standalone::new(body_type)?.run();

    res
}
