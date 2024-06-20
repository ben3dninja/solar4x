use std::{
    fs::File,
    io::{Read, Result, Write},
    path::Path,
};

use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Keymap {
    pub tree: TreeViewKeymap,
    pub search: SearchViewKeymap,
    pub info: InfoViewKeymap,
}

impl Keymap {
    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        toml::from_str(&buf).map_err(std::io::Error::other)
    }

    pub fn write_to_file(&self, path: impl AsRef<Path>, overwrite: bool) -> Result<()> {
        let mut file = if overwrite {
            File::create(path)?
        } else {
            File::create_new(path)?
        };
        file.write_all(
            toml::to_string_pretty(self)
                .map_err(std::io::Error::other)?
                .as_bytes(),
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct TreeViewKeymap {
    pub select_next: KeyCode,
    pub select_previous: KeyCode,
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub display_info: KeyCode,
    pub toggle_expand: KeyCode,
    pub map_offset_up: KeyCode,
    pub map_offset_down: KeyCode,
    pub map_offset_left: KeyCode,
    pub map_offset_right: KeyCode,
    pub map_offset_reset: KeyCode,
    pub enter_search: KeyCode,
    pub focus: KeyCode,
    pub autoscale: KeyCode,
    pub quit: KeyCode,
    pub speed_up: KeyCode,
    pub slow_down: KeyCode,
    pub toggle_time: KeyCode,
}

#[derive(Serialize, Deserialize)]
pub struct SearchViewKeymap {
    pub move_cursor_right: KeyCode,
    pub move_cursor_left: KeyCode,
    pub select_next: KeyCode,
    pub select_previous: KeyCode,
    pub leave_search: KeyCode,
    pub validate_search: KeyCode,
    pub delete_char: KeyCode,
}

#[derive(Serialize, Deserialize)]
pub struct InfoViewKeymap {
    pub leave_info: KeyCode,
}

impl Default for TreeViewKeymap {
    fn default() -> Self {
        use KeyCode::*;
        let (w, a) = ('w', 'a');
        #[cfg(feature = "azerty")]
        let (w, a) = ('z', 'q');

        Self {
            select_next: Down,
            select_previous: Up,
            zoom_in: Char('+'),
            zoom_out: Char('-'),
            display_info: Char('i'),
            toggle_expand: Char(' '),
            map_offset_up: Char(w),
            map_offset_down: Char('s'),
            map_offset_left: Char(a),
            map_offset_right: Char('d'),
            map_offset_reset: Char('0'),
            enter_search: Char('/'),
            focus: Char('f'),
            autoscale: Char('x'),
            quit: Esc,
            speed_up: Char('>'),
            slow_down: Char('<'),
            toggle_time: Char('t'),
        }
    }
}

impl Default for SearchViewKeymap {
    fn default() -> Self {
        use KeyCode::*;
        Self {
            move_cursor_right: Right,
            move_cursor_left: Left,
            select_next: Down,
            select_previous: Up,
            leave_search: Esc,
            validate_search: Enter,
            delete_char: Backspace,
        }
    }
}

impl Default for InfoViewKeymap {
    fn default() -> Self {
        use KeyCode::*;
        Self {
            leave_info: Char('i'),
        }
    }
}
