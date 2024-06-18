use std::{env, error::Error};

use rust_space_trading::app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let mut keymap = Keymap::default();
    if let Some(command) = args.nth(1) {
        match &command[..] {
            "--writekeymap" => {
                Keymap::default()
                    .write_to_file(args.next().ok_or("Expected output file path")?, false)?;
            }
            "-k" => {
                keymap = Keymap::from_toml_file(args.next().ok_or("Expected keymap file path")?)?;
            }
            _ => {}
        }
    }
    #[cfg(not(feature = "asteroids"))]
    let res = App::new_moons_client()?.with_keymap(keymap).run();
    #[cfg(feature = "asteroids")]
    let res = App::new_complete_client()?.with_keymap(keymap).run();

    res
}
