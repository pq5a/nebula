use std::{env, fs, path::Path};

// The Vencord files in vendor/ are pinned copies of the "devbuild" release
// (DevBuild d07030c, https://github.com/Vendicated/Vencord/releases/tag/devbuild).
// The build never downloads them, so builds are offline and reproducible.
// To update, replace the three files in vendor/ with the matching assets from
// that release (browser.js, browser.css, browser.js.LEGAL.txt).
const VENDOR_FILES: [&str; 3] = [
    "vendor/vencord.browser.js",
    "vendor/vencord.browser.css",
    "vendor/vencord.browser.js.LEGAL.txt",
];

// Top-level declaration in the Vencord bundle that must become a global.
const BUNDLE_GLOBAL_DECL: &str = "var Vencord=";

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

// Rewrites the bundle so its global `Vencord` is assigned to `window`
// instead of declared with `var`. A `var` inside a function wrapper would be
// function-scoped and never reach the page's global object.
fn globalize_bundle(js: &str) -> String {
    let pos = js
        .find(BUNDLE_GLOBAL_DECL)
        .expect("vendored Vencord bundle has no `var Vencord=` declaration");
    assert!(
        pos < 512 && js[pos + BUNDLE_GLOBAL_DECL.len()..].find(BUNDLE_GLOBAL_DECL).is_none(),
        "unexpected layout of the vendored Vencord bundle"
    );
    format!(
        "{}window.Vencord={}",
        &js[..pos],
        &js[pos + BUNDLE_GLOBAL_DECL.len()..]
    )
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    for file in VENDOR_FILES {
        println!("cargo:rerun-if-changed={file}");
        assert!(
            Path::new(file).exists(),
            "missing {file}; see the note at the top of build.rs"
        );
    }

    let js = fs::read_to_string("vendor/vencord.browser.js").expect("read Vencord JS");
    let css = fs::read_to_string("vendor/vencord.browser.css").expect("read Vencord CSS");

    // Runs once per document of discord.com. The CSS is added as soon as a
    // <head>/<html> exists, and the bundle is started regardless of the CSS.
    let script = format!(
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

{bundle}
}})();
"#,
        css = js_string(&css),
        bundle = globalize_bundle(&js),
    );

    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("nebula-init.js");
    fs::write(out, script).unwrap();

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        let icon = Path::new("assets/icon.ico");
        if icon.exists() {
            println!("cargo:rerun-if-changed=assets/icon.ico");
            res.set_icon("assets/icon.ico");
        }
        res.set("ProductName", "Nebula")
            .set("FileDescription", "Nebula Discord Client");
        res.compile().expect("failed to compile Windows resources");
    }
}
