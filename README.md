# Nebula

Nebula is a small Windows desktop wrapper that opens Discord in its own window
(WebView2 via [wry](https://github.com/tauri-apps/wry) and [tao](https://github.com/tauri-apps/tao)),
with [Vencord](https://github.com/Vendicated/Vencord) injected into the page.

## Behaviour

- Discord (`discord.com`, `discordapp.com` and subdomains) stays inside the Nebula window.
- Other web links open as a new tab in Firefox.
- Vencord is injected only on Discord pages. To start Nebula without it, create
  `%APPDATA%\Nebula\no-vencord`.
- Button hover/press transitions are disabled in the injected stylesheet.

## Building

Requires a Rust toolchain for `x86_64-pc-windows-msvc`.

```sh
cargo build --release
cargo test
```

The Vencord bundle, stylesheet and license notice in `vendor/` are pinned copies of the
Vencord "devbuild" release (see the note at the top of `build.rs`). The build does not
download anything, so it works offline. The build script embeds them into the executable.

## Layout

- `src/main.rs` — window, webview setup, navigation rules, Firefox handoff.
- `build.rs` — builds the init script from the vendored Vencord files and embeds the Windows icon.
- `vendor/` — third-party Vencord files (see `vendor/vencord.browser.js.LEGAL.txt`).
- `assets/icon.ico`, `Logo.png` — application icon and logo.

## License

Nebula is licensed under the GNU General Public License v3.0 (see `LICENSE`). The vendored Vencord and third-party files keep their own licenses, listed in `vendor/vencord.browser.js.LEGAL.txt`.
