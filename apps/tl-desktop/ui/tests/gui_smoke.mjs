#!/usr/bin/env node
// TensorLab desktop GUI smoke test (zero npm dependencies).
//
// Loads the real apps/tl-desktop/ui/index.html in headless Edge/Chrome, injects
// a Tauri IPC shim, and drives the UI through the Chrome DevTools Protocol:
//   - initial render (editor + syntax highlight + console banner + version)
//   - Run button -> console output, workspace table, FigureScene -> SVG
//   - About dialog (version / kernel / license, F1 and Escape)
//   - console REPL command + history
//   - Command Window context menu (Copy / Clear) and Ctrl+L clear
//   - error rendering and clearConsole handling
//
// Usage: node apps/tl-desktop/ui/tests/gui_smoke.mjs [--browser <path>]
// Exit code 0 = all assertions passed.

import http from "node:http";
import { spawn } from "node:child_process";
import { readFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const indexHtml = readFileSync(join(here, "..", "index.html"), "utf8");

const argBrowser = process.argv.indexOf("--browser");
const candidates = [
  argBrowser > 0 ? process.argv[argBrowser + 1] : null,
  "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
  "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
  "C:/Program Files/Google/Chrome/Application/chrome.exe",
].filter(Boolean);

function findBrowser() {
  for (const p of candidates) {
    try {
      readFileSync(p);
      return p;
    } catch {
      /* keep looking */
    }
  }
  throw new Error("no Edge/Chrome binary found; pass --browser <path>");
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---------------------------------------------------------------- static server
const server = http.createServer((req, res) => {
  if (req.url === "/" || req.url.startsWith("/index.html")) {
    res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    res.end(indexHtml);
  } else {
    res.writeHead(404);
    res.end("not found");
  }
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const pageUrl = `http://127.0.0.1:${server.address().port}/index.html`;

// ---------------------------------------------------------------- browser + CDP
const profile = mkdtempSync(join(tmpdir(), "tl-gui-"));
const port = 9333 + Math.floor(Math.random() * 300);
const browser = spawn(findBrowser(), [
  "--headless=new",
  "--disable-gpu",
  "--no-first-run",
  "--no-default-browser-check",
  `--remote-debugging-port=${port}`,
  `--user-data-dir=${profile}`,
  "about:blank",
], { stdio: "ignore" });

async function browserWs() {
  for (let i = 0; i < 100; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${port}/json/version`);
      if (r.ok) return (await r.json()).webSocketDebuggerUrl;
    } catch {
      /* not ready */
    }
    await sleep(100);
  }
  throw new Error("DevTools endpoint did not come up");
}

const bws = new WebSocket(await browserWs());
await new Promise((res, rej) => {
  bws.onopen = res;
  bws.onerror = rej;
});
let bmsgId = 0;
const bpending = new Map();
bws.onmessage = (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.id && bpending.has(msg.id)) {
    const { resolve, reject } = bpending.get(msg.id);
    bpending.delete(msg.id);
    msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
  }
};
function bsend(method, params = {}) {
  const id = ++bmsgId;
  bws.send(JSON.stringify({ id, method, params }));
  return new Promise((resolve, reject) => bpending.set(id, { resolve, reject }));
}

const { targetId } = await bsend("Target.createTarget", { url: "about:blank" });
const { sessionId } = await bsend("Target.attachToTarget", { targetId, flatten: true });
const send = (method, params = {}) =>
  bsend(method, params).catch(async (e) => {
    throw e;
  });

// Flattened sessions: attach commands need sessionId; wrap it.
function ssend(method, params = {}) {
  const id = ++bmsgId;
  bws.send(JSON.stringify({ id, method, params, sessionId }));
  return new Promise((resolve, reject) => bpending.set(id, { resolve, reject }));
}

await ssend("Page.enable");
await ssend("Runtime.enable");

// Tauri shim: record every invoke and return canned RunReports.
const shim = `
window.__TL_CALLS__ = [];
window.__TAURI_INTERNALS__ = {
  invoke: async (cmd, args) => {
    window.__TL_CALLS__.push({ cmd, args });
    if (cmd === "app_info") {
      return { version: "0.2.1", kernel: "RunMat 0.6.2", compat: "MATLAB R2026a", license: "MIT" };
    }
    if (cmd !== "run") return null;
    const src = (args && args.source) || "";
    const vars = [
      { name: "x", className: "double", shape: [1, 5], bytes: 40, preview: "[1 2 3 4 5]", isGpu: false },
      { name: "A", className: "double", shape: [2, 2], bytes: 32, preview: "[1 2; 3 4]", isGpu: false },
    ];
    if (src.indexOf("boom") >= 0) {
      return { events: [], resultText: null, error: { identifier: "test:boom", message: "boom!", line: 1, callstack: [], severity: "error" }, warnings: [], variables: vars, figures: [], durationMs: 1.5, usedJit: false };
    }
    if (src === "clear") {
      return { events: [{ kind: "clearConsole", text: "" }], resultText: null, error: null, warnings: [], variables: [], figures: [], durationMs: 0.5, usedJit: false };
    }
    if (src === "vars") {
      return { events: [{ kind: "output", text: "x =  1  2  3\\n\\nA =\\n\\n     1     2\\n     3     4\\n\\n" }], resultText: null, error: null, warnings: [], variables: vars, figures: [], durationMs: 0.7, usedJit: false };
    }
    if (src.indexOf("figure") >= 0) {
      const scene = {
        createdAt: "2026-10-17T00:00:00Z",
        figure: {
          layout: { axesRows: 1, axesCols: 2, axesIndices: [0, 1] },
          metadata: {
            title: "figure",
            gridEnabled: true,
            axesMetadata: [
              { title: "sin(x)", xLabel: "t", yLabel: "y", gridEnabled: true },
              { title: "cos(x)", xLabel: "t", yLabel: "y", gridEnabled: true },
            ],
          },
          plots: [
            { kind: "line", x: [0, 1, 2, 3], y: [0, 1, 0, -1], color_rgba: [0, 0.45, 0.74, 1], line_width: 1.5, line_style: "Solid", axes_index: 0, visible: true },
            { kind: "line", x: [0, 1, 2, 3], y: [1, 0, -1, 0], color_rgba: [0.85, 0.33, 0.1, 1], line_width: 1.5, line_style: "Solid", axes_index: 1, visible: true },
          ],
        },
      };
      return { events: [{ kind: "output", text: "plotting\\n" }], resultText: null, error: null, warnings: [], variables: vars, figures: [{ handle: 1, sceneJson: scene }], durationMs: 2.0, usedJit: true };
    }
    return { events: [{ kind: "output", text: "mock output\\n" }], resultText: "ans = 42", error: null, warnings: [], variables: vars, figures: [], durationMs: 1.0, usedJit: false };
  },
};
`;

await ssend("Page.addScriptToEvaluateOnNewDocument", { source: shim });
await ssend("Page.navigate", { url: pageUrl });
for (let i = 0; i < 100; i++) {
  const { result } = await ssend("Runtime.evaluate", { expression: "document.readyState", returnByValue: true });
  if (result.value === "complete") break;
  await sleep(50);
}
await sleep(100);

async function evalJs(expression) {
  const { result, exceptionDetails } = await ssend("Runtime.evaluate", {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  if (exceptionDetails) throw new Error(exceptionDetails.text || "JS exception");
  return result.value;
}

async function waitFor(expression, label, timeoutMs = 4000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (await evalJs(expression)) return;
    await sleep(50);
  }
  throw new Error(`timeout waiting for: ${label}`);
}

let passed = 0;
const failures = [];
async function check(name, fn) {
  try {
    await fn();
    passed++;
    console.log(`ok   ${name}`);
  } catch (e) {
    failures.push(`${name}: ${e.message}`);
    console.log(`FAIL ${name}: ${e.message}`);
  }
}

// ---------------------------------------------------------------- assertions
await check("editor is populated and syntax highlighted", async () => {
  const len = await evalJs("document.getElementById('src').value.length");
  if (!(len > 50)) throw new Error(`editor too short: ${len}`);
  const highlighted = await evalJs("document.querySelectorAll('#highlight .tok-keyword, #highlight .tok-function').length");
  if (!(highlighted > 3)) throw new Error(`no syntax tokens: ${highlighted}`);
});

await check("console shows the startup banner", async () => {
  const text = await evalJs("document.getElementById('console').textContent");
  if (!text.includes("TensorLab desktop")) throw new Error(text);
});

await check("About dialog shows version, kernel and license", async () => {
  await waitFor("document.getElementById('app-version').textContent === 'v0.2.1'", "header version");
  await evalJs("document.getElementById('about-btn').click()");
  await waitFor("document.getElementById('about-overlay').hidden === false", "about open");
  const version = await evalJs("document.getElementById('about-version').textContent");
  if (version !== "0.2.1") throw new Error(`version: ${version}`);
  const text = await evalJs("document.getElementById('about-dialog').textContent");
  for (const want of ["TensorLab", "Version", "RunMat 0.6.2", "MATLAB R2026a", "MIT"]) {
    if (!text.includes(want)) throw new Error(`missing "${want}": ${text}`);
  }
  // 版本值必须与下方 RunMat 版本值左对齐（同一 grid 值列）
  const aligned = await evalJs(`(() => {
    const a = document.getElementById('about-version').getBoundingClientRect().left;
    const b = document.getElementById('about-kernel').getBoundingClientRect().left;
    return Math.abs(a - b) < 0.5;
  })()`);
  if (!aligned) throw new Error("version value is not left-aligned with kernel value");
  await evalJs("document.getElementById('about-close').click()");
  await waitFor("document.getElementById('about-overlay').hidden === true", "about closed");
  await evalJs("document.dispatchEvent(new KeyboardEvent('keydown', { key: 'F1', bubbles: true }))");
  await waitFor("document.getElementById('about-overlay').hidden === false", "F1 opens about");
  await evalJs("document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))");
  await waitFor("document.getElementById('about-overlay').hidden === true", "Escape closes about");
});

await check("Run renders console output, workspace table and SVG figure", async () => {
  await evalJs("document.getElementById('run-btn').click()");
  await waitFor("document.querySelectorAll('#vars table tr').length >= 3", "vars rows");
  await waitFor("document.querySelector('#figures svg polyline') !== null", "figure polyline");
  const out = await evalJs("document.getElementById('console').textContent");
  if (!out.includes("plotting")) throw new Error(`missing output: ${out}`);
  const rows = await evalJs("document.querySelectorAll('#vars table tr').length");
  if (rows !== 3) throw new Error(`expected header + 2 vars, got ${rows}`);
  const ticks = await evalJs("document.querySelectorAll('#figures svg text').length");
  if (ticks < 5) throw new Error(`figure has too few labels: ${ticks}`);
  const status = await evalJs("document.getElementById('status').textContent");
  if (!status.includes("JIT")) throw new Error(`status: ${status}`);
});

// 结构级断言：不只判断 <polyline> 存在，还要校验轴数量、折线点数和刻度标签。
await check("figure SVG exposes two axes, two polylines and tick labels", async () => {
  const viewBox = await evalJs("document.querySelector('#figures svg').getAttribute('viewBox')");
  if (viewBox !== "0 0 830 270") throw new Error(`viewBox: ${viewBox}`);

  const axes = await evalJs("document.querySelectorAll('#figures svg > g').length");
  if (axes !== 2) throw new Error(`expected 2 axes groups, got ${axes}`);

  const lines = await evalJs("document.querySelectorAll('#figures svg polyline').length");
  if (lines !== 2) throw new Error(`expected 2 polylines, got ${lines}`);

  const points = await evalJs("document.querySelector('#figures svg polyline').getAttribute('points').trim().split(/\\s+/).length");
  if (points !== 4) throw new Error(`expected 4 points in first polyline, got ${points}`);

  const labels = await evalJs("Array.from(document.querySelectorAll('#figures svg text')).map(t => t.textContent)");
  for (const label of ["sin(x)", "cos(x)", "t", "y", "0.6", "1.2", "1.8", "2.4"]) {
    if (!labels.includes(label)) throw new Error(`missing figure label: ${label} (got ${labels.join(", ")})`);
  }
  if (labels.length < 20) throw new Error(`too few SVG text nodes: ${labels.length}`);
});

await check("console REPL submits a host command and refreshes variables", async () => {
  const before = await evalJs("window.__TL_CALLS__.length");
  await evalJs(`(() => {
    const el = document.getElementById('console-input');
    el.value = 'vars';
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    return true;
  })()`);
  await waitFor(`window.__TL_CALLS__.length > ${before}`, "invoke call");
  const last = await evalJs("JSON.stringify(window.__TL_CALLS__[window.__TL_CALLS__.length - 1])");
  if (!last.includes('"source":"vars"')) throw new Error(last);
  const out = await evalJs("document.getElementById('console').textContent");
  if (!out.includes("A =")) throw new Error("vars output not rendered");
});

await check("console history recalls the previous command", async () => {
  await evalJs(`(() => {
    const el = document.getElementById('console-input');
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }));
    return true;
  })()`);
  const v = await evalJs("document.getElementById('console-input').value");
  if (v !== "vars") throw new Error(`history value: ${v}`);
});

await check("runtime errors are rendered with the error style", async () => {
  await evalJs(`(() => {
    const el = document.getElementById('console-input');
    el.value = 'boom';
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    return true;
  })()`);
  await waitFor("document.querySelector('#console .err') !== null", "error line");
  const text = await evalJs("document.querySelector('#console .err').textContent");
  if (!text.includes("boom!")) throw new Error(text);
});

// Command Window 右键菜单：复制选中行（对齐 MATLAB Command Window）
await check("console context menu copies the selected text", async () => {
  const prepared = await evalJs(`(() => {
    const host = document.getElementById('console');
    const first = host.querySelector('div');
    if (!first) return false;
    window.__COPY_TARGET__ = first.textContent;
    window.__COPIED__ = null;
    const range = document.createRange();
    range.selectNodeContents(first);
    const sel = window.getSelection();
    sel.removeAllRanges();
    sel.addRange(range);
    try {
      Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: async (t) => { window.__COPIED__ = t; } } });
    } catch (e) { /* 老浏览器走 execCommand 兜底 */ }
    document.execCommand = (cmd) => {
      if (cmd !== 'copy') return false;
      const a = document.activeElement;
      window.__COPIED__ = a && 'value' in a ? a.value : window.getSelection().toString();
      return true;
    };
    host.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, clientX: 140, clientY: 160 }));
    return true;
  })()`);
  if (!prepared) throw new Error("console has no text to select");

  const state = await evalJs(`(() => {
    const menu = document.getElementById('console-menu');
    return {
      hidden: menu.hidden,
      copyDisabled: menu.querySelector('[data-action=copy]').disabled,
      items: menu.querySelectorAll('button[data-action]').length,
    };
  })()`);
  if (state.hidden) throw new Error("context menu did not open");
  if (state.items !== 4) throw new Error(`expected 4 menu items, got ${state.items}`);
  if (state.copyDisabled) throw new Error("Copy is disabled despite a selection");

  await evalJs(`document.querySelector('#console-menu [data-action=copy]').click()`);
  await waitFor("window.__COPIED__ !== null", "clipboard write");
  const copied = await evalJs("window.__COPIED__");
  const expected = await evalJs("window.__COPY_TARGET__");
  if (copied !== expected) throw new Error(`copied ${JSON.stringify(copied)} != ${JSON.stringify(expected)}`);
  const hidden = await evalJs("document.getElementById('console-menu').hidden");
  if (!hidden) throw new Error("menu did not close after Copy");
});

await check("Ctrl+L clears the Command Window", async () => {
  const before = await evalJs("document.getElementById('console').children.length");
  if (before === 0) throw new Error("console is already empty");
  await evalJs("document.dispatchEvent(new KeyboardEvent('keydown', { key: 'l', ctrlKey: true, bubbles: true }))");
  await waitFor("document.getElementById('console').children.length === 0", "Ctrl+L clear");
});

await check("clearConsole event empties the console", async () => {
  await evalJs(`(() => {
    const el = document.getElementById('console-input');
    el.value = 'vars';
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    return true;
  })()`);
  await waitFor("document.getElementById('console').children.length > 0", "console seeded");
  await evalJs(`(() => {
    const el = document.getElementById('console-input');
    el.value = 'clear';
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    return true;
  })()`);
  await waitFor("document.getElementById('console').children.length === 0", "console cleared");
});

// ---------------------------------------------------------------- teardown
browser.kill();
server.close();
try {
  rmSync(profile, { recursive: true, force: true });
} catch {
  /* Windows may still hold the profile briefly */
}

console.log(`\n${passed} passed, ${failures.length} failed`);
if (failures.length) {
  for (const f of failures) console.log(`  - ${f}`);
  process.exit(1);
}
process.exit(0);
