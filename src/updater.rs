//! Checks GitHub for a newer Nebula release and, with the user's consent,
//! replaces the running executable with it.
//!
//! The check runs in the background once the window is up. If a newer release
//! exists, a Yes/No prompt is shown. On Yes, the release's `Nebula.exe` is
//! downloaded and checked against the SHA-256 digest GitHub publishes for it,
//! then staged in `%APPDATA%\Nebula\update`. The event loop swaps it in for the
//! running executable and starts the new one.

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use serde_json::Value;
use sha2::{Digest, Sha256};
use tao::event_loop::EventLoopProxy;

const RELEASE_API: &str = "https://api.github.com/repos/pq5a/nebula/releases/latest";
const ASSET_NAME: &str = "Nebula.exe";
const USER_AGENT: &str = "Nebula";
const TIMEOUT: Duration = Duration::from_secs(60);
const MAX_JSON_BYTES: u64 = 1024 * 1024;
const MAX_EXE_BYTES: u64 = 64 * 1024 * 1024;
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const STAGED_NAME: &str = "Nebula.exe";
// The previous executable, moved aside during an update. Deleted on next launch.
const OLD_NAME: &str = "Nebula.old.exe";

pub struct Release {
    pub version: String,
    pub url: String,
    pub sha256: String,
}

fn agent() -> Result<ureq::Agent, String> {
    let tls = native_tls::TlsConnector::new().map_err(|e| format!("TLS unavailable: {e}"))?;
    Ok(ureq::AgentBuilder::new()
        .timeout(TIMEOUT)
        .tls_connector(Arc::new(tls))
        .build())
}

fn get(agent: &ureq::Agent, url: &str, accept: &str) -> Result<ureq::Response, String> {
    agent
        .get(url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", accept)
        .call()
        .map_err(|e| format!("request failed ({url}): {e}"))
}

// Parses "v1.2.3" or "1.2.3". Anything else (pre-releases, partial versions) is None.
fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    let text = text.trim();
    let text = text.strip_prefix('v').unwrap_or(text);
    let parts: Vec<u64> = text
        .split('.')
        .map(|p| p.parse().ok())
        .collect::<Option<_>>()?;
    match parts[..] {
        [major, minor, patch] => Some((major, minor, patch)),
        _ => None,
    }
}

/// True only when both versions parse and `latest` is strictly newer.
pub fn is_newer(latest: &str, current: &str) -> bool {
    matches!(
        (parse_version(latest), parse_version(current)),
        (Some(l), Some(c)) if l > c
    )
}

/// Returns the latest release if it is newer than this build, otherwise `None`.
pub fn latest_release() -> Result<Option<Release>, String> {
    let agent = agent()?;
    let release: Value = serde_json::from_reader(
        get(&agent, RELEASE_API, "application/vnd.github+json")?
            .into_reader()
            .take(MAX_JSON_BYTES),
    )
    .map_err(|e| format!("release lookup returned invalid JSON: {e}"))?;

    let tag = release["tag_name"].as_str().ok_or("release has no tag")?;
    if !is_newer(tag, CURRENT_VERSION) {
        return Ok(None);
    }
    let asset = release["assets"]
        .as_array()
        .ok_or("release has no asset list")?
        .iter()
        .find(|a| a["name"].as_str() == Some(ASSET_NAME))
        .ok_or("release has no Nebula.exe asset")?;
    let url = asset["browser_download_url"]
        .as_str()
        .ok_or("asset has no download URL")?;
    let sha256 = asset["digest"]
        .as_str()
        .and_then(|d| d.strip_prefix("sha256:"))
        .ok_or("no sha256 digest published for Nebula.exe")?;

    Ok(Some(Release {
        version: tag.to_owned(),
        url: url.to_owned(),
        sha256: sha256.to_owned(),
    }))
}

/// Downloads the release and stages it. Returns the staged path only if the
/// bytes match the published digest.
fn download(agent: &ureq::Agent, release: &Release, data_dir: &Path) -> Result<PathBuf, String> {
    let dir = data_dir.join("update");
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create update folder: {e}"))?;

    let mut bytes = Vec::new();
    get(agent, &release.url, "application/octet-stream")?
        .into_reader()
        .take(MAX_EXE_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("download interrupted: {e}"))?;

    let got = sha256_hex(&bytes);
    if got != release.sha256 {
        return Err(format!(
            "Nebula.exe failed digest check (expected {}, got {got})",
            release.sha256
        ));
    }

    let part = dir.join("Nebula.exe.part");
    let staged = dir.join(STAGED_NAME);
    fs::write(&part, &bytes).map_err(|e| format!("cannot write update: {e}"))?;
    fs::rename(&part, &staged).map_err(|e| format!("cannot stage update: {e}"))?;
    Ok(staged)
}

