#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(target_os = "windows")]
use tao::platform::windows::IconExtWindows;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Icon, WindowBuilder},
};
use wry::{WebContext, WebViewBuilder, RGBA};

mod vencord;

const CHROME_SCRIPT: &str = include_str!("chrome.js");

const APP_NAME: &str = "Nebula";
const DISCORD_URL: &str = "https://discord.com/app";

// Hosts that stay inside the Nebula window. Everything else opens in Firefox.
const IN_APP_HOSTS: [&str; 2] = ["discord.com", "discordapp.com"];

// Resource ordinal of the icon embedded by build.rs (winres uses 1).
#[cfg(target_os = "windows")]
const APP_ICON_RESOURCE: u16 = 1;

fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(APP_NAME)
}

// Create %APPDATA%\Nebula\no-vencord to start Nebula without Vencord.
fn vencord_enabled(data_dir: &Path) -> bool {
    !data_dir.join("no-vencord").exists()
}

// Returns the host part of an http(s) URL, without userinfo or port.
fn http_host(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host_port = authority.rsplit('@').next()?;
    host_port.split(':').next()
}

fn is_in_app(url: &str) -> bool {
    http_host(url).is_some_and(|host| {
        let host = host.to_ascii_lowercase();
        IN_APP_HOSTS
            .iter()
            .any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
    })
}

fn is_web_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

// Finds firefox.exe in the standard install locations, falling back to PATH.
fn firefox_exe() -> PathBuf {
    ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"]
        .iter()
        .filter_map(std::env::var_os)
        .map(|base| PathBuf::from(base).join("Mozilla Firefox").join("firefox.exe"))
        .find(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from("firefox.exe"))
}

// Opens an http(s) link as a new tab in Firefox. The URL is passed as a
// single argument (no shell), and only web URLs are forwarded.
fn open_in_firefox(url: &str) {
    if !is_web_url(url) {
        return;
    }
    let _ = Command::new(firefox_exe())
        .arg("-new-tab")
        .arg(url)
        .spawn();
}

// Window actions requested by the in-page title bar (src/chrome.js).
#[derive(Debug, PartialEq)]
enum WindowCommand {
    Drag,
    Minimize,
    Maximize,
    Close,
}

fn parse_command(msg: &str) -> Option<WindowCommand> {
    match msg {
        "drag" => Some(WindowCommand::Drag),
        "minimize" => Some(WindowCommand::Minimize),
        "maximize" => Some(WindowCommand::Maximize),
        "close" => Some(WindowCommand::Close),
        _ => None,
    }
}

// Built-in themes: id, and the background shown before Discord paints. The
// full palettes live in src/chrome.js.
const THEMES: [(&str, RGBA); 3] = [
    ("nebula", (0x0d, 0x0a, 0x1f, 255)),
    ("midnight", (0x05, 0x08, 0x0f, 255)),
    ("ember", (0x14, 0x0a, 0x08, 255)),
];

fn known_theme(id: &str) -> Option<&'static str> {
    THEMES.iter().map(|(t, _)| *t).find(|t| *t == id)
}

fn theme_background(id: &str) -> RGBA {
    THEMES
        .iter()
        .find(|(t, _)| *t == id)
        .map_or(THEMES[0].1, |(_, color)| *color)
}

fn theme_path(data_dir: &Path) -> PathBuf {
    data_dir.join("theme.txt")
}

// The saved theme, or the default if none is saved or the file is invalid.
fn saved_theme(data_dir: &Path) -> &'static str {
    fs::read_to_string(theme_path(data_dir))
        .ok()
        .and_then(|s| known_theme(s.trim()))
        .unwrap_or(THEMES[0].0)
}

