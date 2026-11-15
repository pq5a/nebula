//! The saved theme. There are eight built-in themes and one custom theme, made
//! from two colours the user picks. The page (src/chrome.js) builds every palette.
//! Rust only checks the choice, saves it, and gives the window its background
//! colour, so the window matches the page before Discord has painted.

use std::{fs, io, path::{Path, PathBuf}};

use serde_json::{json, Value};
use wry::RGBA;

// Built-in themes: id, and the background shown before Discord paints.
const THEMES: [(&str, RGBA); 8] = [
    ("nebula", (0x0d, 0x0a, 0x1f, 255)),
    ("midnight", (0x05, 0x08, 0x0f, 255)),
    ("ember", (0x14, 0x0a, 0x08, 255)),
    ("aurora", (0x07, 0x13, 0x0f, 255)),
    ("rose", (0x17, 0x0a, 0x12, 255)),
    ("forest", (0x09, 0x12, 0x0a, 255)),
    ("solar", (0x15, 0x11, 0x0a, 255)),
    ("graphite", (0x0b, 0x0c, 0x0f, 255)),
];

pub const CUSTOM: &str = "custom";
const DEFAULT: &str = "nebula";
const THEME_FILE: &str = "theme.txt";
const CUSTOM_FILE: &str = "custom-theme.txt";

/// The two colors of a custom theme, as lower-case "#rrggbb".
#[derive(Clone, Debug, PartialEq)]
pub struct Custom {
    pub bg: String,
    pub accent: String,
}

/// Returns the built-in theme with this id, if there is one.
pub fn known_theme(id: &str) -> Option<&'static str> {
    THEMES.iter().map(|(t, _)| *t).find(|t| *t == id)
}

fn theme_path(data_dir: &Path) -> PathBuf {
    data_dir.join(THEME_FILE)
}

fn custom_path(data_dir: &Path) -> PathBuf {
    data_dir.join(CUSTOM_FILE)
}

/// The saved theme id. A missing or damaged file gives the default. "custom"
/// is only returned when a custom theme has been saved.
pub fn saved_theme(data_dir: &Path) -> String {
    let id = fs::read_to_string(theme_path(data_dir))
        .ok()
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();
    if known_theme(&id).is_some() || (id == CUSTOM && saved_custom(data_dir).is_some()) {
        id
    } else {
        DEFAULT.to_owned()
    }
}

/// Whether the page may switch to this theme. Built-in themes always, custom only once saved.
pub fn can_select(id: &str, data_dir: &Path) -> bool {
    known_theme(id).is_some() || (id == CUSTOM && saved_custom(data_dir).is_some())
}

pub fn save_theme(data_dir: &Path, id: &str) -> io::Result<()> {
    fs::write(theme_path(data_dir), id)
}

/// Accepts a "#rrggbb" color in any case and returns it in lower case.
fn normalize_hex(text: &str) -> Option<String> {
    let rest = text.strip_prefix('#')?;
    if rest.len() == 6 && rest.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(format!("#{}", rest.to_ascii_lowercase()))
    } else {
        None
    }
}

fn hex_to_rgba(hex: &str) -> Option<RGBA> {
    let n = u32::from_str_radix(hex.strip_prefix('#')?, 16).ok()?;
    Some((((n >> 16) & 0xff) as u8, ((n >> 8) & 0xff) as u8, (n & 0xff) as u8, 255))
}

/// Reads a custom theme from JSON such as {"bg":"#0d0a1f","accent":"#a78bfa"}.
/// Anything other than two valid hex colors is rejected.
pub fn parse_custom(text: &str) -> Option<Custom> {
    let v: Value = serde_json::from_str(text).ok()?;
    Some(Custom {
        bg: normalize_hex(v.get("bg")?.as_str()?)?,
        accent: normalize_hex(v.get("accent")?.as_str()?)?,
    })
}

pub fn saved_custom(data_dir: &Path) -> Option<Custom> {
    parse_custom(&fs::read_to_string(custom_path(data_dir)).ok()?)
}

pub fn save_custom(data_dir: &Path, custom: &Custom) -> io::Result<()> {
    let text = json!({ "bg": custom.bg, "accent": custom.accent }).to_string();
    fs::write(custom_path(data_dir), text)
}

/// The custom colors as a JavaScript value: an object, or null if there are none.
/// Built with serde_json, so the output is always a valid literal.
pub fn custom_json(custom: Option<&Custom>) -> String {
    match custom {
        Some(c) => json!({ "bg": c.bg, "accent": c.accent }).to_string(),
        None => "null".to_owned(),
    }
}

/// The window background for a theme. For the custom theme it is the picked
/// background color, which the page uses unchanged.
pub fn window_background(theme: &str, custom: Option<&Custom>) -> RGBA {
    if theme == CUSTOM {
        if let Some(rgba) = custom.and_then(|c| hex_to_rgba(&c.bg)) {
            return rgba;
        }
    }
    THEMES
        .iter()
        .find(|(t, _)| *t == theme)
        .map_or(THEMES[0].1, |(_, color)| *color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_built_in_themes_are_known() {
        assert_eq!(known_theme("midnight"), Some("midnight"));
        assert_eq!(known_theme("graphite"), Some("graphite"));
        assert_eq!(known_theme("custom"), None);
        assert_eq!(known_theme("../evil"), None);
        assert_eq!(known_theme("nebula\";alert(1)//"), None);
    }

    #[test]
    fn custom_colors_must_be_hex() {
        assert_eq!(
            parse_custom(r##"{"bg":"#0D0A1F","accent":"#a78bfa"}"##),
            Some(Custom { bg: "#0d0a1f".into(), accent: "#a78bfa".into() })
        );
        assert_eq!(parse_custom(r##"{"bg":"#0d0a1f"}"##), None);
        assert_eq!(parse_custom(r##"{"bg":"red","accent":"#a78bfa"}"##), None);
        assert_eq!(parse_custom(r##"{"bg":"#0d0a1","accent":"#a78bfa"}"##), None);
        assert_eq!(parse_custom(r##"{"bg":"#0d0a1f\";alert(1)//","accent":"#a78bfa"}"##), None);
        assert_eq!(parse_custom("not json"), None);
    }

    #[test]
    fn custom_json_is_always_a_valid_literal() {
        assert_eq!(custom_json(None), "null");
        let c = Custom { bg: "#000000".into(), accent: "#ffffff".into() };
        assert_eq!(custom_json(Some(&c)), r##"{"accent":"#ffffff","bg":"#000000"}"##);
    }

    #[test]
    fn window_background_follows_the_theme() {
        let c = Custom { bg: "#102030".into(), accent: "#ffffff".into() };
        assert_eq!(window_background("custom", Some(&c)), (0x10, 0x20, 0x30, 255));
        // Without a custom theme, "custom" falls back to the default background.
        assert_eq!(window_background("custom", None), THEMES[0].1);
        assert_eq!(window_background("midnight", Some(&c)), THEMES[1].1);
    }
}