/// Replaces the running executable with the staged one and returns its path,
/// so the caller can start it. On Windows a running .exe can be renamed but
/// not overwritten, so the current file is moved aside first.
pub fn install(data_dir: &Path) -> Result<PathBuf, String> {
    let staged = data_dir.join("update").join(STAGED_NAME);
    if !staged.is_file() {
        return Err("the downloaded update is missing".into());
    }
    let current = std::env::current_exe().map_err(|e| format!("cannot find Nebula.exe: {e}"))?;
    let dir = current.parent().ok_or("Nebula.exe has no folder")?;
    let old = dir.join(OLD_NAME);

    let _ = fs::remove_file(&old);
    fs::rename(&current, &old).map_err(|e| format!("cannot move Nebula.exe aside: {e}"))?;
    if let Err(e) = fs::copy(&staged, &current) {
        let _ = fs::rename(&old, &current);
        return Err(format!("cannot write the new Nebula.exe: {e}"));
    }
    let _ = fs::remove_file(&staged);
    Ok(current)
}

/// Deletes the executable left behind by the last update, if it is no longer locked.
pub fn remove_leftover() {
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        let _ = fs::remove_file(dir.join(OLD_NAME));
    }
}

/// Runs the check on a background thread. If a newer release exists the user
/// is asked; on Yes the update is staged and `ready_msg` is sent to the event loop.
pub fn spawn_check(data_dir: PathBuf, proxy: EventLoopProxy<String>, ready_msg: &'static str) {
    // A debug build must not replace the developer's executable.
    if cfg!(debug_assertions) {
        return;
    }
    std::thread::spawn(move || {
        let release = match latest_release() {
            Ok(Some(release)) => release,
            Ok(None) => return,
            Err(e) => {
                eprintln!("update check skipped: {e}");
                return;
            }
        };
        let question = format!(
            "Nebula {} is available (you have {}).\n\nUpdate now? Nebula will restart.",
            release.version, CURRENT_VERSION
        );
        if !ask_yes_no(&question) {
            return;
        }
        match agent().and_then(|agent| download(&agent, &release, &data_dir)) {
            Ok(_) => {
                let _ = proxy.send_event(ready_msg.to_owned());
            }
            Err(e) => show_error(&format!("Nebula could not be updated: {e}")),
        }
    });
}

#[cfg(target_os = "windows")]
fn message_box(text: &str, flags: u32) -> i32 {
    use std::ptr::null_mut;
    use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW;

    let text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = "Nebula".encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: both buffers are NUL-terminated and live for the duration of the call.
    unsafe { MessageBoxW(null_mut(), text.as_ptr(), title.as_ptr(), flags) }
}

/// Shows a Yes/No prompt. Returns true only if the user clicked Yes.
#[cfg(target_os = "windows")]
fn ask_yes_no(text: &str) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        IDYES, MB_ICONQUESTION, MB_SETFOREGROUND, MB_TOPMOST, MB_YESNO,
    };
    message_box(text, MB_YESNO | MB_ICONQUESTION | MB_TOPMOST | MB_SETFOREGROUND) == IDYES
}

#[cfg(target_os = "windows")]
pub fn show_error(text: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MB_TOPMOST};
    message_box(text, MB_OK | MB_ICONERROR | MB_TOPMOST);
}

#[cfg(not(target_os = "windows"))]
fn ask_yes_no(_text: &str) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn show_error(text: &str) {
    eprintln!("{text}");
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{is_newer, parse_version};

    #[test]
    fn versions_parse_with_or_without_v() {
        assert_eq!(parse_version("v0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_version("1.10.3"), Some((1, 10, 3)));
        assert_eq!(parse_version("0.2"), None);
        assert_eq!(parse_version("0.2.0-beta"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn only_strictly_newer_releases_count() {
        assert!(is_newer("v0.2.0", "0.1.0"));
        assert!(is_newer("v0.10.0", "0.9.9"));
        assert!(!is_newer("v0.2.0", "0.2.0"));
        assert!(!is_newer("v0.1.0", "0.2.0"));
        assert!(!is_newer("nightly", "0.2.0"));
    }

    #[test]
    fn sha256_matches_known_value() {
        assert_eq!(
            super::sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
