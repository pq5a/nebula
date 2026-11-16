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
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{Icon, WindowBuilder},
};
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;
use wry::{WebContext, WebView, WebViewBuilder};

use serde_json::json;

mod accounts;
mod themes;
mod updater;
mod vencord;

const CHROME_SCRIPT: &str = include_str!("chrome.js");

const APP_NAME: &str = "Nebula";

// WebView2 (Chromium) switches. Background networking, component updates,
// sync and reporting are off, which cuts idle CPU, RAM and network use.
#[cfg(target_os = "windows")]
const WEBVIEW_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,MediaRouter,OptimizationHints \
    --disable-background-networking --disable-component-update --disable-sync \
    --disable-breakpad --metrics-recording-only --no-first-run --disable-default-apps";
#[cfg(not(target_os = "windows"))]
const WEBVIEW_ARGS: &str = "";
const DISCORD_URL: &str = "https://discord.com/app";
const LOGIN_URL: &str = "https://discord.com/login";
// Internal event from the switch timer. Page messages with this prefix are dropped.
const LOGIN_CONTINUE: &str = "internal:account-continue";
// Sent by the update thread once the new executable is staged and the user said yes.
const UPDATE_READY: &str = "internal:update-ready";

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

// Low-power mode (10 FPS while tabbed out) is saved as "on" or "off". It is on
// unless the file says "off", so a missing or damaged file keeps the default.
fn throttle_path(data_dir: &Path) -> PathBuf {
    data_dir.join("throttle.txt")
}

fn parse_throttle(text: &str) -> bool {
    text.trim() != "off"
}

fn saved_throttle(data_dir: &Path) -> bool {
    fs::read_to_string(throttle_path(data_dir))
        .map_or(true, |s| parse_throttle(&s))
}

// The pointer style chosen in the settings menu (src/chrome.js), saved by id.
// "system" is the default, so a missing or damaged file keeps Windows' pointer.
const CURSORS: [&str; 5] = ["system", "cross", "dot", "ring", "corners"];

fn cursor_path(data_dir: &Path) -> PathBuf {
    data_dir.join("cursor.txt")
}

fn parse_cursor(text: &str) -> Option<&'static str> {
    let text = text.trim();
    CURSORS.iter().copied().find(|id| *id == text)
}

fn saved_cursor(data_dir: &Path) -> &'static str {
    let text = fs::read_to_string(cursor_path(data_dir)).unwrap_or_default();
    parse_cursor(&text).unwrap_or("system")
}

fn main() -> wry::Result<()> {
    // Messages from the page (window commands, theme and account requests) are
    // handled on the event loop, which owns the window and webview.
    let event_loop = EventLoopBuilder::<String>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    #[cfg(target_os = "windows")]
    let icon = Icon::from_resource(APP_ICON_RESOURCE, None).ok();
    #[cfg(not(target_os = "windows"))]
    let icon: Option<Icon> = None;

    let data_dir = data_dir();
    let theme = themes::saved_theme(&data_dir);
    let custom = themes::saved_custom(&data_dir);
    // Painted until Discord renders, so startup shows the theme, not white.
    let background = themes::window_background(&theme, custom.as_ref());

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
    let throttle = saved_throttle(&data_dir);
    // The id is one of CURSORS, so it is safe to put in a JavaScript string.
    let cursor = saved_cursor(&data_dir);
    let chrome_script = format!(
        "window.__nebulaTheme = \"{theme}\";\nwindow.__nebulaCustom = {};\nwindow.__nebulaThrottle = {throttle};\nwindow.__nebulaCursor = \"{cursor}\";\n{CHROME_SCRIPT}",
        themes::custom_json(custom.as_ref())
    );

    let mut builder = WebViewBuilder::with_web_context(&mut web_context)
        .with_url(DISCORD_URL)
        .with_background_color(background)
        // Replaces wry's default args, so its three default disabled features
        // are repeated here. The rest turn off background services Nebula
        // doesn't need (updates, sync, crash and usage reporting).
        .with_additional_browser_args(WEBVIEW_ARGS)
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
        // Title bar, settings and account messages. Only Discord pages may send them.
        .with_ipc_handler(move |req| {
            let from_discord = is_in_app(&req.uri().to_string());
            let body = req.into_body();
            if from_discord && !body.starts_with("internal:") {
                let _ = proxy.send_event(body);
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

    // The update check starts only after everything is loaded, so it never delays startup.
    updater::remove_leftover();
    updater::spawn_check(data_dir.clone(), event_loop.create_proxy(), UPDATE_READY);

    let throttle_file = throttle_path(&data_dir);
    let accounts_file = data_dir.join(accounts::FILE_NAME);
    let mut saved_accounts = accounts::load(&accounts_file);
    // The account being switched to. It is filled in once Discord's login page loads.
    let mut pending_login: Option<accounts::Account> = None;
    let later_proxy = event_loop.create_proxy();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            // Keeps the title bar's maximize/restore button in step with the window.
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let _ = webview.evaluate_script(&format!(
                    "window.__nebulaMaximized && window.__nebulaMaximized({});",
                    window.is_maximized()
                ));
            }
            Event::UserEvent(msg) => {
                if msg == LOGIN_CONTINUE {
                    let _ = webview.load_url(LOGIN_URL);
                } else if msg == UPDATE_READY {
                    match updater::install(&data_dir) {
                        Ok(exe) => {
                            let _ = std::process::Command::new(exe).spawn();
                            *control_flow = ControlFlow::Exit;
                        }
                        Err(e) => updater::show_error(&format!("Nebula could not be updated: {e}")),
                    }
                } else if let Some(id) = msg.strip_prefix("theme:") {
                    if themes::can_select(id, &data_dir) {
                        if let Err(e) = themes::save_theme(&data_dir, id) {
                            eprintln!("could not save theme: {e}");
                        }
                    }
                } else if let Some(json) = msg.strip_prefix("custom:") {
                    // The page sends "theme:custom" right after this, so the saved colors exist by then.
                    if let Some(custom) = themes::parse_custom(json) {
                        if let Err(e) = themes::save_custom(&data_dir, &custom) {
                            eprintln!("could not save custom theme: {e}");
                        }
                    }
                } else if let Some(value) = msg.strip_prefix("throttle:") {
                    match value {
                        "on" => {
                            let _ = fs::write(&throttle_file, "on");
                        }
                        "off" => {
                            let _ = fs::write(&throttle_file, "off");
                        }
                        _ => {}
                    }
                } else if let Some(id) = msg.strip_prefix("cursor:") {
                    if let Some(id) = parse_cursor(id) {
                        if let Err(e) = fs::write(cursor_path(&data_dir), id) {
                            eprintln!("could not save cursor: {e}");
                        }
                    }
                } else if let Some(request) = accounts::parse_request(&msg) {
                    handle_account(
                        request,
                        &mut saved_accounts,
                        &accounts_file,
                        &mut pending_login,
                        &later_proxy,
                        &webview,
                    );
                } else if let Some(cmd) = parse_command(&msg) {
                    match cmd {
                        WindowCommand::Drag => {
                            let _ = window.drag_window();
                        }
                        WindowCommand::Minimize => window.set_minimized(true),
                        WindowCommand::Maximize => window.set_maximized(!window.is_maximized()),
                        WindowCommand::Close => *control_flow = ControlFlow::Exit,
                    }
                }
            }
            _ => {}
        }

        // Keep the window, webview and web context alive for the loop's lifetime.
        let _ = (&window, &webview, &web_context);
    });
}

