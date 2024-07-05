use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{Read, Result, Write},
    path::Path,
};

use bevy::ecs::system::Resource;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{
    de::{value::StrDeserializer, IntoDeserializer, Visitor},
    Deserialize, Serialize,
};

#[derive(Resource, Default, Clone, Serialize, Deserialize)]
pub struct Keymap {
    pub explorer: ExplorerKeymap,
    pub start_menu: StartMenuKeymap,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ExplorerKeymap {
    pub tree: TreeViewKeymap,
    pub search: SearchViewKeymap,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartMenuKeymap {
    pub select_next: Key,
    pub select_previous: Key,
    pub quit: Key,
    pub validate: Key,
}

#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

const POSSIBLE_CODE_STRINGS: [&str; 26] = [
    "backspace",
    "enter",
    "left",
    "right",
    "up",
    "down",
    "home",
    "end",
    "pageup",
    "pagedown",
    "tab",
    "back_tab",
    "del",
    "ins",
    "null",
    "esc",
    "caps",
    "scroll",
    "numlock",
    "prscrn",
    "pause",
    "menu",
    "begin",
    "space",
    "<c>",
    "f<i>",
];

const POSSIBLE_CODES: [KeyCode; 24] = [
    KeyCode::Backspace,
    KeyCode::Enter,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Home,
    KeyCode::End,
    KeyCode::PageUp,
    KeyCode::PageDown,
    KeyCode::Tab,
    KeyCode::BackTab,
    KeyCode::Delete,
    KeyCode::Insert,
    KeyCode::Null,
    KeyCode::Esc,
    KeyCode::CapsLock,
    KeyCode::ScrollLock,
    KeyCode::NumLock,
    KeyCode::PrintScreen,
    KeyCode::Pause,
    KeyCode::Menu,
    KeyCode::KeypadBegin,
    KeyCode::Char(' '),
];

pub fn keycode_string_pairing() -> impl IntoIterator<Item = (KeyCode, &'static str)> {
    POSSIBLE_CODES.into_iter().zip(
        POSSIBLE_CODE_STRINGS
            .into_iter()
            .skip_while(|s| s.contains('<') && s.contains('>')),
    )
}

fn keycode_string_map() -> HashMap<KeyCode, &'static str> {
    keycode_string_pairing().into_iter().collect()
}

fn string_keycode_map() -> HashMap<&'static str, KeyCode> {
    keycode_string_pairing()
        .into_iter()
        .map(|(a, b)| (b, a))
        .collect()
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            s.push_str("C ");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            s.push_str("A ");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            s.push_str("S ")
        }
        s.push_str(&match self.code {
            KeyCode::Char(c) if c != ' ' => format!("{c}"),
            KeyCode::F(i) => format!("f{i}"),
            KeyCode::Media(m) => format!("{:?}", m),
            KeyCode::Modifier(m) => format!("{:?}", m),
            code => keycode_string_map()
                .get(&code)
                .cloned()
                .unwrap_or("unknown")
                .to_owned(),
        });
        f.write_str(&s)
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Key {
    pub fn from_str_unchecked(s: &str) -> Self {
        let d: StrDeserializer<serde::de::value::Error> = s.into_deserializer();
        Key::deserialize(d).unwrap()
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        event.code == self.code && event.modifiers == self.modifiers
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct KeyVisitor;
        impl<'d> Visitor<'d> for KeyVisitor {
            type Value = Key;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a keycode with optionnal modifiers")
            }
            fn visit_str<E>(self, v: &str) -> std::prelude::v1::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut modifiers = KeyModifiers::NONE;
                let mut code = None;
                for s in v.split(' ') {
                    match s {
                        "A" => modifiers |= KeyModifiers::ALT,
                        "S" => modifiers |= KeyModifiers::SHIFT,
                        "C" => modifiers |= KeyModifiers::CONTROL,
                        s => {
                            let chars: Vec<_> = s.chars().collect();
                            code = match chars.len() {
                                1 => Some(KeyCode::Char(chars[0])),
                                2 if chars[0] == 'f' => Some(KeyCode::F(
                                    chars[1]
                                        .to_string()
                                        .parse::<u8>()
                                        .map_err(|_| E::custom(format!("Unknown fkey : {s}")))?,
                                )),
                                _ => {
                                    Some(*string_keycode_map().get(s).ok_or(E::custom(format!(
                                        "Invalid key code {s}, expected one of : {:#?}",
                                        POSSIBLE_CODE_STRINGS
                                    )))?)
                                }
                            };
                            break;
                        }
                    }
                }
                Ok(Key {
                    code: code.ok_or(E::missing_field("code"))?,
                    modifiers,
                })
            }
        }
        deserializer.deserialize_str(KeyVisitor)
    }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TreeViewKeymap {
    pub select_next: Key,
    pub select_previous: Key,
    pub zoom_in: Key,
    pub zoom_out: Key,
    pub toggle_expand: Key,
    pub map_offset_up: Key,
    pub map_offset_down: Key,
    pub map_offset_left: Key,
    pub map_offset_right: Key,
    pub map_offset_reset: Key,
    pub enter_search: Key,
    pub focus: Key,
    pub autoscale: Key,
    pub quit: Key,
    pub speed_up: Key,
    pub slow_down: Key,
    pub toggle_time: Key,
    pub toggle_info: Key,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchViewKeymap {
    pub move_cursor_right: Key,
    pub move_cursor_left: Key,
    pub select_next: Key,
    pub select_previous: Key,
    pub leave_search: Key,
    pub validate_search: Key,
    pub delete_char: Key,
}

impl Default for TreeViewKeymap {
    fn default() -> Self {
        let (w, a) = ("w", "a");
        #[cfg(feature = "azerty")]
        let (w, a) = ("z", "q");
        Self {
            select_next: Key::from_str_unchecked("down"),
            select_previous: Key::from_str_unchecked("up"),
            zoom_in: Key::from_str_unchecked("+"),
            zoom_out: Key::from_str_unchecked("-"),
            toggle_info: Key::from_str_unchecked("i"),
            toggle_expand: Key::from_str_unchecked("space"),
            map_offset_up: Key::from_str_unchecked(w),
            map_offset_down: Key::from_str_unchecked("s"),
            map_offset_left: Key::from_str_unchecked(a),
            map_offset_right: Key::from_str_unchecked("d"),
            map_offset_reset: Key::from_str_unchecked("0"),
            enter_search: Key::from_str_unchecked("/"),
            focus: Key::from_str_unchecked("f"),
            autoscale: Key::from_str_unchecked("x"),
            quit: Key::from_str_unchecked("esc"),
            speed_up: Key::from_str_unchecked(">"),
            slow_down: Key::from_str_unchecked("<"),
            toggle_time: Key::from_str_unchecked("t"),
        }
    }
}

impl Default for SearchViewKeymap {
    fn default() -> Self {
        Self {
            move_cursor_right: Key::from_str_unchecked("right"),
            move_cursor_left: Key::from_str_unchecked("left"),
            select_next: Key::from_str_unchecked("down"),
            select_previous: Key::from_str_unchecked("up"),
            leave_search: Key::from_str_unchecked("esc"),
            validate_search: Key::from_str_unchecked("enter"),
            delete_char: Key::from_str_unchecked("backspace"),
        }
    }
}

impl Default for StartMenuKeymap {
    fn default() -> Self {
        Self {
            select_next: Key::from_str_unchecked("down"),
            select_previous: Key::from_str_unchecked("up"),
            quit: Key::from_str_unchecked("esc"),
            validate: Key::from_str_unchecked("enter"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ExplorerKeymap;

    #[test]
    fn test_default_keymap() {
        dbg!(ExplorerKeymap::default());
    }
}
