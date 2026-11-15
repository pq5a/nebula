// Nebula window chrome: custom title bar, window controls, the settings menu
// (theme and low-power mode) and the account dropdown. Injected before every
// page; runs on discord.com only.
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
    { id: "nebula", name: "Nebula", blurb: "Deep violet with a soft glow", bg: "#0d0a1f", panel: "#15112e", accent: "#a78bfa", text: "#ede9ff", muted: "#8b82b5", hue: 258, tint: 55, accentHue: 262 },
    { id: "midnight", name: "Midnight", blurb: "Cold blue, crisp edges", bg: "#05080f", panel: "#0a1322", accent: "#38bdf8", text: "#dbeafe", muted: "#64748b", hue: 212, tint: 50, accentHue: 198 },
    { id: "ember", name: "Ember", blurb: "Warm dark with orange heat", bg: "#140a08", panel: "#22120d", accent: "#fb923c", text: "#ffedd5", muted: "#9a6b57", hue: 22, tint: 48, accentHue: 24 },
    { id: "aurora", name: "Aurora", blurb: "Teal green, cool and calm", bg: "#07130f", panel: "#0c1d18", accent: "#34d399", text: "#d9fbeb", muted: "#7aa893", hue: 165, tint: 50, accentHue: 160 },
    { id: "rose", name: "Rose", blurb: "Soft pink on deep plum", bg: "#170a12", panel: "#251020", accent: "#f472b6", text: "#ffe4f1", muted: "#b38a9e", hue: 330, tint: 50, accentHue: 330 },
    { id: "forest", name: "Forest", blurb: "Moss green, earthy and quiet", bg: "#09120a", panel: "#0f1d12", accent: "#84cc16", text: "#e6f5d9", muted: "#86a374", hue: 105, tint: 45, accentHue: 100 },
    { id: "solar", name: "Solar", blurb: "Gold on warm charcoal", bg: "#15110a", panel: "#221b0e", accent: "#fbbf24", text: "#fef3c7", muted: "#b5a074", hue: 42, tint: 50, accentHue: 40 },
    { id: "graphite", name: "Graphite", blurb: "Neutral grey, no colour cast", bg: "#0b0c0f", panel: "#16181d", accent: "#94a3b8", text: "#e2e8f0", muted: "#7b8594", hue: 215, tint: 8, accentHue: 215 }
  ];

  // Built from THEMES so the menu, the page styles and the palette never drift apart.
  var THEME_CSS = THEMES.map(function (t) {
    return '[data-nb-theme="' + t.id + '"] { --nb-bg:' + t.bg + "; --nb-panel:" + t.panel + "; --nb-accent:" + t.accent +
      "; --nb-text:" + t.text + "; --nb-muted:" + t.muted + "; --nb-glow:" + t.accent + '59; }';
  });

  var STYLE = [
    ":root { --nb-bg:#0d0a1f; --nb-panel:#15112e; --nb-accent:#a78bfa; --nb-text:#ede9ff; --nb-muted:#8b82b5; --nb-glow:#a78bfa59; }"
  ].concat(THEME_CSS, [
    "#nb-titlebar { position:fixed; top:0; left:0; right:0; height:36px; z-index:2147483647; display:flex; align-items:center;",
    "  background:var(--nb-panel); border-bottom:1px solid var(--nb-glow); color:var(--nb-text); user-select:none; font:600 12px/1 'Segoe UI Variable','Segoe UI',sans-serif; }",
    "#nb-titlebar .nb-brand { display:flex; align-items:center; gap:9px; padding:0 14px; height:100%; letter-spacing:.12em; text-transform:uppercase; font-size:11px; }",
    "#nb-titlebar .nb-dot { width:8px; height:8px; border-radius:50%; background:var(--nb-accent); }",
    "#nb-titlebar .nb-spacer { flex:1; height:100%; }",
    "#nb-titlebar button { all:unset; box-sizing:border-box; width:46px; height:36px; display:grid; place-items:center; color:var(--nb-muted); cursor:default; }",
    "#nb-titlebar button.nb-settings { width:42px; }",
    "#nb-titlebar button:hover { background:rgba(255,255,255,.08); color:var(--nb-text); }",
    "#nb-titlebar button.nb-settings[aria-expanded=\"true\"] { background:rgba(255,255,255,.10); color:var(--nb-text); }",
    "#nb-titlebar button.nb-close:hover { background:#e81123; color:#fff; }",
    "#nb-titlebar svg { width:12px; height:12px; fill:none; stroke:currentColor; stroke-width:1.4; stroke-linecap:round; }",
    "#nb-titlebar .nb-settings svg { width:15px; height:15px; }",
    "html body #app-mount { padding-top:36px !important; box-sizing:border-box !important; }",
    "@keyframes nb-in { from { opacity:0; transform:translateY(-4px); } to { opacity:1; transform:none; } }",
    "#nb-menu, #nb-accounts { position:fixed; top:40px; z-index:2147483646; display:none; flex-direction:column; background:var(--nb-panel); color:var(--nb-text); border-radius:12px; border:1px solid var(--nb-glow); box-shadow:0 12px 32px rgba(0,0,0,.5); font:13px/1.4 'Segoe UI Variable','Segoe UI',sans-serif; overflow:hidden; }",
    "#nb-menu { left:8px; width:264px; }",
    "#nb-accounts { right:8px; width:280px; }",
    "#nb-menu.open, #nb-accounts.open { display:flex; animation:nb-in .12s ease-out; }",
    "#nb-menu .nb-m-head, #nb-accounts .nb-acc-head { padding:12px 14px 6px; font-weight:700; font-size:11px; letter-spacing:.08em; text-transform:uppercase; color:var(--nb-muted); }",
    "#nb-menu .nb-m-list { display:flex; flex-direction:column; padding:0 6px 6px; }",
    "#nb-menu .nb-opt { all:unset; cursor:pointer; display:flex; align-items:center; gap:10px; padding:7px 8px; border-radius:8px; }",
    "#nb-menu .nb-opt:hover { background:rgba(255,255,255,.08); }",
    "#nb-menu .nb-swatch { width:22px; height:22px; flex:none; border-radius:7px; background:var(--sw-bg); box-shadow:inset 0 0 0 2px var(--sw-accent); }",
    "#nb-menu .nb-opt-text { flex:1; min-width:0; }",
    "#nb-menu .nb-opt-name { font-weight:600; }",
    "#nb-menu .nb-opt-blurb { color:var(--nb-muted); font-size:11.5px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }",
    "#nb-menu .nb-tick { width:16px; color:var(--sw-accent); font-weight:800; text-align:center; visibility:hidden; }",
    "#nb-menu .nb-opt.active .nb-tick { visibility:visible; }",
    "#nb-menu .nb-m-sep { height:1px; background:rgba(255,255,255,.07); margin:2px 0 0; }",
    "#nb-menu .nb-m-row { display:flex; align-items:center; justify-content:space-between; gap:12px; padding:12px 14px 4px; cursor:pointer; }",
    "#nb-menu .nb-m-hint { margin:0; padding:0 14px 12px; color:var(--nb-muted); font-size:11.5px; }",
    "#nb-menu .nb-toggle { appearance:none; width:34px; height:18px; flex:none; border-radius:999px; background:rgba(255,255,255,.16); position:relative; cursor:pointer; margin:0; }",
    "#nb-menu .nb-toggle::after { content:''; position:absolute; top:2px; left:2px; width:14px; height:14px; border-radius:50%; background:var(--nb-text); transition:left .15s; }",
    "#nb-menu .nb-toggle:checked { background:var(--nb-accent); }",
    "#nb-menu .nb-toggle:checked::after { left:18px; background:#0b0716; }",
    "#nb-titlebar button.nb-account { width:38px; }",
    "#nb-titlebar .nb-avatar { width:22px; height:22px; border-radius:50%; display:grid; place-items:center; background:var(--nb-glow); color:var(--nb-text); }",
    "#nb-titlebar .nb-avatar svg { width:13px; height:13px; }",
    "#nb-accounts .nb-acc-empty { padding:4px 14px 12px; color:var(--nb-muted); font-size:12px; }",
    "#nb-accounts .nb-acc-row { display:flex; align-items:center; gap:8px; padding:4px 6px 4px 8px; }",
    "#nb-accounts .nb-acc-switch { all:unset; flex:1; display:flex; align-items:center; gap:10px; min-width:0; padding:7px 6px; border-radius:8px; cursor:pointer; }",
    "#nb-accounts .nb-acc-switch:hover { background:rgba(255,255,255,.08); }",
    "#nb-accounts .nb-acc-letter { width:26px; height:26px; flex:none; border-radius:50%; display:grid; place-items:center; background:var(--nb-accent); color:#0b0716; font-weight:800; font-size:12px; }",
    "#nb-accounts .nb-acc-name { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-weight:600; }",
    "#nb-accounts .nb-acc-remove { all:unset; cursor:pointer; color:var(--nb-muted); padding:4px 7px; border-radius:6px; font-size:13px; }",
    "#nb-accounts .nb-acc-remove:hover { color:#fff; background:#e81123; }",
    "#nb-accounts .nb-acc-add { border-top:1px solid rgba(255,255,255,.07); padding:12px 14px 14px; display:flex; flex-direction:column; gap:8px; }",
    "#nb-accounts .nb-acc-hint { color:var(--nb-muted); font-size:11.5px; }",
    "#nb-accounts .nb-acc-form { display:flex; gap:6px; }",
    "#nb-accounts input { all:unset; box-sizing:border-box; flex:1; min-width:0; padding:7px 10px; border-radius:8px; background:rgba(255,255,255,.06); border:1px solid rgba(255,255,255,.1); color:var(--nb-text); }",
    "#nb-accounts input:focus { border-color:var(--nb-accent); }",
    "#nb-accounts .nb-acc-save { all:unset; cursor:pointer; padding:7px 12px; border-radius:8px; background:var(--nb-accent); color:#0b0716; font-weight:700; }",
    "#nb-accounts .nb-acc-save:hover { filter:brightness(1.1); }",
    "#nb-accounts .nb-acc-error { color:#fca5a5; font-size:12px; min-height:0; }"
  ]).join("\n");

  var ICON = {
    min: '<svg viewBox="0 0 12 12"><path d="M1.5 6h9"/></svg>',
    max: '<svg viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" rx="1.5"/></svg>',
    close: '<svg viewBox="0 0 12 12"><path d="M2 2l8 8M10 2l-8 8"/></svg>',
    user: '<svg viewBox="0 0 16 16"><circle cx="8" cy="5.5" r="2.8"/><path d="M2.8 14c.6-2.6 2.6-4 5.2-4s4.6 1.4 5.2 4"/></svg>',
    gear: '<svg viewBox="0 0 16 16"><circle cx="8" cy="8" r="2.3"/><path d="M8 1.6v2.1M8 12.3v2.1M1.6 8h2.1M12.3 8h2.1M3.4 3.4l1.5 1.5M11.1 11.1l1.5 1.5M3.4 12.6l1.5-1.5M11.1 4.9l1.5-1.5"/></svg>'
  };

  function send(cmd) {
    if (window.ipc) window.ipc.postMessage(cmd);
  }

  // ---- Low-power mode -------------------------------------------------------
  // While Nebula is tabbed out (window hidden or not focused), requestAnimationFrame
  // callbacks run 10 times a second instead of at the display rate. Discord's
  // animations and its own loops use rAF, so this cuts their work in the background.
  // Set by Rust from the saved choice; on by default.
  var throttleOn = window.__nebulaThrottle !== false;
  var SLOW_MS = 100;
  var nativeRaf = window.requestAnimationFrame.bind(window);
  var nativeCaf = window.cancelAnimationFrame.bind(window);
  var slowTimers = {};
  var slowSeq = 0;
  function tabbedOut() {
    return document.hidden || !document.hasFocus();
  }
  window.requestAnimationFrame = function (cb) {
    if (!throttleOn || !tabbedOut()) return nativeRaf(cb);
    // Negative ids keep these apart from the browser's own frame request ids.
    var id = --slowSeq;
    slowTimers[id] = setTimeout(function () {
      delete slowTimers[id];
      cb(performance.now());
    }, SLOW_MS);
    return id;
  };
  window.cancelAnimationFrame = function (id) {
    if (id < 0) {
      clearTimeout(slowTimers[id]);
      delete slowTimers[id];
    } else {
      nativeCaf(id);
    }
  };

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
  var styleObserver = null;
  function schedule(fresh) {
    scheduledFresh = scheduledFresh || fresh;
    if (scheduled) return;
    scheduled = requestAnimationFrame(function () {
      var freshNow = scheduledFresh;
      scheduled = 0;
      scheduledFresh = false;
      retheme(current, freshNow);
      // Our own writes to the root's inline style would otherwise queue
      // another pass, and each pass writes again. Drop those records.
      if (styleObserver) styleObserver.takeRecords();
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

  // ---- Settings menu -------------------------------------------------------
  var menu = null;
  var accountsPanel = null;

  function markActive() {
    if (!menu) return;
    menu.querySelectorAll(".nb-opt").forEach(function (opt) {
      var on = opt.dataset.id === current.id;
      opt.classList.toggle("active", on);
      opt.setAttribute("aria-checked", String(on));
    });
  }

  function applyTheme(theme, save) {
    current = theme;
    root.setAttribute("data-nb-theme", theme.id);
    if (save) send("theme:" + theme.id);
    retheme(theme, false);
    markActive();
  }

  function el(tag, cls) {
    var node = document.createElement(tag);
    node.className = cls;
    return node;
  }

  function buildMenu() {
    var panel = el("div", "");
    panel.id = "nb-menu";
    panel.setAttribute("role", "menu");
    var options = THEMES.map(function (t) {
      return '<button class="nb-opt" role="menuitemradio" data-id="' + t.id + '" style="--sw-bg:' + t.bg +
        ";--sw-accent:" + t.accent + '"><span class="nb-swatch"></span><span class="nb-opt-text">' +
        '<span class="nb-opt-name">' + t.name + '</span><span class="nb-opt-blurb">' + t.blurb + "</span></span>" +
        '<span class="nb-tick">&#10003;</span></button>';
    }).join("");
    panel.innerHTML =
      '<div class="nb-m-head">Theme</div><div class="nb-m-list">' + options + "</div>" +
      '<div class="nb-m-sep"></div><div class="nb-m-head">Power</div>' +
      '<label class="nb-m-row"><span>Slow to 10 FPS when tabbed out</span>' +
      '<input class="nb-toggle" type="checkbox" role="switch"' + (throttleOn ? " checked" : "") + "></label>" +
      '<p class="nb-m-hint">Saves CPU and GPU while Nebula is in the background.</p>';
    document.body.appendChild(panel);

    panel.addEventListener("click", function (e) {
      var opt = e.target.closest(".nb-opt");
      if (!opt) return;
      applyTheme(THEMES.filter(function (t) { return t.id === opt.dataset.id; })[0], true);
    });
    panel.querySelector(".nb-toggle").addEventListener("change", function (e) {
      throttleOn = e.target.checked;
      send("throttle:" + (throttleOn ? "on" : "off"));
    });
    markActive();
    return panel;
  }

  // ---- Account dropdown ----------------------------------------------------
  // Saved accounts hold an email and password, kept by Rust (encrypted). The page
  // only sees names and emails. A switch opens Discord's login page, and
  // window.__nebulaFill fills in its form.
  var accountList = [];

  // Sets a React-controlled input's value so the framework sees the change.
  function setInputValue(input, value) {
    var setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set;
    setter.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  }

  // Called by Rust on the login page with the chosen account. The form may
  // render late, so it retries for a few seconds, then submits.
  window.__nebulaFill = function (creds) {
    var tries = 0;
    // Discord's form only: skip Nebula's own account inputs.
    function find(selector) {
      var all = document.querySelectorAll(selector);
      for (var i = 0; i < all.length; i++) {
        if (!all[i].closest("#nb-accounts")) return all[i];
      }
      return null;
    }
    var timer = setInterval(function () {
      tries++;
      var email = find('input[name="email"]') || find('input[type="email"]');
      var pass = find('input[name="password"]') || find('input[type="password"]');
      if (email && pass) {
        clearInterval(timer);
        setInputValue(email, creds.email);
        setInputValue(pass, creds.password);
        var submit = document.querySelector('button[type="submit"]');
        if (submit) setTimeout(function () { submit.click(); }, 150);
      } else if (tries > 40) {
        clearInterval(timer);
      }
    }, 250);
  };

  function buildAccounts() {
    var panel = el("div", "");
    panel.id = "nb-accounts";
    panel.innerHTML =
      '<div class="nb-acc-head">Accounts</div>' +
      '<div class="nb-acc-list"></div>' +
      '<div class="nb-acc-add">' +
      '<div class="nb-acc-form"><input class="nb-in-name" type="text" maxlength="32" placeholder="Account name" spellcheck="false"></div>' +
      '<div class="nb-acc-form"><input class="nb-in-email" type="email" maxlength="254" placeholder="Email" spellcheck="false" autocomplete="off"></div>' +
      '<div class="nb-acc-form"><input class="nb-in-pass" type="password" maxlength="1024" placeholder="Password" autocomplete="off">' +
      '<button class="nb-acc-save">Save</button></div><div class="nb-acc-error"></div></div>';
    document.body.appendChild(panel);

    var nameInput = panel.querySelector(".nb-in-name");
    var emailInput = panel.querySelector(".nb-in-email");
    var passInput = panel.querySelector(".nb-in-pass");
    var errorBox = panel.querySelector(".nb-acc-error");

    function setError(text) {
      errorBox.textContent = text || "";
    }

    function renderList() {
      var box = panel.querySelector(".nb-acc-list");
      box.textContent = "";
      if (!accountList.length) {
        var empty = el("div", "nb-acc-empty");
        empty.textContent = "No saved accounts yet.";
        box.appendChild(empty);
        return;
      }
      accountList.forEach(function (a) {
        var row = el("div", "nb-acc-row");
        var pick = el("button", "nb-acc-switch");
        pick.dataset.act = "switch";
        pick.dataset.id = a.id;
        pick.title = "Switch to " + a.name;
        var letter = el("span", "nb-acc-letter");
        letter.textContent = (a.name.charAt(0) || "?").toUpperCase();
        var name = el("span", "nb-acc-name");
        name.textContent = a.name;
        pick.appendChild(letter);
        pick.appendChild(name);
        var remove = el("button", "nb-acc-remove");
        remove.dataset.act = "remove";
        remove.dataset.id = a.id;
        remove.title = "Remove " + a.name;
        remove.textContent = "✕";
        row.appendChild(pick);
        row.appendChild(remove);
        box.appendChild(row);
      });
    }

    // Called by Rust after every account change, with the new list.
    window.__nebulaSetAccounts = function (status) {
      accountList = status.accounts || [];
      setError(status.error);
      if (status.added) {
        nameInput.value = "";
        emailInput.value = "";
        passInput.value = "";
      }
      renderList();
    };

    function save() {
      var name = nameInput.value.trim();
      var email = emailInput.value.trim();
      var password = passInput.value;
      if (!name) return setError("Enter a name for this account.");
      if (!email) return setError("Enter the email for this account.");
      if (!password) return setError("Enter the password for this account.");
      setError("");
      send(JSON.stringify({ op: "add", name: name, email: email, password: password }));
    }

    panel.addEventListener("click", function (e) {
      var button = e.target.closest("button");
      if (e.target.closest(".nb-acc-save")) return save();
      if (!button || !button.dataset.id) return;
      if (button.dataset.act === "switch") {
        close();
        send(JSON.stringify({ op: "switch", id: button.dataset.id }));
      } else if (button.dataset.act === "remove") {
        send(JSON.stringify({ op: "remove", id: button.dataset.id }));
      }
    });
    panel.addEventListener("keydown", function (e) {
      if (e.key === "Enter" && e.target.tagName === "INPUT") save();
    });

    function close() {
      panel.classList.remove("open");
    }
    function open() {
      closeMenu();
      setError("");
      panel.classList.add("open");
      send(JSON.stringify({ op: "list" }));
    }
    panel.open = open;
    panel.close = close;
    panel.toggle = function () {
      if (panel.classList.contains("open")) close();
      else open();
    };
    return panel;
  }

  // Opening and closing the two dropdowns.
  function closeMenu() {
    if (!menu) return;
    menu.classList.remove("open");
    var btn = document.querySelector("#nb-titlebar .nb-settings");
    if (btn) btn.setAttribute("aria-expanded", "false");
  }
  function toggleMenu() {
    var open = !menu.classList.contains("open");
    if (open) accountsPanel.close();
    menu.classList.toggle("open", open);
    document.querySelector("#nb-titlebar .nb-settings").setAttribute("aria-expanded", String(open));
  }

  function buildTitlebar() {
    var bar = document.createElement("div");
    bar.id = "nb-titlebar";
    bar.innerHTML =
      '<button class="nb-settings" data-act="settings" title="Nebula settings" aria-haspopup="menu" aria-expanded="false">' + ICON.gear + "</button>" +
      '<div class="nb-brand"><span class="nb-dot"></span>Nebula</div><div class="nb-spacer"></div>' +
      '<button class="nb-account" data-act="accounts" title="Accounts"><span class="nb-avatar">' + ICON.user + "</span></button>" +
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
      if (button.dataset.act === "settings") toggleMenu();
      else if (button.dataset.act === "accounts") {
        closeMenu();
        accountsPanel.toggle();
      } else send(button.dataset.act);
    });
  }

  // Clicks outside an open dropdown close it. Escape closes both.
  function wireDismiss() {
    document.addEventListener("mousedown", function (e) {
      if (menu.classList.contains("open") && !menu.contains(e.target) && !e.target.closest(".nb-settings")) closeMenu();
      if (accountsPanel.classList.contains("open") && !accountsPanel.contains(e.target) && !e.target.closest(".nb-account")) accountsPanel.close();
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape") {
        closeMenu();
        accountsPanel.close();
      }
    });
  }

  function build() {
    root = document.documentElement;
    var style = document.createElement("style");
    style.id = "nb-chrome-css";
    style.textContent = STYLE;
    document.head.appendChild(style);

    menu = buildMenu();
    accountsPanel = buildAccounts();
    buildTitlebar();
    wireDismiss();
    root.setAttribute("data-nb-theme", current.id);
    retheme(current, true);
    // Ask Rust for the saved accounts. It answers through __nebulaSetAccounts.
    send(JSON.stringify({ op: "list" }));
    // On Discord's login page, ask Rust for a pending account to fill in.
    if (location.pathname.indexOf("/login") === 0) send(JSON.stringify({ op: "login-page" }));

    // Discord defines its palette in stylesheets that can arrive after this
    // runs and after load. Those are picked up here, so colors never fall back
    // to Discord's default. A class change on <html> is a theme switch: its
    // palette is read again.
    // Discord often rewrites the class attribute with the same value. Only a
    // real change in the class list is a theme switch, so only that triggers a
    // full palette re-read; an identical rewrite would just repaint everything.
    var lastClass = root.className;
    new MutationObserver(function () {
      if (root.className === lastClass) return;
      lastClass = root.className;
      schedule(true);
    }).observe(root, { attributes: true, attributeFilter: ["class"] });
    // Our overrides live in the root's inline style. If anything replaces that
    // attribute (window resize or layout changes can do this), the overrides are
    // lost, so reapply them now. The scheduled pass drops the records of its own
    // writes, so this does not loop.
    styleObserver = new MutationObserver(function () { schedule(false); });
    styleObserver.observe(root, { attributes: true, attributeFilter: ["style"] });
    // Only new stylesheets matter for the palette. Other head churn is ignored.
    new MutationObserver(function (records) {
      for (var i = 0; i < records.length; i++) {
        var nodes = records[i].addedNodes;
        for (var j = 0; j < nodes.length; j++) {
          var name = nodes[j].nodeName;
          if (name === "STYLE" || name === "LINK") return schedule(true);
        }
      }
    }).observe(document.head, { childList: true });
    window.addEventListener("load", function () { schedule(true); });
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", build, { once: true });
  else build();
})();