// Runs an account request from the page, then sends the page the new list.
fn handle_account(
    request: accounts::Request,
    saved: &mut Vec<accounts::Account>,
    file: &Path,
    pending: &mut Option<accounts::Account>,
    later: &EventLoopProxy<String>,
    webview: &WebView,
) {
    let mut error = None;
    let mut added = false;
    match request {
        accounts::Request::List => {}
        accounts::Request::LoginPage => {
            // Discord's login form is showing. Fill it in, then submit it.
            if let Some(account) = pending.take() {
                let creds = json!({ "email": account.email, "password": account.password });
                let _ = webview.evaluate_script(&format!("window.__nebulaFill({creds});"));
            }
            return;
        }
        accounts::Request::Add {
            name,
            email,
            password,
        } => match accounts::add(saved, &name, &email, &password) {
            Ok(_) => {
                added = true;
                if let Err(e) = accounts::save(file, saved) {
                    eprintln!("could not save accounts: {e}");
                    error = Some("Could not save the account.");
                }
            }
            Err(message) => error = Some(message),
        },
        accounts::Request::Remove { id } => {
            if accounts::remove(saved, &id) {
                if let Err(e) = accounts::save(file, saved) {
                    eprintln!("could not save accounts: {e}");
                    error = Some("Could not save the account list.");
                }
            }
        }
        accounts::Request::Switch { id } => {
            // Sign the current session out, then open Discord's login page. The
            // page asks for the pending account, and its form gets filled in.
            // The webview has no localStorage, so sign out by clearing its browsing
            // data. WebView2 does that asynchronously, so the login page opens after
            // a short wait (see "internal:account-continue" in the event loop).
            if let Some(account) = saved.iter().find(|a| a.id == id) {
                *pending = Some(account.clone());
                if let Err(e) = webview.clear_all_browsing_data() {
                    eprintln!("could not clear browsing data: {e}");
                }
                let later = later.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                    let _ = later.send_event(LOGIN_CONTINUE.to_owned());
                });
            }
            return;
        }
    }

    let list = accounts::public_json(saved);
    let error = serde_json::to_string(&error).unwrap_or_else(|_| "null".into());
    let _ = webview.evaluate_script(&format!(
        "window.__nebulaSetAccounts({{accounts:{list},error:{error},added:{added}}});"
    ));
}

#[cfg(test)]
mod tests {
    use super::{
        http_host, is_in_app, is_web_url, parse_command, parse_cursor, parse_throttle, WindowCommand,
    };

    #[test]
    fn throttle_is_on_unless_turned_off() {
        assert!(parse_throttle(""));
        assert!(parse_throttle("on\n"));
        assert!(parse_throttle("garbage"));
        assert!(!parse_throttle("off\n"));
    }

    #[test]
    fn only_known_cursors_are_accepted() {
        assert_eq!(parse_cursor("cross"), Some("cross"));
        assert_eq!(parse_cursor("corners\n"), Some("corners"));
        assert_eq!(parse_cursor("system"), Some("system"));
        assert_eq!(parse_cursor("Cross"), None);
        assert_eq!(parse_cursor("../evil"), None);
        assert_eq!(parse_cursor(""), None);
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
