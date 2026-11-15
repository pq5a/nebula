// Nebula window chrome: custom title bar, window controls and the settings
// modal with theme picker. Injected before every page; runs on discord.com only.
(function () {
  "use strict";
  var host = location.hostname;
  if (host !== "discord.com" && !host.endsWith(".discord.com")) return;
  if (window.__nebulaChrome) return;
  window.__nebulaChrome = true;

  var THEME_KEY = "nebula-theme";
  var THEMES = [
    { id: "nebula", name: "Nebula", blurb: "Deep violet with a soft glow", bg: "#0d0a1f", panel: "#15112e", accent: "#a78bfa" },
    { id: "midnight", name: "Midnight", blurb: "Cold blue, crisp edges", bg: "#05080f", panel: "#0a1322", accent: "#38bdf8" },
    { id: "ember", name: "Ember", blurb: "Warm dark with orange heat", bg: "#140a08", panel: "#22120d", accent: "#fb923c" }
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
    "#nb-titlebar button { all:unset; box-sizing:border-box; width:46px; height:36px; display:grid; place-items:center; color:var(--nb-muted); cursor:default; transition:background .12s,color .12s; }",
    "#nb-titlebar button.nb-gear { width:42px; }",
    "#nb-titlebar button:hover { background:rgba(255,255,255,.08); color:var(--nb-text); }",
    "#nb-titlebar button.nb-close:hover { background:#e81123; color:#fff; }",
    "#nb-titlebar svg { width:12px; height:12px; fill:none; stroke:currentColor; stroke-width:1.4; stroke-linecap:round; }",
    "#nb-titlebar .nb-gear svg { width:15px; height:15px; }",
    "html body #app-mount { padding-top:36px !important; box-sizing:border-box !important; }",
    "#nb-overlay { position:fixed; inset:0; z-index:2147483646; display:none; align-items:center; justify-content:center; background:rgba(3,2,10,.6); backdrop-filter:blur(6px); font:13px/1.4 'Segoe UI Variable','Segoe UI',sans-serif; }",
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
    "#nb-modal .nb-card { position:relative; cursor:pointer; padding:10px; border-radius:14px; background:rgba(255,255,255,.03); border:1px solid rgba(255,255,255,.07); transition:transform .18s,box-shadow .18s,border-color .18s; }",
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

  function applyTheme(id) {
    var theme = THEMES.filter(function (t) { return t.id === id; })[0] || THEMES[0];
    document.documentElement.setAttribute("data-nb-theme", theme.id);
    localStorage.setItem(THEME_KEY, theme.id);
    document.querySelectorAll("#nb-modal .nb-card").forEach(function (card) {
      card.classList.toggle("active", card.dataset.id === theme.id);
    });
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
      if (card) applyTheme(card.dataset.id);
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape") close();
    });
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
      if (button.dataset.act === "settings") overlay.classList.add("open");
      else send(button.dataset.act);
    });
  }

  function build() {
    var style = document.createElement("style");
    style.id = "nb-chrome-css";
    style.textContent = STYLE;
    document.head.appendChild(style);

    var overlay = buildSettings();
    buildTitlebar(overlay);
    applyTheme(localStorage.getItem(THEME_KEY) || THEMES[0].id);
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", build, { once: true });
  else build();
})();
