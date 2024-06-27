use std::{env::Args, error::Error};

use crate::keyboard::Keymap;

pub fn get_keymap(mut args: Args) -> Result<Keymap, Box<dyn Error>> {
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
    Ok(keymap)
}
