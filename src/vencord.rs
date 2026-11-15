//! Loads Vencord and keeps it current.
//!
//! Vencord publishes its browser build as assets of the GitHub release tagged
//! `devbuild`, which is rebuilt on each Vencord commit. GitHub reports a SHA-256
//! digest for every asset, and a download is only used when it matches.
//!
//! Layout under `%APPDATA%\Nebula\vencord`:
//! - `current.txt`: name of the active version directory
//! - `versions/<digest>/`: the three files plus `digests.json`
//!
//! Updates are staged and only become active by rewriting `current.txt`, so a
//! failed or interrupted download never leaves a broken version active. A
//! downloaded update takes effect on the next launch, because the page script
//! is built once at startup.

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const RELEASE_API: &str =
    "https://api.github.com/repos/Vendicated/Vencord/releases/tags/devbuild";
const DOWNLOAD_BASE: &str = "https://github.com/Vendicated/Vencord/releases/download/devbuild/";
const USER_AGENT: &str = "Nebula";
const TIMEOUT: Duration = Duration::from_secs(30);
const MAX_JSON_BYTES: u64 = 1024 * 1024;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;

const JS: &str = "browser.js";
const CSS: &str = "browser.css";
const LEGAL: &str = "browser.js.LEGAL.txt";
const FILES: [&str; 3] = [JS, CSS, LEGAL];

// Appended to the Vencord stylesheet. Button hover/press fades are removed so
// clicks and hovers respond immediately instead of animating in.
const NEBULA_CSS: &str = r#"
button, [role="button"], a[role="button"] { transition: none !important; }
"#;

// Top-level declaration in the Vencord bundle that must become a global.
const BUNDLE_GLOBAL_DECL: &str = "var Vencord=";

pub struct Bundle {
    pub js: String,
    pub css: String,
}

/// The copy compiled into the executable. Used when no verified download
/// exists yet, so Nebula works on first run and offline.
pub fn embedded() -> Bundle {
    Bundle {
        js: include_str!("../vendor/vencord.browser.js").to_owned(),
        css: include_str!("../vendor/vencord.browser.css").to_owned(),
    }
}

/// Returns the verified cached bundle, or `None` if there is none or any file
/// no longer matches its recorded digest.
pub fn load_cached(data_dir: &Path) -> Option<Bundle> {
    let root = data_dir.join("vencord");
    let version = fs::read_to_string(root.join("current.txt")).ok()?;
    let version = version.trim();
    if version.is_empty() || !version.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let dir = root.join("versions").join(version);
    let digests: Value = serde_json::from_slice(&fs::read(dir.join("digests.json")).ok()?).ok()?;

    let read = |name: &str| -> Option<String> {
        let bytes = fs::read(dir.join(name)).ok()?;
        (digests.get(name)?.as_str()? == sha256_hex(&bytes))
            .then(|| String::from_utf8(bytes).ok())
            .flatten()
    };
    Some(Bundle {
        js: read(JS)?,
        css: read(CSS)?,
    })
}

/// Checks GitHub for a newer Vencord build and stores it if one is found.
/// Returns `Ok(true)` if a new version was installed.
pub fn update(data_dir: &Path) -> Result<bool, String> {
    let tls = native_tls::TlsConnector::new().map_err(|e| format!("TLS unavailable: {e}"))?;
    let agent = ureq::AgentBuilder::new()
        .timeout(TIMEOUT)
        .tls_connector(Arc::new(tls))
        .build();

    let release: Value = serde_json::from_reader(
        agent
            .get(RELEASE_API)
            .set("User-Agent", USER_AGENT)
            .set("Accept", "application/vnd.github+json")
            .call()
            .map_err(|e| format!("release lookup failed: {e}"))?
            .into_reader()
            .take(MAX_JSON_BYTES),
    )
    .map_err(|e| format!("release lookup returned invalid JSON: {e}"))?;

    let assets = release["assets"]
        .as_array()
        .ok_or("release has no asset list")?;
    let mut expected = Vec::with_capacity(FILES.len());
    for name in FILES {
        let digest = assets
            .iter()
            .find(|a| a["name"].as_str() == Some(name))
            .and_then(|a| a["digest"].as_str())
            .and_then(|d| d.strip_prefix("sha256:"))
            .ok_or_else(|| format!("no sha256 digest published for {name}"))?;
        expected.push((name, digest.to_owned()));
    }

    // The digest of browser.js identifies the build.
    let version = expected[0].1.clone();
    let root = data_dir.join("vencord");
    if fs::read_to_string(root.join("current.txt")).ok().as_deref() == Some(version.as_str())
        && load_cached(data_dir).is_some()
    {
        return Ok(false);
    }

    let staging = root.join("versions").join(format!(".staging-{version}"));
    let _ = fs::remove_dir_all(&staging);
    let result = stage(&agent, &expected, &staging)
        .and_then(|()| activate(&root, &staging, &version));
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result.map(|()| true)
}

