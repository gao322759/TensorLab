/* ============================================================================
   TensorLab UI prototype logic (MATLAB R2026a paradigm study, clean-room)
   ========================================================================== */
(function () {
  "use strict";

  const $ = (sel, root) => (root || document).querySelector(sel);
  const $$ = (sel, root) => Array.from((root || document).querySelectorAll(sel));

  /* ------------------------------------------------------------- ribbon data
     Groups follow the measured R2026a ribbon (design-tokens.md §4) with
     English placeholder labels (decision: English copy).
     Simplified to FOUR tabs (product decision 2026-11-11): HOME / EDITOR /
     VIEW / HELP.  PLOT-related groups folded into HOME; APPS and PUBLISH are
     dropped because this product has no App Designer / Publish equivalent;
     the SIMULINK group is dropped too (out of scope per AGENTS.md). */
  const RIBBONS = {
    HOME: [
      { label: "FILE", items: [
        { ico: "ico-new", text: "New", menu: true, menuId: "new" },
        { ico: "ico-open", text: "Open", menu: true },
        { ico: "ico-find", text: "Find Files" }
      ]},
      { label: "DATA", items: [
        { ico: "ico-file", text: "Go To File" },
        { ico: "ico-import", text: "Import Data" },
        { ico: "ico-save", text: "Save Workspace" },
        { ico: "ico-clear", text: "Clear Workspace", act: "clearWs" }
      ]},
      { label: "CODE", items: [
        { ico: "ico-timer", text: "Run && Time" },
        { ico: "ico-star", text: "Favorites" },
        { ico: "ico-clear", text: "Clear Commands", act: "clc" }
      ]},
      { label: "PLOT", items: [
        { ico: "ico-plot", text: "Plot", big: true },
        { ico: "ico-plot", text: "Scatter" },
        { ico: "ico-plot", text: "Bar" },
        { ico: "ico-plot", text: "Stairs" }
      ]},
      { label: "ENVIRONMENT", items: [
        { ico: "ico-layout", text: "Layout", menu: true, menuId: "layout" },
        { ico: "ico-path", text: "Set Path" },
        { ico: "ico-addon", text: "Add-Ons" },
        { ico: "ico-parallel", text: "Parallel", menu: true }
      ]}
    ],
    EDITOR: [
      { label: "FILE", items: [
        { ico: "ico-new", text: "New", menu: true, menuId: "new" },
        { ico: "ico-open", text: "Open" },
        { ico: "ico-save", text: "Save", act: "save" },
        { ico: "ico-print", text: "Print" },
        { ico: "ico-file", text: "Compare" },
        { ico: "ico-file", text: "Go To" }
      ]},
      { label: "NAVIGATE", items: [
        { ico: "ico-find", text: "Find" },
        { ico: "ico-star", text: "Bookmarks" },
        { ico: "ico-sort", text: "Navigate" }
      ]},
      { label: "CODE", items: [
        { ico: "ico-code", text: "Refactor" },
        { ico: "ico-help", text: "Code Issues" },
        { ico: "ico-stop", text: "Debug" }
      ]},
      { label: "TEST", items: [
        { ico: "ico-find", text: "Find Tests" },
        { ico: "ico-run", text: "Run Section" },
        { ico: "ico-step", text: "Run && Advance" },
        { ico: "ico-step", text: "Run to End" }
      ]},
      { label: "RUN", items: [
        { ico: "ico-run", text: "Run", big: true, green: true, menu: true, act: "run" },
        { ico: "ico-step", text: "Step" },
        { ico: "ico-stop", text: "Stop", disabled: true }
      ]}
    ],
    VIEW: [
      { label: "PANELS", items: [
        { ico: "ico-layout", text: "Layout", big: true, menu: true, menuId: "layout" },
        { ico: "ico-table", text: "Workspace" },
        { ico: "ico-folder", text: "Current Folder" },
        { ico: "ico-history", text: "History" }
      ]},
      { label: "THEME", items: [
        { ico: "ico-gear", text: "Light", act: "themeLight" },
        { ico: "ico-gear", text: "Dark", act: "themeDark" }
      ]},
      { label: "SETTINGS", items: [
        { ico: "ico-gear", text: "Preferences", big: true, act: "prefs" }
      ]}
    ],
    HELP: [
      { label: "RESOURCES", items: [
        { ico: "ico-support", text: "Support" },
        { ico: "ico-feedback", text: "Feedback" },
        { ico: "ico-book", text: "Learn" },
        { ico: "ico-help", text: "Help", big: true }
      ]},
      { label: "DOCUMENTATION", items: [
        { ico: "ico-book", text: "Quick Start" },
        { ico: "ico-code", text: "Examples" },
        { ico: "ico-info", text: "Release Notes" }
      ]},
      { label: "PRODUCT", items: [
        { ico: "ico-info", text: "About", act: "about" }
      ]}
    ]
  };

  const state = {
    activeTab: "HOME",
    theme: "light",
    vars: [
      { name: "a", value: "3", size: "1x1", cls: "double" },
      { name: "b", value: "2", size: "1x1", cls: "double" },
      { name: "c", value: "3", size: "1x1", cls: "double" },
      { name: "sig", value: "1x4096", size: "1x4096", cls: "double" }
    ],
    files: [
      { name: "..", dir: true },
      { name: "benchmarks", dir: true },
      { name: "demos", dir: true },
      { name: "toolbox", dir: true },
      { name: "fft_demo.m", dir: false },
      { name: "plot_result.m", dir: false },
      { name: "stage8_benchmark.m", dir: false },
      { name: "README.md", dir: false }
    ],
    docs: {
      "untitled.m": [
        "% TensorLab scratch pad",
        "a = 1;",
        "b = 2;",
        "c = a + b;",
        "disp(c)"
      ].join("\n"),
      "plot_result.m": [
        "% FFT demo: spectrum of a two-tone signal",
        "n = 4096;",
        "t = linspace(0, 1, n);",
        "sig = sin(2*pi*50*t) + 0.5*sin(2*pi*120*t);",
        "X = fft(sig);",
        "f = (0:n-1)*(1/n);",
        "plot(f, abs(X))",
        "title('Spectrum')",
        "xlabel('Hz')"
      ].join("\n")
    },
    openDocs: ["untitled.m", "plot_result.m"],
    activeDoc: "untitled.m",
    cmdHistory: [],
    histIdx: -1,
    editing: 60
  };

  /* -------------------------------------------------------------- utilities */
  function toast(msg) {
    const t = $("#toast");
    t.textContent = msg;
    t.classList.add("show");
    clearTimeout(toast._h);
    toast._h = setTimeout(() => t.classList.remove("show"), 1400);
  }

  function setStatus(txt) { $("#stStatus").textContent = txt; }

  /* ------------------------------------------------------------- ribbon render */
  function renderRibbonTabs() {
    const wrap = $("#ribbonTabs");
    wrap.innerHTML = "";
    Object.keys(RIBBONS).forEach((name) => {
      const el = document.createElement("div");
      el.className = "rtab" + (name === state.activeTab ? " active" : "");
      el.textContent = name;
      el.onclick = () => { state.activeTab = name; renderRibbonTabs(); renderRibbonBody(); };
      wrap.appendChild(el);
    });
  }

  function icon(name, size) {
    return `<svg class="ico" width="${size || 20}" height="${size || 20}"><use href="#${name}"/></svg>`;
  }

  function renderRibbonBody() {
    const body = $("#ribbonBody");
    body.innerHTML = "";
    (RIBBONS[state.activeTab] || []).forEach((group) => {
      const g = document.createElement("div");
      g.className = "rgroup";
      const items = document.createElement("div");
      items.className = "rgroup-items";
      group.items.forEach((it) => {
        const b = document.createElement("div");
        b.className = "rbtn" + (it.big ? " big" : "") + (it.green ? " green" : "") +
          (it.disabled ? " disabled" : "") + (it.menu ? " has-menu" : "");
        b.innerHTML = icon(it.ico || "ico-file", it.big ? 26 : 20) + `<span>${it.text}</span>`;
        if (!it.disabled) {
          b.onclick = (ev) => {
            // stop the click from reaching the document-level "click outside"
            // handler, which would otherwise close the popup we are opening
            ev.stopPropagation();
            if (it.act) return doAction(it.act);
            if (it.menuId) return openMenu(it.menuId, ev.currentTarget);
            toast(it.text + " — prototype placeholder");
          };
        }
        items.appendChild(b);
      });
      const lbl = document.createElement("div");
      lbl.className = "rgroup-label";
      lbl.textContent = group.label;
      g.appendChild(items);
      g.appendChild(lbl);
      body.appendChild(g);
    });
  }

  /* -------------------------------------------------------------------- menus */
  let openPopup = null;
  function closePopup() {
    if (openPopup) { openPopup.remove(); openPopup = null; }
  }
  document.addEventListener("click", (e) => {
    if (openPopup && !openPopup.contains(e.target)) closePopup();
  });

  function openMenu(id, anchor) {
    closePopup();
    const p = document.createElement("div");
    p.className = "popup";
    const items = id === "layout"
      ? [
          { t: "Default", h: "Workspace right", fn: () => applyLayout("default") },
          { t: "Workspace bottom-left", h: "as captured", fn: () => applyLayout("stacked") },
          { sep: true },
          { t: "Reset panels", fn: () => applyLayout(state.layout || "default") }
        ]
      : [
          { t: "Script", fn: () => newDoc("untitled2.m") },
          { t: "Live Script", fn: () => toast("Live Script is out of scope") },
          { t: "Figure", fn: () => makeFigure() },
          { sep: true },
          { t: "Folder", fn: () => toast("New folder — placeholder") }
        ];
    if (id === "layout") {
      const title = document.createElement("div");
      title.className = "popup-title";
      title.textContent = "Layout";
      p.appendChild(title);
    }
    items.forEach((it) => {
      if (it.sep) { const s = document.createElement("div"); s.className = "popup-sep"; p.appendChild(s); return; }
      const row = document.createElement("div");
      row.className = "popup-item";
      row.innerHTML = `<span>${it.t}</span>` + (it.h ? `<span class="hint">${it.h}</span>` : "");
      row.onclick = () => { closePopup(); it.fn(); };
      p.appendChild(row);
    });
    document.body.appendChild(p);
    const r = anchor.getBoundingClientRect();
    p.style.left = Math.min(r.left, window.innerWidth - 220) + "px";
    p.style.top = (r.bottom + 2) + "px";
    openPopup = p;
  }

  /* --------------------------------------------------------- folder / workspace */
  function renderFolder() {
    const wrap = $("#folderTree");
    wrap.innerHTML = "";
    state.files.forEach((f, i) => {
      const row = document.createElement("div");
      row.className = "tree-row" + (i === 0 ? " selected" : "");
      row.innerHTML = icon(f.dir ? "ico-folder" : "ico-file", 15).replace('class="ico"', 'class="ico ' + (f.dir ? "folder" : "") + '"') +
        `<span>${f.name}</span>`;
      row.onclick = () => {
        $$(".tree-row", wrap).forEach((x) => x.classList.remove("selected"));
        row.classList.add("selected");
        setStatus("Selected: " + f.name);
      };
      wrap.appendChild(row);
    });
  }

  let wsSort = { key: null, dir: 1 };
  function renderWorkspace() {
    const body = $("#wsBody");
    const cols = [
      { k: "name", t: "Name", cls: "name" },
      { k: "value", t: "Value" },
      { k: "size", t: "Size" },
      { k: "cls", t: "Class", cls: "cls" }
    ];
    let rows = state.vars.slice();
    if (wsSort.key) {
      rows.sort((x, y) => {
        const a = String(x[wsSort.key === "cls" ? "cls" : wsSort.key]);
        const b = String(y[wsSort.key === "cls" ? "cls" : wsSort.key]);
        return a.localeCompare(b, undefined, { numeric: true }) * wsSort.dir;
      });
    }
    let html = "<table class='vars'><thead><tr>";
    cols.forEach((c) => { html += `<th data-k="${c.k}">${c.t}</th>`; });
    html += "</tr></thead><tbody>";
    rows.forEach((r) => {
      html += `<tr><td class="${cols[0].cls}">${r.name}</td><td>${r.value}</td>` +
        `<td>${r.size}</td><td class="${cols[3].cls}">${r.cls}</td></tr>`;
    });
    html += "</tbody></table>";
    body.innerHTML = html;
    $$("th", body).forEach((th) => {
      th.onclick = () => {
        const k = th.getAttribute("data-k");
        if (wsSort.key === k) wsSort.dir *= -1;
        else { wsSort.key = k; wsSort.dir = 1; }
        renderWorkspace();
      };
    });
    $$("tbody tr", body).forEach((tr) => {
      tr.onclick = () => {
        $$("tbody tr", body).forEach((x) => x.classList.remove("selected"));
        tr.classList.add("selected");
      };
    });
  }

  /* ------------------------------------------------------------------- editor */
  const KEYWORDS = /\b(for|while|if|elseif|else|end|function|return|break|continue|switch|case|otherwise|try|catch|global|persistent|classdef|properties|methods|arguments|parfor|spmd)\b/g;
  const BUILTINS = /\b(disp|fft|plot|linspace|abs|sin|cos|title|xlabel|ylabel|zeros|ones|size|numel|length|mean|sum|max|min|sqrt|exp|log|figure|hold|grid|legend|sprintf|fprintf|error|warning)\b/g;

  function escapeHtml(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function highlightLine(line) {
    let s = escapeHtml(line);
    // comment: everything after % (outside a string, simplified)
    const cm = s.indexOf("%");
    let code = cm >= 0 ? s.slice(0, cm) : s;
    let comment = cm >= 0 ? s.slice(cm) : "";
    // strings
    code = code.replace(/'([^']*)'/g, '<span class="tok-str">\'$1\'</span>');
    // keywords + builtins (skip inside spans: acceptable simplification for a prototype)
    code = code.replace(KEYWORDS, '<span class="tok-kw">$1</span>');
    code = code.replace(BUILTINS, '<span class="tok-fn">$1</span>');
    // numbers
    code = code.replace(/\b(\d+(?:\.\d+)?(?:e[+-]?\d+)?)\b/gi, '<span class="tok-num">$1</span>');
    if (comment) comment = '<span class="tok-cm">' + comment + '</span>';
    return code + comment;
  }

  function renderEditor() {
    const src = state.docs[state.activeDoc] || "";
    const lines = src.split("\n");
    const code = $("#code");
    const gutter = $("#gutter");
    code.innerHTML = lines.map((l, i) =>
      `<span class="ln${i === 4 ? " cur" : ""}">${highlightLine(l) || "&nbsp;"}</span>`
    ).join("");
    gutter.innerHTML = lines.map((_, i) => String(i + 1)).join("\n");
    $("#docTitle").textContent = state.activeDoc;
    // tabs
    const tabs = $("#editorTabs");
    tabs.innerHTML = "";
    state.openDocs.forEach((name) => {
      const t = document.createElement("span");
      t.className = "ptab" + (name === state.activeDoc ? " active" : "");
      t.innerHTML = `<span>${name}</span><span class="close" title="Close">✕</span>`;
      t.onclick = (e) => {
        if (e.target.classList.contains("close")) {
          state.openDocs = state.openDocs.filter((x) => x !== name);
          if (state.activeDoc === name) state.activeDoc = state.openDocs[0] || "";
          renderEditor();
          return;
        }
        state.activeDoc = name;
        renderEditor();
      };
      tabs.appendChild(t);
    });
    const add = document.createElement("span");
    add.className = "ptab";
    add.style.cursor = "pointer";
    add.innerHTML = `<span>＋</span>`;
    add.onclick = () => newDoc("untitled" + (Object.keys(state.docs).length + 1) + ".m");
    tabs.appendChild(add);
  }

  function newDoc(name) {
    state.docs[name] = "% New script";
    state.openDocs.push(name);
    state.activeDoc = name;
    renderEditor();
    toast("Created " + name);
  }

  /* ---------------------------------------------------------- command window */
  function cmdAppend(echo, out, isErr) {
    const log = $("#cmdLog");
    if (echo) {
      const d = document.createElement("div");
      d.className = "cmd-echo";
      d.textContent = ">> " + echo;
      log.appendChild(d);
    }
    if (out) {
      const d = document.createElement("div");
      d.className = isErr ? "cmd-err" : "cmd-out";
      d.textContent = out;
      log.appendChild(d);
    }
    const body = $("#cmdBody");
    body.scrollTop = body.scrollHeight;
  }

  function fmtNum(n) {
    if (Number.isInteger(n)) return String(n);
    return String(Number(n.toFixed(6)));
  }

  function execCommand(raw) {
    const cmd = raw.trim();
    if (!cmd) return;
    state.cmdHistory.push(cmd);
    state.histIdx = state.cmdHistory.length;

    const lower = cmd.toLowerCase();
    if (lower === "clc") { $("#cmdLog").innerHTML = ""; return; }
    if (lower === "who") {
      const names = state.vars.map((v) => v.name).join("  ");
      cmdAppend(cmd, names || "No variables.");
      return;
    }
    if (lower === "clear" || lower === "clear all") {
      state.vars = [];
      renderWorkspace();
      cmdAppend(cmd, "");
      setStatus("Workspace cleared");
      return;
    }
    if (lower.startsWith("help")) {
      cmdAppend(cmd, "TensorLab prototype commands:\n" +
        "  plot(<expr>)  open a docked figure\n" +
        "  who           list workspace variables\n" +
        "  clear         clear the workspace\n" +
        "  clc           clear the command window\n" +
        "  a = 3         assign a variable\n" +
        "  prefs         open Preferences");
      return;
    }
    if (lower === "prefs" || lower === "preferences") { openPrefs(); cmdAppend(cmd, ""); return; }

    // assignment: name = <number>
    const m = cmd.match(/^([A-Za-z]\w*)\s*=\s*(-?\d+(?:\.\d+)?)\s*;?$/);
    if (m) {
      const name = m[1], val = Number(m[2]);
      const existing = state.vars.find((v) => v.name === name);
      if (existing) { existing.value = fmtNum(val); }
      else { state.vars.push({ name: name, value: fmtNum(val), size: "1x1", cls: "double" }); }
      renderWorkspace();
      if (!cmd.trim().endsWith(";")) cmdAppend(cmd, "\n" + name + " =\n\n     " + fmtNum(val));
      else cmdAppend(cmd, "");
      return;
    }

    // plot(...)
    if (/^(plot|figure|surf|scatter|bar)\s*\(/.test(lower) || lower === "figure") {
      makeFigure(cmd);
      cmdAppend(cmd, "");
      return;
    }

    if (/^[\w.]+\s*\(/.test(cmd)) {
      cmdAppend(cmd, "ans =\n\n     1", false);
      return;
    }

    cmdAppend(cmd, "Undefined function or variable '" + cmd.split(/\s+/)[0] + "'.", true);
  }

  /* ------------------------------------------------------------------- figure */
  function makeFigure(expr) {
    closeFigure();
    const center = $("#colCenter");
    const panel = document.createElement("section");
    panel.className = "panel";
    panel.id = "panelFigure";
    panel.setAttribute("data-panel", "figure");
    panel.style.flex = "1 1 34%";
    panel.innerHTML = `
      <div class="panel-head">
        <div class="ptabs">
          <span class="ptab active">Figure 1</span>
        </div>
        <div class="panel-tools">
          <span class="ptool" title="Save"><svg width="13" height="13"><use href="#ico-save"/></svg></span>
          <span class="ptool" title="Print"><svg width="13" height="13"><use href="#ico-print"/></svg></span>
          <span class="ptool" title="Close" data-close-fig="1"><svg width="13" height="13"><use href="#ico-x"/></svg></span>
        </div>
      </div>
      <div class="panel-body">
        <div class="fig-wrap">
          <div class="fig-toolbar">
            <span class="fig-tool" title="Pan"><svg width="14" height="14"><use href="#ico-pan"/></svg></span>
            <span class="fig-tool" title="Zoom"><svg width="14" height="14"><use href="#ico-zoom"/></svg></span>
            <span class="fig-tool active" title="Data cursor"><svg width="14" height="14"><use href="#ico-cursor"/></svg></span>
            <span class="fig-tool" title="Home"><svg width="14" height="14"><use href="#ico-layout"/></svg></span>
          </div>
          <div class="fig-canvas">${sinePlotSvg(expr || "figure")}</div>
        </div>
      </div>`;
    center.appendChild(panel);
    wirePanel(panel);
    // three-way split so the new panel does not squeeze the command window
    const ed = $("#panelEditor"), cmdp = $("#panelCommand");
    if (ed) ed.style.flex = "1 1 33%";
    if (cmdp) cmdp.style.flex = "1 1 33%";
    panel.querySelector("[data-close-fig]").onclick = closeFigure;
    setStatus("Figure 1 created (docked)");
    toast("Figure 1 docked into the center column");
  }

  function closeFigure() {
    const f = $("#panelFigure");
    if (f) f.remove();
  }

  function sinePlotSvg(expr) {
    const W = 620, H = 300, padL = 46, padR = 12, padT = 22, padB = 30;
    const pts = [];
    const ymax = 1.15, ymin = -1.15;
    for (let i = 0; i <= 200; i++) {
      const t = i / 200;
      const y = Math.sin(2 * Math.PI * 4 * t) * 0.9;
      const x = padL + t * (W - padL - padR);
      const yy = padT + (ymax - y) / (ymax - ymin) * (H - padT - padB);
      pts.push((i ? "L" : "M") + x.toFixed(1) + " " + yy.toFixed(1));
    }
    let ticks = "";
    for (let k = 0; k <= 4; k++) {
      const y = ymax - k * (ymax - ymin) / 4;
      const yy = padT + k * (H - padT - padB) / 4;
      ticks += `<line x1="${padL}" y1="${yy}" x2="${W - padR}" y2="${yy}" stroke="currentColor" stroke-opacity=".14"/>`;
      ticks += `<text x="${padL - 6}" y="${yy + 3}" text-anchor="end" font-size="9" fill="currentColor" fill-opacity=".75">${y.toFixed(1)}</text>`;
    }
    for (let k = 0; k <= 5; k++) {
      const tac = k / 5;
      const xx = padL + tac * (W - padL - padR);
      ticks += `<text x="${xx}" y="${H - padB + 14}" text-anchor="middle" font-size="9" fill="currentColor" fill-opacity=".75">${Math.round(tac * 100)}</text>`;
    }
    return `<svg viewBox="0 0 ${W} ${H}" preserveAspectRatio="none">
      <rect x="0" y="0" width="${W}" height="${H}" fill="var(--bg-panel)"/>
      ${ticks}
      <rect x="${padL}" y="${padT}" width="${W - padL - padR}" height="${H - padT - padB}"
            fill="none" stroke="currentColor" stroke-opacity=".55"/>
      <path d="${pts.join(" ")}" fill="none" stroke="var(--accent-blue)" stroke-width="1.4"/>
      <text x="${(W + padL) / 2}" y="13" text-anchor="middle" font-size="11" fill="currentColor">${escapeHtml(expr)}</text>
      <text x="${(W + padL) / 2}" y="${H - 4}" text-anchor="middle" font-size="10" fill="currentColor" fill-opacity=".8">index (mock series)</text>
    </svg>`;
  }

  /* -------------------------------------------------------------- docking */
  let dragPanel = null;
  function wirePanel(panel) {
    const head = panel.querySelector(".panel-head");
    if (!head) return;
    head.setAttribute("draggable", "true");
    head.addEventListener("dragstart", (e) => {
      dragPanel = panel;
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", panel.id);
      panel.style.opacity = ".55";
    });
    head.addEventListener("dragend", () => {
      panel.style.opacity = "";
      dragPanel = null;
      $$(".dock-col").forEach((c) => c.classList.remove("drop-target"));
    });
    head.addEventListener("dblclick", (e) => {
      if (e.target.closest(".ptool")) return;
      panel.classList.toggle("collapsed");
      if (panel.classList.contains("collapsed")) panel.style.flex = "0 0 auto";
      else panel.style.flex = panel.id === "panelFolder" ? "1 1 55%" :
        panel.id === "panelWorkspace" ? "1 1 45%" :
        panel.id === "panelEditor" ? "1 1 60%" : "1 1 40%";
    });
  }

  function initDocking() {
    $$(".dock-col").forEach((col) => {
      col.addEventListener("dragover", (e) => {
        if (!dragPanel) return;
        e.preventDefault();
        col.classList.add("drop-target");
      });
      col.addEventListener("dragleave", () => col.classList.remove("drop-target"));
      col.addEventListener("drop", (e) => {
        e.preventDefault();
        col.classList.remove("drop-target");
        if (!dragPanel) return;
        if (dragPanel.parentElement === col) return;
        // keep center column's splitter meaningful: panels just append
        col.appendChild(dragPanel);
        dragPanel.style.flex = "1 1 50%";
        setStatus("Panel moved to " + (col.id.replace("col", "").toLowerCase()) + " column");
        dragPanel = null;
      });
    });
  }

  /* ------------------------------------------------------------ layout presets */
  function applyLayout(kind) {
    const left = $("#colLeft");
    const right = $("#colRight");
    const center = $("#colCenter");
    const folder = $("#panelFolder");
    const ws = $("#panelWorkspace");

    if (kind === "default") {
      // measured factory default: Current Folder left, Workspace right
      left.appendChild(folder);
      folder.style.flex = "1 1 auto";
      right.appendChild(ws);
      ws.style.flex = "1 1 auto";
      right.style.display = "";
      state.layout = "default";
      setStatus("Layout: default (Workspace right)");
    } else {
      // as captured on the reference machine: Workspace stacked under Current Folder
      left.appendChild(folder);
      folder.style.flex = "1 1 55%";
      left.appendChild(ws);
      ws.style.flex = "1 1 45%";
      state.layout = "stacked";
      setStatus("Layout: Workspace bottom-left (as captured)");
    }
    // editor + command stay in the center column
    [ $("#panelEditor"), $("#splitEditorCmd"), $("#panelCommand") ].forEach((n) => {
      if (n && n.parentElement !== center) center.insertBefore(n, center.firstChild.nextSibling ? null : center.firstChild);
    });
    toast("Layout: " + (kind === "default" ? "Default" : "Workspace bottom-left"));
  }

  /* ---------------------------------------------------------------- splitter */
  function initSplitter() {
    const sp = $("#splitEditorCmd");
    const editor = $("#panelEditor");
    const cmd = $("#panelCommand");
    let dragging = false, startY = 0, startH = 0;
    sp.addEventListener("mousedown", (e) => {
      dragging = true; startY = e.clientY;
      startH = editor.getBoundingClientRect().height;
      document.body.style.cursor = "row-resize";
      e.preventDefault();
    });
    document.addEventListener("mousemove", (e) => {
      if (!dragging) return;
      const total = editor.getBoundingClientRect().height + cmd.getBoundingClientRect().height;
      let h = Math.max(80, Math.min(total - 80, startH + (e.clientY - startY)));
      editor.style.flex = "0 0 " + h + "px";
      cmd.style.flex = "1 1 auto";
    });
    document.addEventListener("mouseup", () => {
      if (!dragging) return;
      dragging = false;
      document.body.style.cursor = "";
    });
  }

  /* -------------------------------------------------------------------- theme */
  function setTheme(t) {
    state.theme = t;
    document.documentElement.setAttribute("data-theme", t);
    $("#stTheme").textContent = t === "dark" ? "Dark" : "Light";
    const b = $("#themeBtn");
    b.title = "Switch to " + (t === "dark" ? "light" : "dark") + " theme";
    toast(t === "dark" ? "Dark theme (measured tokens)" : "Light theme (measured tokens)");
  }

  /* -------------------------------------------------------------- preferences */
  const PREF_PAGES = {
    "appearance-colors": () => `
      <h3>Colors</h3>
      <p class="hint">Theme presets measured from the reference desktop. The prototype applies them live.</p>
      <div class="pref-row"><span class="label">Desktop theme</span>
        <div class="radio-row">
          <label><input type="radio" name="theme" value="light" ${state.theme === "light" ? "checked" : ""}> Light</label>
          <label><input type="radio" name="theme" value="dark" ${state.theme === "dark" ? "checked" : ""}> Dark</label>
        </div>
      </div>
      <div class="pref-row"><span class="label">Syntax highlighting</span>
        <select><option>Editor default (measured)</option><option>Monochrome</option></select>
      </div>`,
    "appearance-fonts": () => `
      <h3>Fonts</h3>
      <div class="pref-row"><span class="label">Desktop text</span>
        <select><option>Segoe UI 12</option><option>Segoe UI 11</option></select>
      </div>
      <div class="pref-row"><span class="label">Editor / Command</span>
        <select><option>Consolas 11</option><option>Consolas 12</option></select>
      </div>
      <div class="pref-row"><span class="label">Line spacing</span>
        <input type="number" value="18"> px
      </div>`,
    editor: () => `
      <h3>Editor</h3>
      <div class="pref-row"><span class="label">Show line numbers</span><input type="checkbox" checked></div>
      <div class="pref-row"><span class="label">Highlight current line</span><input type="checkbox" checked></div>
      <div class="pref-row"><span class="label">Tab size</span><input type="number" value="4"></div>`,
    command: () => `
      <h3>Command Window</h3>
      <div class="pref-row"><span class="label">Banner on start</span><input type="checkbox" checked></div>
      <div class="pref-row"><span class="label">History depth</span><input type="number" value="500"></div>`,
    workspace: () => `
      <h3>Workspace</h3>
      <div class="pref-row"><span class="label">Show class column</span><input type="checkbox" checked></div>
      <div class="pref-row"><span class="label">Sort on create</span><input type="checkbox"></div>`,
    figure: () => `
      <h3>Figure Copy</h3>
      <div class="pref-row"><span class="label">Copy format</span>
        <select><option>Metafile</option><option>Bitmap</option><option>SVG</option></select>
      </div>
      <div class="pref-row"><span class="label">Dock new figures</span><input type="checkbox" checked></div>`,
    layout: () => `
      <h3>Layout</h3>
      <div class="pref-row"><span class="label">Preset</span>
        <select id="prefLayout">
          <option value="default" ${state.layout === "default" || !state.layout ? "selected" : ""}>Default (Workspace right)</option>
          <option value="stacked" ${state.layout === "stacked" ? "selected" : ""}>Workspace bottom-left</option>
        </select>
      </div>
      <p class="hint">The default preset follows the factory arrangement; the stacked one reproduces the reference machine.</p>`,
    general: () => `
      <h3>General</h3>
      <div class="pref-row"><span class="label">Product</span><span>TensorLab (UI paradigm study)</span></div>
      <div class="pref-row"><span class="label">Kernel</span><span>RunMat 0.6.2</span></div>
      <div class="pref-row"><span class="label">Study source</span><span>clean-room captures of a licensed desktop</span></div>`
  };

  const PREF_TITLES = {
    "appearance-colors": "Colors",
    "appearance-fonts": "Fonts",
    editor: "Editor",
    command: "Command Window",
    workspace: "Workspace",
    figure: "Figure Copy",
    layout: "Layout",
    general: "General"
  };

  function openPrefs() {
    $("#prefOverlay").classList.add("open");
    showPrefPage("appearance-colors");
  }

  function showPrefPage(node) {
    $$(".pref-item").forEach((el) => el.classList.toggle("active", el.getAttribute("data-node") === node));
    const render = PREF_PAGES[node];
    const box = $("#prefContent");
    if (render) {
      box.innerHTML = render();
      // wire live controls
      $$('input[name="theme"]', box).forEach((r) => {
        r.onchange = () => setTheme(r.value);
      });
      const sel = $("#prefLayout", box);
      if (sel) sel.onchange = () => applyLayout(sel.value);
    } else {
      box.innerHTML = `<h3>${PREF_TITLES[node] || "TensorLab"}</h3>
        <p class="hint">Select an element in the tree to configure it.</p>`;
    }
  }

  function initPrefs() {
    $("#prefClose").onclick = () => $("#prefOverlay").classList.remove("open");
    $("#prefOverlay").addEventListener("click", (e) => {
      if (e.target.id === "prefOverlay") $("#prefOverlay").classList.remove("open");
    });
    $$(".pref-item").forEach((el) => {
      el.onclick = () => {
        const node = el.getAttribute("data-node");
        if (node === "root" || node === "appearance") return;
        showPrefPage(node);
      };
    });
    $("#prefSearch").oninput = (e) => {
      const q = e.target.value.trim().toLowerCase();
      $$(".pref-item").forEach((el) => {
        if (!el.getAttribute("data-leaf")) return;
        const t = el.textContent.toLowerCase();
        el.style.display = !q || t.indexOf(q) >= 0 ? "" : "none";
      });
    };
  }

  /* ------------------------------------------------------------------ actions */
  function doAction(act) {
    if (act === "clc") { $("#cmdLog").innerHTML = ""; setStatus("Command Window cleared"); return; }
    if (act === "clearWs") { state.vars = []; renderWorkspace(); setStatus("Workspace cleared"); return; }
    if (act === "save") { toast("Saved " + state.activeDoc + " (prototype)"); setStatus("Saved"); return; }
    if (act === "run") {
      cmdAppend("run('" + state.activeDoc + "')", "");
      setStatus("Running " + state.activeDoc);
      setTimeout(() => { makeFigure(); setStatus("Ready"); }, 120);
      return;
    }
    if (act === "themeLight") { setTheme("light"); return; }
    if (act === "themeDark") { setTheme("dark"); return; }
    if (act === "prefs") { openPrefs(); return; }
    if (act === "about") {
      toast("TensorLab 0.1.0 · RunMat 0.6.2 · UI paradigm study (clean-room)");
      return;
    }
  }

  /* --------------------------------------------------------------------- init */
  function init() {
    renderRibbonTabs();
    renderRibbonBody();
    renderFolder();
    renderWorkspace();
    renderEditor();
    initDocking();
    initSplitter();
    initPrefs();
    $$(".panel").forEach(wirePanel);

    $("#themeBtn").onclick = () => setTheme(state.theme === "light" ? "dark" : "light");
    $("#dismissBanner").onclick = () => $("#banner").remove();
    $("#quickStart").onclick = (e) => { e.preventDefault(); toast("Quick Start — placeholder"); };

    const input = $("#cmdInput");
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        const v = input.value;
        input.value = "";
        execCommand(v);
      } else if (e.key === "ArrowUp") {
        if (!state.cmdHistory.length) return;
        state.histIdx = Math.max(0, state.histIdx - 1);
        input.value = state.cmdHistory[state.histIdx] || "";
        e.preventDefault();
      } else if (e.key === "ArrowDown") {
        if (!state.cmdHistory.length) return;
        state.histIdx = Math.min(state.cmdHistory.length, state.histIdx + 1);
        input.value = state.cmdHistory[state.histIdx] || "";
        e.preventDefault();
      }
    });

    // editor line tracking
    $("#code").addEventListener("click", (e) => {
      const rect = e.currentTarget.getBoundingClientRect();
      const line = Math.max(1, Math.min(999, Math.floor((e.clientY - rect.top) / 18) + 1));
      const col = Math.max(1, Math.round((e.clientX - rect.left) / 6.6) + 1);
      $$(".ln", e.currentTarget).forEach((n, i) => n.classList.toggle("cur", i === line - 1));
      $("#stPos").textContent = "Ln " + line + ", Col " + col;
    });

    // global search
    document.addEventListener("keydown", (e) => {
      if (e.ctrlKey && e.shiftKey && e.code === "Space") {
        e.preventDefault();
        $("#globalSearch").focus();
      }
      if (e.key === "Escape") {
        const ov = $("#prefOverlay");
        if (ov.classList.contains("open")) ov.classList.remove("open");
      }
    });
    $("#globalSearch").addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        toast('Search: "' + e.target.value + '" — placeholder');
        setStatus("Search: " + e.target.value);
      }
    });

    // start in the measured default layout
    applyLayout("default");
    cmdAppend("", "TensorLab UI paradigm study (prototype)\nKernel: RunMat 0.6.2 — mock data only.\nTry: plot, who, clc, help, a = 5, prefs");
    setStatus("Ready");
  }

  document.addEventListener("DOMContentLoaded", init);
})();
