// Nebula window chrome: custom title bar, window controls and the settings
// modal with theme picker. Injected before every page; runs on discord.com only.
(function () {
  "use strict";
  var host = location.hostname;
  if (host !== "discord.com" && !host.endsWith(".discord.com")) return;
  if (window.__nebulaChrome) return;
  window.__nebulaChrome = true;

  // hue/tint: hue and saturation (%) for Discord's neutral and primary ramps.
  // accentHue: hue for Discord's blurple ramp (brand color). Lightness is kept
  // from Discord's own palette, so text contrast matches the stock theme.
  var THEMES = [
    { id: "nebula", name: "Nebula", blurb: "Deep violet with a soft glow", bg: "#0d0a1f", panel: "#15112e", accent: "#a78bfa", hue: 258, tint: 55, accentHue: 262 },
    { id: "midnight", name: "Midnight", blurb: "Cold blue, crisp edges", bg: "#05080f", panel: "#0a1322", accent: "#38bdf8", hue: 212, tint: 50, accentHue: 198 },
    { id: "ember", name: "Ember", blurb: "Warm dark with orange heat", bg: "#140a08", panel: "#22120d", accent: "#fb923c", hue: 22, tint: 48, accentHue: 24 }
  ];

  var STYLE = [
    ":root { --nb-bg:#0d0a1f; --nb-panel:#15112e; --nb-accent:#a78bfa; --nb-text:#ede9ff; --nb-muted:#8b82b5; --nb-glow:rgba(167,139,250,.35); }",
    '[data-nb-theme="midnight"] { --nb-bg:#05080f; --nb-panel:#0a1322; --nb-accent:#38bdf8; --nb-text:#dbeafe; --nb-muted:#64748b; --nb-glow:rgba(56,189,248,.30); }',
    '[data-nb-theme="ember"] { --nb-bg:#140a08; --nb-panel:#22120d; --nb-accent:#fb923c; --nb-text:#ffedd5; --nb-muted:#9a6b57; --nb-glow:rgba(251,146,60,.30); }',
    "#nb-titlebar { position:fixed; top:0; left:0; right:0; height:36px; z-index:2147483647; display:flex; align-items:center;",
    "  background:linear-gradient(90deg,var(--nb-bg),var(--nb-panel)); border-bottom:1px solid var(--nb-glow);",
    "  box-shadow:0 2px 14px var(--nb-glow); color:var(--nb-text); user-select:none; font:600 12px/1 'Segoe UI Variable','Segoe UI',sans-serif; }",
    "#nb-titlebar .nb-brand { display:flex; align-items:center; gap:9px; padding:0 14px; height:100%; letter-spacing:.12em; text-transform:uppercase; font-size:11px; }",
    "#nb-titlebar .nb-dot { width:8px; height:8px; border-radius:50%; background:var(--nb-accent); box-shadow:0 0 10px var(--nb-accent); }",
    "#nb-titlebar .nb-spacer { flex:1; height:100%; }",
    "#nb-titlebar button { all:unset; box-sizing:border-box; width:46px; height:36px; display:grid; place-items:center; color:var(--nb-muted); cursor:default; }",
    "#nb-titlebar button.nb-gear { width:42px; }",
    "#nb-titlebar button:hover { background:rgba(255,255,255,.08); color:var(--nb-text); }",
    "#nb-titlebar button.nb-close:hover { background:#e81123; color:#fff; }",
    "#nb-titlebar svg { width:12px; height:12px; fill:none; stroke:currentColor; stroke-width:1.4; stroke-linecap:round; }",
    "#nb-titlebar .nb-gear svg { width:15px; height:15px; }",
    "html body #app-mount { padding-top:36px !important; box-sizing:border-box !important; }",
    "#nb-overlay { position:fixed; inset:0; z-index:2147483646; display:none; align-items:center; justify-content:center; background:rgba(3,2,10,.7); font:13px/1.4 'Segoe UI Variable','Segoe UI',sans-serif; }",
    "#nb-overlay.open { display:flex; animation:nb-in .18s ease-out; }",
    "@keyframes nb-in { from { opacity:0; transform:scale(.98); } to { opacity:1; transform:none; } }",
    "#nb-modal { width:580px; max-width:92vw; background:var(--nb-panel); color:var(--nb-text); border-radius:16px; border:1px solid var(--nb-glow); box-shadow:0 24px 70px rgba(0,0,0,.6),0 0 46px var(--nb-glow); overflow:hidden; }",
    "#nb-modal header { display:flex; align-items:center; justify-content:space-between; padding:18px 22px; border-bottom:1px solid rgba(255,255,255,.06); }",
    "#nb-modal h2 { margin:0; font-size:16px; font-weight:700; letter-spacing:.02em; }",
    "#nb-modal .nb-close-modal { all:unset; cursor:pointer; color:var(--nb-muted); font-size:18px; line-height:1; padding:4px 6px; border-radius:6px; }",
    "#nb-modal .nb-close-modal:hover { color:var(--nb-text); background:rgba(255,255,255,.08); }",
    "#nb-modal .nb-tabs { display:flex; gap:6px; padding:12px 22px 0; }",
    "#nb-modal .nb-tab { padding:7px 14px; border-radius:999px; color:var(--nb-accent); background:rgba(255,255,255,.06); font-weight:600; font-size:12px; }",
    "#nb-modal .nb-hint { margin:14px 22px 0; color:var(--nb-muted); font-size:12px; }",
    "#nb-modal .nb-cards { display:grid; grid-template-columns:repeat(3,1fr); gap:12px; padding:14px 22px 22px; }",
    "#nb-modal .nb-card { position:relative; cursor:pointer; padding:10px; border-radius:14px; background:rgba(255,255,255,.03); border:1px solid rgba(255,255,255,.07); transition:transform .12s,border-color .12s; }",
    "#nb-modal .nb-card:hover { transform:translateY(-3px); border-color:var(--card-accent); }",
    "#nb-modal .nb-card.active { border-color:var(--card-accent); box-shadow:0 0 0 1px var(--card-accent),0 10px 28px rgba(0,0,0,.45),0 0 26px var(--card-glow); }",
    "#nb-modal .nb-preview { position:relative; height:92px; border-radius:9px; overflow:hidden; background:var(--card-bg); }",
    "#nb-modal .nb-preview::before { content:''; position:absolute; inset:0; background:radial-gradient(circle at 80% 15%,var(--card-accent),transparent 55%); opacity:.55; }",
    "#nb-modal .nb-preview .nb-bar { position:absolute; left:8px; right:8px; top:8px; height:8px; border-radius:4px; background:var(--card-panel); }",
    "#nb-modal .nb-preview .nb-line { position:absolute; left:8px; height:6px; border-radius:3px; background:rgba(255,255,255,.14); }",
    "#nb-modal .nb-preview .nb-dot { position:absolute; left:8px; bottom:10px; width:22px; height:22px; border-radius:50%; background:var(--card-accent); box-shadow:0 0 14px var(--card-accent); }",
    "#nb-modal .nb-name { margin:10px 2px 2px; font-weight:700; font-size:14px; }",
    "#nb-modal .nb-blurb { margin:0 2px; color:var(--nb-muted); font-size:11.5px; }",
    "#nb-modal .nb-check { position:absolute; top:18px; right:18px; width:20px; height:20px; border-radius:50%; background:var(--card-accent); color:#0b0716; display:none; place-items:center; font-size:12px; font-weight:800; box-shadow:0 0 12px var(--card-glow); }",
    "#nb-modal .nb-card.active .nb-check { display:grid; }"
  ].join("\n");

  var ICON = {
    min: '<svg viewBox="0 0 12 12"><path d="M1.5 6h9"/></svg>',
    max: '<svg viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" rx="1.5"/></svg>',
    close: '<svg viewBox="0 0 12 12"><path d="M2 2l8 8M10 2l-8 8"/></svg>',
    gear: '<svg viewBox="0 0 16 16"><circle cx="8" cy="8" r="2.3"/><path d="M8 1.6v2.1M8 12.3v2.1M1.6 8h2.1M12.3 8h2.1M3.4 3.4l1.5 1.5M11.1 11.1l1.5 1.5M3.4 12.6l1.5-1.5M11.1 4.9l1.5-1.5"/></svg>'
  };

  function send(cmd) {
    if (window.ipc) window.ipc.postMessage(cmd);
  }

  function rgbToHsl(r, g, b) {
    r /= 255; g /= 255; b /= 255;
    var max = Math.max(r, g, b), min = Math.min(r, g, b), h = 0, s = 0, l = (max + min) / 2;
    if (max !== min) {
      var d = max - min;
      s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
      if (max === r) h = (g - b) / d + (g < b ? 6 : 0);
      else if (max === g) h = (b - r) / d + 2;
      else h = (r - g) / d + 4;
      h *= 60;
    }
    return [h, s * 100, l * 100];
  }

  // Discord uses color-mix(in oklab, ...), which canvas reports in oklab form.
  function oklabToRgb(L, a, b) {
    var l = Math.pow(L + 0.3963377774 * a + 0.2158037573 * b, 3);
    var m = Math.pow(L - 0.1055613458 * a - 0.0638541728 * b, 3);
    var s = Math.pow(L - 0.0894841775 * a - 1.2914855480 * b, 3);
    return [
      4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
      -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
      -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    ].map(function (x) {
      x = Math.min(1, Math.max(0, x));
      x = x <= 0.0031308 ? 12.92 * x : 1.055 * Math.pow(x, 1 / 2.4) - 0.055;
      return Math.round(x * 255);
    });
  }

  // Returns [r, g, b, alpha] (0-255) for a CSS color, or null if `value` is not one.
  var canvas = document.createElement("canvas").getContext("2d");
  function parseColor(value) {
    canvas.fillStyle = "#010203";
    canvas.fillStyle = value;
    var s = canvas.fillStyle;
    if (s === "#010203") return null;
    if (s.charAt(0) === "#") {
      return [parseInt(s.substr(1, 2), 16), parseInt(s.substr(3, 2), 16), parseInt(s.substr(5, 2), 16), 1];
    }
    var n = s.match(/-?[\d.]+/g).map(Number);
    var alpha = n.length > 3 ? n[3] : 1;
    if (s.indexOf("oklab(") === 0) return oklabToRgb(n[0], n[1], n[2]).concat(alpha);
    if (s.indexOf("oklch(") === 0) {
      var h = n[2] * Math.PI / 180;
      return oklabToRgb(n[0], n[1] * Math.cos(h), n[1] * Math.sin(h)).concat(alpha);
    }
    if (s.indexOf("color(") === 0) return [n[0] * 255, n[1] * 255, n[2] * 255, alpha];
    return [n[0], n[1], n[2], alpha];
  }

  // Discord's colors are CSS custom properties, and many are copied as literal
  // values rather than referenced. So every color property is remapped by hue
  // family, keeping its lightness and alpha: neutral surfaces and text take the
  // theme's hue, blurple/brand colors take the accent hue, and red, green and
  // yellow are left alone. Lightness is unchanged, so contrast stays as in Discord.
  // Discord's own value for each variable, parsed to [r, g, b, alpha] (or null
  // for non-colors). Read once per palette, not on every update.
  var originals = {};
  // Variable -> { want, shown }: the value we set, and the value the browser stored.
  var applied = {};
  var root = null;
  // Set by Rust from the saved choice: the webview has no localStorage.
  var current = THEMES.filter(function (t) { return t.id === window.__nebulaTheme; })[0] || THEMES[0];

  // Recolors Discord's variables for `theme`. With `fresh`, Discord's palette
  // may have changed, so our overrides are removed first and the originals
  // are read again. Only values that differ from what is already set are written.
  function retheme(theme, fresh) {
    var names;
    if (fresh) {
      Object.keys(applied).forEach(function (name) { root.style.removeProperty(name); });
      applied = {};
      originals = {};
      // Walk the computed style once to find the variables, and read them all
      // before any write: the computed style is live, so alternating reads and
      // writes would force a full restyle on every access.
      var cs = getComputedStyle(root);
      names = [];
      for (var i = 0; i < cs.length; i++) if (cs[i].indexOf("--") === 0) names.push(cs[i]);
      names.forEach(function (name) { originals[name] = parseColor(cs.getPropertyValue(name)); });
    } else {
      // The variable names are already known, so skip the computed-style walk.
      names = Object.keys(originals);
    }
    // Light colors are text, so they must stay readable on the lightest surface.
    var surface = hslLum(theme.hue, theme.tint, 18);
    var next = {};
    names.forEach(function (name) {
      var c = originals[name];
      if (!c) return;
      var hsl = rgbToHsl(c[0], c[1], c[2]);
      var h = hsl[0], s = hsl[1], l = hsl[2];
      if (s > 30 && h >= 195 && h <= 265) h = theme.accentHue;
      else if (s <= 30) { h = theme.hue; s = theme.tint; }
      else return;
      if (l > 50) while (l < 100 && contrast(hslLum(h, s, l), surface) < 4.5) l += 0.5;
      var value = "hsl(" + h + " " + s + "% " + l + "% / " + c[3] + ")";
      var prev = applied[name];
      // Rewrite if the wanted value changed, or if something replaced our
      // override (the browser stores the value in its own form, so compare that).
      var intact = prev && root.style.getPropertyValue(name) === prev.shown &&
        root.style.getPropertyPriority(name) === "important";
      if (!intact || prev.want !== value) root.style.setProperty(name, value, "important");
      next[name] = { want: value, shown: root.style.getPropertyValue(name) };
    });
    Object.keys(applied).forEach(function (name) {
      if (!(name in next)) root.style.removeProperty(name);
    });
    applied = next;
  }

  // Coalesces bursts of Discord DOM changes into one update per frame.
  var scheduled = 0;
  var scheduledFresh = false;
  function schedule(fresh) {
    scheduledFresh = scheduledFresh || fresh;
    if (scheduled) return;
    scheduled = requestAnimationFrame(function () {
      var freshNow = scheduledFresh;
      scheduled = 0;
      scheduledFresh = false;
      retheme(current, freshNow);
    });
  }

  // Relative luminance (WCAG) of an HSL color.
  function hslLum(h, s, l) {
    s /= 100; l /= 100;
    var a = s * Math.min(l, 1 - l);
    var f = function (n) {
      var k = (n + h / 30) % 12;
      return l - a * Math.max(-1, Math.min(k - 3, 9 - k, 1));
    };
    var c = [f(0), f(8), f(4)].map(function (x) {
      return x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4);
    });
    return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
  }

  function contrast(a, b) {
    return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
  }

  function markActive() {
    document.querySelectorAll("#nb-modal .nb-card").forEach(function (card) {
      card.classList.toggle("active", card.dataset.id === current.id);
    });
  }

  function applyTheme(theme, save) {
    current = theme;
    root.setAttribute("data-nb-theme", theme.id);
    if (save) send("theme:" + theme.id);
    retheme(theme, false);
    markActive();
  }

  function cardHtml(t) {
    return '<div class="nb-card" data-id="' + t.id + '" style="--card-bg:' + t.bg + ";--card-panel:" + t.panel +
      ";--card-accent:" + t.accent + ";--card-glow:" + t.accent + '55">' +
      '<div class="nb-preview"><div class="nb-bar"></div><div class="nb-line" style="top:26px;width:58%"></div>' +
      '<div class="nb-line" style="top:40px;width:38%"></div><div class="nb-dot"></div></div>' +
      '<span class="nb-check">&#10003;</span>' +
      '<div class="nb-name">' + t.name + '</div><p class="nb-blurb">' + t.blurb + "</p></div>";
  }

  function buildSettings() {
    var overlay = document.createElement("div");
    overlay.id = "nb-overlay";
    overlay.innerHTML =
      '<div id="nb-modal" role="dialog" aria-label="Nebula settings">' +
      '<header><h2>Nebula Settings</h2><button class="nb-close-modal" title="Close">&#10005;</button></header>' +
      '<div class="nb-tabs"><span class="nb-tab">Themes</span></div>' +
      '<p class="nb-hint">Pick a look for the Nebula window. Changes apply instantly.</p>' +
      '<div class="nb-cards">' + THEMES.map(cardHtml).join("") + "</div></div>";
    document.body.appendChild(overlay);

    var close = function () { overlay.classList.remove("open"); };
    overlay.addEventListener("click", function (e) {
      if (e.target === overlay || e.target.closest(".nb-close-modal")) close();
      var card = e.target.closest(".nb-card");
      if (card) applyTheme(THEMES.filter(function (t) { return t.id === card.dataset.id; })[0], true);
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape") close();
    });
    overlay.open = function () {
      markActive();
      overlay.classList.add("open");
    };
    return overlay;
  }

  function buildTitlebar(overlay) {
    var bar = document.createElement("div");
    bar.id = "nb-titlebar";
    bar.innerHTML =
      '<div class="nb-brand"><span class="nb-dot"></span>Nebula</div><div class="nb-spacer"></div>' +
      '<button class="nb-gear" data-act="settings" title="Nebula settings">' + ICON.gear + "</button>" +
      '<button data-act="minimize" title="Minimize">' + ICON.min + "</button>" +
      '<button data-act="maximize" title="Maximize">' + ICON.max + "</button>" +
      '<button class="nb-close" data-act="close" title="Close">' + ICON.close + "</button>";
    document.body.appendChild(bar);

    bar.addEventListener("mousedown", function (e) {
      if (e.button !== 0 || e.target.closest("button")) return;
      send(e.detail === 2 ? "maximize" : "drag");
    });
    bar.addEventListener("click", function (e) {
      var button = e.target.closest("button");
      if (!button) return;
      if (button.dataset.act === "settings") overlay.open();
      else send(button.dataset.act);
    });
  }

  function build() {
    root = document.documentElement;
    var style = document.createElement("style");
    style.id = "nb-chrome-css";
    style.textContent = STYLE;
    document.head.appendChild(style);

    var overlay = buildSettings();
    buildTitlebar(overlay);
    root.setAttribute("data-nb-theme", current.id);
    retheme(current, true);

    // Discord defines its palette in stylesheets that can arrive after this
    // runs and after load. Those are picked up here, so colors never fall back
    // to Discord's default. A class change on <html> is a theme switch: its
    // palette is read again.
    new MutationObserver(function () { schedule(true); })
      .observe(root, { attributes: true, attributeFilter: ["class"] });
    // Our overrides live in the root's inline style. If anything replaces that
    // attribute (window resize or layout changes can do this), the overrides are
    // lost, so reapply them now. Our own writes trigger this too, but a retheme
    // with nothing to change writes nothing, so it settles.
    new MutationObserver(function () { schedule(false); })
      .observe(root, { attributes: true, attributeFilter: ["style"] });
    new MutationObserver(function () { schedule(false); })
      .observe(document.head, { childList: true, subtree: true });
    window.addEventListener("load", function () { schedule(false); });
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", build, { once: true });
  else build();
})();
