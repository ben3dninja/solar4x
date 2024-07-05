use std::{env::Args, error::Error};

use crate::keyboard::ExplorerKeymap;

pub fn get_keymap(mut args: Args) -> Result<ExplorerKeymap, Box<dyn Error>> {
    let mut keymap = ExplorerKeymap::default();

    if let Some(command) = args.nth(1) {
        match &command[..] {
            "--writekeymap" => {
                ExplorerKeymap::default()
                    .write_to_file(args.next().ok_or("Expected output file path")?, false)?;
            }
            "-k" => {
                keymap = ExplorerKeymap::from_toml_file(
                    args.next().ok_or("Expected keymap file path")?,
                )?;
            }
            _ => {}
        }
    }
    Ok(keymap)
}