fn main() -> wry::Result<()> {
    let event_loop = EventLoopBuilder::<WindowCommand>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    #[cfg(target_os = "windows")]
    let icon = Icon::from_resource(APP_ICON_RESOURCE, None).ok();
    #[cfg(not(target_os = "windows"))]
    let icon: Option<Icon> = None;

    let data_dir = data_dir();
    let theme = saved_theme(&data_dir);
    // Painted until Discord renders, so startup shows the theme, not white.
    let background = theme_background(theme);

    let window = WindowBuilder::new()
        .with_title(APP_NAME)
        .with_decorations(false)
        .with_window_icon(icon)
        .with_background_color(background)
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(LogicalSize::new(640.0, 480.0))
        .build(&event_loop)
        .expect("failed to create window");

    let mut web_context = WebContext::new(Some(data_dir.clone()));
    let chrome_script = format!("window.__nebulaTheme = \"{theme}\";\n{CHROME_SCRIPT}");
    let theme_file = theme_path(&data_dir);

    let mut builder = WebViewBuilder::with_web_context(&mut web_context)
        .with_url(DISCORD_URL)
        .with_background_color(background)
        // Links to other sites leave the app and open as a Firefox tab.
        .with_navigation_handler(|url| {
            if is_in_app(&url) || !is_web_url(&url) {
                true
            } else {
                open_in_firefox(&url);
                false
            }
        })
        // target="_blank" and window.open() never create a browser window.
        .with_new_window_req_handler(|url| {
            if is_in_app(&url) {
                true
            } else {
                open_in_firefox(&url);
                false
            }
        })
        // Title bar and settings messages. Only Discord pages may send them.
        .with_ipc_handler(move |req| {
            if !is_in_app(&req.uri().to_string()) {
                return;
            }
            let msg = req.body();
            if let Some(id) = msg.strip_prefix("theme:") {
                if let Some(id) = known_theme(id) {
                    let _ = fs::write(&theme_file, id);
                }
            } else if let Some(cmd) = parse_command(msg) {
                let _ = proxy.send_event(cmd);
            }
        })
        .with_initialization_script(&chrome_script);

    #[cfg(debug_assertions)]
    {
        builder = builder.with_devtools(true);
    }

    if vencord_enabled(&data_dir) {
        // Use the last verified download if there is one, otherwise the copy
        // compiled into the executable. A new download applies on next launch.
        let bundle = vencord::load_cached(&data_dir).unwrap_or_else(vencord::embedded);
        match vencord::init_script(&bundle) {
            Ok(script) => builder = builder.with_initialization_script(&script),
            Err(e) => eprintln!("Vencord disabled: {e}"),
        }
        vencord::spawn_update(data_dir.clone());
    }

    let webview = builder.build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::UserEvent(cmd) => match cmd {
                WindowCommand::Drag => {
                    let _ = window.drag_window();
                }
                WindowCommand::Minimize => window.set_minimized(true),
                WindowCommand::Maximize => window.set_maximized(!window.is_maximized()),
                WindowCommand::Close => *control_flow = ControlFlow::Exit,
            },
            _ => {}
        }

        // Keep the window, webview and web context alive for the loop's lifetime.
        let _ = (&window, &webview, &web_context);
    });
}

#[cfg(test)]
mod tests {
    use super::{http_host, is_in_app, is_web_url, known_theme, parse_command, WindowCommand};

    #[test]
    fn only_built_in_themes_are_saved() {
        assert_eq!(known_theme("midnight"), Some("midnight"));
        assert_eq!(known_theme("ember"), Some("ember"));
        assert_eq!(known_theme("../evil"), None);
        assert_eq!(known_theme("nebula\";alert(1)//"), None);
    }

    #[test]
    fn only_known_window_commands_are_accepted() {
        assert_eq!(parse_command("drag"), Some(WindowCommand::Drag));
        assert_eq!(parse_command("minimize"), Some(WindowCommand::Minimize));
        assert_eq!(parse_command("maximize"), Some(WindowCommand::Maximize));
        assert_eq!(parse_command("close"), Some(WindowCommand::Close));
        assert_eq!(parse_command("exec x"), None);
        assert_eq!(parse_command(""), None);
    }

    #[test]
    fn discord_urls_stay_in_app() {
        assert!(is_in_app("https://discord.com/app"));
        assert!(is_in_app("https://DISCORD.com/channels/1/2"));
        assert!(is_in_app("https://canary.discord.com/app"));
        assert!(is_in_app("https://discord.com:443/app?x=1#y"));
        assert!(is_in_app("https://discordapp.com/x"));
    }

    #[test]
    fn other_urls_leave_the_app() {
        assert!(!is_in_app("https://example.com/"));
        assert!(!is_in_app("https://notdiscord.com/"));
        assert!(!is_in_app("https://discord.com.evil.net/"));
        assert!(!is_in_app("https://evil.com/discord.com"));
        assert!(!is_in_app("https://evil.com?discord.com"));
        assert!(!is_in_app("https://user@evil.com/@discord.com"));
        assert!(!is_in_app("about:blank"));
    }

    #[test]
    fn host_parsing() {
        assert_eq!(http_host("https://a.b:8080/c?d#e"), Some("a.b"));
        assert_eq!(http_host("http://u:p@host/"), Some("host"));
        assert_eq!(http_host("mailto:x@y"), None);
    }

    #[test]
    fn only_web_urls_are_forwarded() {
        assert!(is_web_url("https://a"));
        assert!(is_web_url("http://a"));
        assert!(!is_web_url("file:///C:/x"));
        assert!(!is_web_url("-new-window"));
    }
}