fn stage(agent: &ureq::Agent, expected: &[(&str, String)], staging: &Path) -> Result<(), String> {
    fs::create_dir_all(staging).map_err(|e| format!("cannot create staging directory: {e}"))?;
    let mut digests = Map::new();
    for (name, want) in expected {
        let bytes = download(agent, &format!("{DOWNLOAD_BASE}{name}"))?;
        let got = sha256_hex(&bytes);
        if got != *want {
            return Err(format!("{name} failed digest check (expected {want}, got {got})"));
        }
        fs::write(staging.join(name), &bytes).map_err(|e| format!("cannot write {name}: {e}"))?;
        digests.insert((*name).to_owned(), Value::String(got));
    }
    let json = serde_json::to_vec(&Value::Object(digests)).map_err(|e| e.to_string())?;
    fs::write(staging.join("digests.json"), json).map_err(|e| format!("cannot write digests: {e}"))
}

fn activate(root: &Path, staging: &Path, version: &str) -> Result<(), String> {
    let versions = root.join("versions");
    let final_dir = versions.join(version);
    let _ = fs::remove_dir_all(&final_dir);
    fs::rename(staging, &final_dir).map_err(|e| format!("cannot install version: {e}"))?;

    // Write the pointer to a temp file first, then rename it into place.
    let pointer = root.join("current.txt");
    let tmp = root.join("current.txt.tmp");
    let mut file = fs::File::create(&tmp).map_err(|e| e.to_string())?;
    file.write_all(version.as_bytes()).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&tmp, &pointer).map_err(|e| format!("cannot activate version: {e}"))?;

    // Remove older versions, keeping the active one.
    if let Ok(entries) = fs::read_dir(&versions) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy() != version {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }
    Ok(())
}

fn download(agent: &ureq::Agent, url: &str) -> Result<Vec<u8>, String> {
    let resp = agent
        .get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("download failed ({url}): {e}"))?;
    let mut buf = Vec::new();
    resp.into_reader()
        .take(MAX_FILE_BYTES)
        .read_to_end(&mut buf)
        .map_err(|e| format!("download interrupted ({url}): {e}"))?;
    Ok(buf)
}

/// Runs `update` on a background thread so startup is not delayed.
pub fn spawn_update(data_dir: PathBuf) {
    std::thread::spawn(move || {
        if let Err(e) = update(&data_dir) {
            eprintln!("Vencord update skipped: {e}");
        }
    });
}

/// Builds the script that runs before each Discord page. It adds the stylesheet
/// and starts the Vencord bundle, guarded to run only on discord.com.
pub fn init_script(bundle: &Bundle) -> Result<String, String> {
    let css = format!("{}\n{NEBULA_CSS}", bundle.css);
    let bundle_js = globalize(&bundle.js)?;
    Ok(format!(
        r#"(function () {{
  "use strict";
  var host = location.hostname;
  if (host !== "discord.com" && !host.endsWith(".discord.com")) return;
  if (window.__nebulaInjected) return;
  window.__nebulaInjected = true;

  var addCss = function () {{
    var style = document.createElement("style");
    style.id = "nebula-vencord-css";
    style.textContent = {css};
    (document.head || document.documentElement).appendChild(style);
  }};
  if (document.documentElement) addCss();
  else document.addEventListener("DOMContentLoaded", addCss, {{ once: true }});

{bundle_js}
}})();
"#,
        css = js_string(&css),
    ))
}

// Rewrites the bundle so its global `Vencord` is assigned to `window` instead
// of declared with `var`. A `var` inside a function wrapper would be
// function-scoped and never reach the page's global object.
fn globalize(js: &str) -> Result<String, String> {
    let pos = js
        .find(BUNDLE_GLOBAL_DECL)
        .ok_or("Vencord bundle has no `var Vencord=` declaration")?;
    if pos >= 512 || js[pos + BUNDLE_GLOBAL_DECL.len()..].contains(BUNDLE_GLOBAL_DECL) {
        return Err("unexpected layout of the Vencord bundle".into());
    }
    Ok(format!(
        "{}window.Vencord={}",
        &js[..pos],
        &js[pos + BUNDLE_GLOBAL_DECL.len()..]
    ))
}

// Escapes `s` as a double-quoted JavaScript string literal.
fn js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_bundle_builds_a_script() {
        let script = init_script(&embedded()).expect("embedded bundle must be valid");
        assert!(script.contains("window.Vencord="));
        assert!(script.contains("nebula-vencord-css"));
    }

    #[test]
    fn globalize_rejects_unexpected_layouts() {
        assert!(globalize("no declaration here").is_err());
        assert!(globalize(&format!("{}{}", "x".repeat(600), BUNDLE_GLOBAL_DECL)).is_err());
        assert!(globalize("var Vencord=1;var Vencord=2;").is_err());
    }

    #[test]
    fn js_string_escapes_quotes_and_newlines() {
        assert_eq!(js_string("a\"b\nc"), "\"a\\\"b\\nc\"");
    }

    // Needs network access: run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn live_update_installs_and_then_skips() {
        let dir = std::env::temp_dir().join(format!("nebula-live-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(update(&dir), Ok(true));
        let cached = load_cached(&dir).expect("downloaded bundle must verify");
        assert!(cached.js.contains("var Vencord=") || cached.js.contains("window.Vencord="));
        assert_eq!(update(&dir), Ok(false), "second run should find nothing new");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sha256_matches_known_value() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
