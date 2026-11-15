# Nebula

Nebula is a small Windows desktop wrapper that opens Discord in its own window
(WebView2 via [wry](https://github.com/tauri-apps/wry) and [tao](https://github.com/tauri-apps/tao)),
with [Vencord](https://github.com/Vendicated/Vencord) injected into the page.

## Install

1. Go to the Nebula website: [nebula-skin.netlify.app](https://nebula-skin.netlify.app)
   and click the download button, or open the [latest release](https://github.com/pq5a/nebula/releases/latest)
   directly.
2. Download `Nebula.zip` from the release and extract it to a folder of your choice.
3. Run `Nebula.exe`. Discord opens in its own window.

All versions are listed on the [releases page](https://github.com/pq5a/nebula/releases).
Nebula runs on Windows only.

## Behaviour

- Discord (`discord.com`, `discordapp.com` and subdomains) stays inside the Nebula window.
- Other web links open as a new tab in Firefox.
- Vencord is injected only on Discord pages. To start Nebula without it, create
  `%APPDATA%\Nebula\no-vencord`.
- Discord itself is the live web app at `discord.com/app`, so it is always current.
- Vencord updates itself. On each launch Nebula checks Vencord's `devbuild` GitHub
  release. A new build is downloaded in the background, each file is checked against
  the SHA-256 digest GitHub publishes for it, and it is stored in
  `%APPDATA%\Nebula\vencord`. It takes effect on the next launch. If there is no
  verified download, the copy compiled into the executable is used, so Nebula still
  starts offline.
- Button hover/press transitions are disabled in the injected stylesheet.
- The window has no native title bar. Nebula draws its own (`src/chrome.js`), with
  minimize, maximize and close buttons. Double-clicking it toggles maximize.
- The gear button opens Nebula Settings with three themes (Nebula, Midnight and
  Ember). A theme recolors the whole Discord client: backgrounds, text, and the
  brand accent. Lightness is kept from Discord's palette, and light text is lifted
  to at least 4.5:1 contrast against the surfaces. Red, green and yellow are unchanged.
- An account button (person icon) sits in the top-right title bar. It opens a
  dropdown of saved accounts. To add one, enter a name, email and password and press
  Save. To switch, click the account: Nebula signs out the current session and opens
  Discord's login page with that email and password filled in, then submits the form.
  Discord performs the login itself, so any captcha or 2FA prompt still appears
  and must be completed by hand.
- Saved accounts are kept in `%APPDATA%\Nebula\accounts.dat`, encrypted with Windows
  DPAPI so only your Windows user can read them. Passwords are never sent back to
  the page. Anyone with access to your Windows account can decrypt them, so only
  save accounts you control.
- The chosen theme is saved to `%APPDATA%\Nebula\theme.txt`. The webview has no
  localStorage, so the file is read at startup and injected before the page loads.
  The window and webview use the theme's background color until Discord paints, so
  startup never shows a white screen.

## Building

Requires a Rust toolchain for `x86_64-pc-windows-msvc`.

```sh
cargo build --release
cargo test
```

`vendor/` holds a copy of the Vencord "devbuild" release, compiled into the executable
as the offline fallback. Its license notice is `vendor/vencord.browser.js.LEGAL.txt`.
The build does not download anything.

## Layout

- `src/main.rs` — window, webview setup, navigation rules, Firefox handoff.
- `src/vencord.rs` — Vencord download, digest verification, cache, and page script.
- `src/accounts.rs` — saved accounts, DPAPI encryption, and account requests from the page.
- `src/chrome.js` — title bar, settings, account dropdown, and theme recoloring (injected).
- `build.rs` — embeds the Windows icon.
- `vendor/` — third-party Vencord files (fallback copy).
- `assets/icon.ico`, `Logo.png` — application icon and logo.

## License

Nebula is licensed under the GNU General Public License v3.0 (see `LICENSE`). The vendored Vencord and third-party files keep their own licenses, listed in `vendor/vencord.browser.js.LEGAL.txt`.
