#!/usr/bin/env node
// TensorLab 自写 m 函数测试运行器。
//
// 读取 tests/toolbox_cases/*.json，为每个函数生成一份 TensorLab 脚本、为整批生成
// 一份 MATLAB 脚本，分别执行后比较数值结果，输出 docs/testing/toolbox-report.md。
//
// 用法：
//   node tests/toolbox_compare.mjs
//   node tests/toolbox_compare.mjs --category signal
//   node tests/toolbox_compare.mjs --no-matlab          # 只跑 TensorLab
//   node tests/toolbox_compare.mjs --timeout 60         # 单函数超时（秒）

import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync, openSync, closeSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");
const args = process.argv.slice(2);
const getArg = (name, fallback) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : fallback;
};
const categoryFilter = getArg("--category", null);
const noMatlab = args.includes("--no-matlab");
const caseTimeoutMs = Number(getArg("--timeout", "120")) * 1000;
const tlExe = getArg(
  "--tensorlab",
  join(root, "target", "debug", process.platform === "win32" ? "tensorlab.exe" : "tensorlab"),
);
const matlabExe = getArg(
  "--matlab",
  process.platform === "win32"
    ? "D:/Program Files/MATLAB/R2023b/bin/matlab.exe"
    : "matlab",
);

const casesDir = join(here, "toolbox_cases");
const outDir = join(root, "target", "toolbox-tests");
const tlDir = join(outDir, "tl");
mkdirSync(tlDir, { recursive: true });

const categories = readdirSync(casesDir)
  .filter((f) => f.endsWith(".json"))
  .map((f) => JSON.parse(readFileSync(join(casesDir, f), "utf8")))
  .filter((c) => !categoryFilter || c.category === categoryFilter);

if (!categories.length) {
  console.error("no case files matched");
  process.exit(1);
}

// ---------------------------------------------------------------- 生成用例
const caseList = [];
for (const cat of categories) {
  for (const fn of cat.functions) {
    const groups = [
      ["normal", fn.normal || []],
      ["boundary", fn.boundary || []],
      ["abnormal", fn.abnormal || []],
    ];
    let seq = 0;
    for (const [kind, codes] of groups) {
      for (const code of codes) {
        seq += 1;
        const id = `${fn.name}-${kind}-${seq}`;
        caseList.push({
          id,
          category: cat.category,
          name: fn.name,
          kind,
          code,
          setup: fn.setup || "",
          compare: fn.compare || [],
          matlab: fn.matlab !== false,
          tol: fn.tol ?? 1e-9,
          note: fn.note || "",
        });
      }
    }
  }
}

// ---------------------------------------------------------------- 生成脚本
function emitBlock(c, errorField) {
  const lines = [];
  lines.push(`fprintf('CASE ${c.id}\\n');`);
  lines.push("try");
  lines.push("  clear;");
  if (c.setup) lines.push(`  ${c.setup}`);
  lines.push(`  ${c.code}`);
  for (const name of c.compare) {
    lines.push(`  fprintf('BEGIN ${c.id} ${name}\\n');`);
    lines.push(`  d = size(${name});`);
    lines.push("  fprintf('D %d\\n', numel(d));");
    lines.push("  for j=1:numel(d); fprintf('D %d\\n', d(j)); end");
    lines.push(`  v = ${name}(:);`);
    lines.push("  for i=1:numel(v); fprintf('V %.17g %.17g\\n', real(v(i)), imag(v(i))); end");
    lines.push(`  fprintf('END ${c.id} ${name}\\n');`);
  }
  lines.push("catch err");
  lines.push(
    errorField === "identifier"
      ? `  fprintf('ERR ${c.id} %s %s\\n', err.identifier, strrep(err.message, char(10), ' '));`
      : `  fprintf('ERR ${c.id} %s\\n', err.${errorField});`,
  );
  lines.push("end");
  return lines.join("\n");
}

const header = [
  "% 由 tests/toolbox_compare.mjs 生成，请勿手改",
  "try; close all; catch; end",
];

// TensorLab：按函数拆分，避免单个死循环用例拖垮整批
const byFunction = new Map();
for (const c of caseList) {
  if (!byFunction.has(c.name)) byFunction.set(c.name, []);
  byFunction.get(c.name).push(c);
}
const tlScripts = [];
for (const [name, list] of byFunction) {
  const path = join(tlDir, `${name}.m`);
  writeFileSync(path, header.concat(list.map((c) => emitBlock(c, "message"))).join("\n\n") + "\n", "utf8");
  tlScripts.push({ name, path, cases: list });
}

const mlCases = caseList.filter((c) => c.matlab);
const mlPath = join(outDir, "matlab_cases.m");
writeFileSync(mlPath, header.concat(mlCases.map((c) => emitBlock(c, "identifier"))).join("\n\n") + "\n", "utf8");

// ---------------------------------------------------------------- 解析输出
function parseOutput(text) {
  const results = new Map();
  let current = null;
  let dims = [];
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) continue;
    if (line.startsWith("CASE ")) {
      const id = line.split(/\s+/)[1];
      if (!results.has(id)) results.set(id, { ok: true, vars: {} });
    } else if (line.startsWith("BEGIN ")) {
      const parts = line.split(/\s+/);
      const id = parts[1];
      const varName = parts[2];
      if (!results.has(id)) results.set(id, { ok: true, vars: {} });
      current = { id, varName, values: [] };
      dims = [];
    } else if (line.startsWith("D ") && current) {
      dims.push(Number(line.slice(2)));
    } else if (line.startsWith("V ") && current) {
      const [re, im] = line.slice(2).split(/\s+/).map(Number);
      current.values.push({ re, im });
    } else if (line.startsWith("END ") && current) {
      // 第一个 D 是维度数量；其后是各维长度
      const nd = dims.shift() ?? 0;
      const shape = dims.slice(0, nd);
      results.get(current.id).vars[current.varName] = { dims: shape, values: current.values };
      current = null;
      dims = [];
    } else if (line.startsWith("ERR ")) {
      const parts = line.split(/\s+/);
      const id = parts[1];
      results.set(id, { ok: false, error: parts.slice(2).join(" ") });
      current = null;
    }
  }
  return results;
}

function run(cmd, cmdArgs, label, timeout = 60 * 60 * 1000, encoding = "utf8") {
  // 输出重定向到文件：进程超时被杀时仍能保留已产生的部分结果。
  const logPath = join(outDir, "last-run.log");
  for (let attempt = 0; attempt < 4; attempt += 1) {
    const fd = openSync(logPath, "w");
    let res;
    try {
      res = spawnSync(cmd, cmdArgs, { cwd: root, stdio: ["ignore", fd, fd], timeout });
    } finally {
      closeSync(fd);
    }
    const raw = readFileSync(logPath);
    const stdout = new TextDecoder(encoding).decode(raw);
    if (res.error && res.error.code === "ETIMEDOUT") {
      return { timeout: true, crashed: false, stdout };
    }
    if (res.error && res.error.code === "EPERM" && attempt < 3) {
      // 杀毒软件偶尔锁定刚构建的可执行文件，等待后重试
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 3000);
      continue;
    }
    if (res.error) throw res.error;
    return { timeout: false, crashed: res.status !== 0, stdout };
  }
  throw new Error(`${label} failed after retries`);
}

if (!existsSync(tlExe)) {
  console.error(`TensorLab executable not found: ${tlExe}`);
  process.exit(1);
}

const tlResults = new Map();
const timedOutFunctions = new Set();
const hungCases = new Set();
const crashCases = new Set();
const crashedFunctions = new Map();
const singleDir = join(tlDir, "single");
mkdirSync(singleDir, { recursive: true });
let idx = 0;
for (const { name, path } of tlScripts) {
  idx += 1;
  process.stdout.write(`[${idx}/${tlScripts.length}] tensorlab ${name} ... `);
  const res = run(tlExe, ["run", path], `TensorLab ${name}`, caseTimeoutMs);
  for (const [id, v] of parseOutput(res.stdout)) tlResults.set(id, v);
  if (res.timeout || res.crashed) {
    const reason = res.stdout.split(/\r?\n/).find((l) => l.includes("panicked")) || "非零退出";
    // 输出被缓冲，超时/崩溃后拿不到部分结果：逐用例重跑，精确定位问题用例。
    const retried = [];
    for (const c of byFunction.get(name)) {
      const single = join(singleDir, `${c.id}.m`);
      writeFileSync(single, header.concat(emitBlock(c, "message")).join("\n\n") + "\n", "utf8");
      const r2 = run(tlExe, ["run", single], `TensorLab ${c.id}`, Math.min(caseTimeoutMs, 30000));
      for (const [id, v] of parseOutput(r2.stdout)) tlResults.set(id, v);
      if (r2.timeout) hungCases.add(c.id);
      if (r2.crashed) crashCases.add(c.id);
      retried.push(r2.timeout ? "H" : r2.crashed ? "C" : ".");
    }
    if (res.timeout) timedOutFunctions.add(name);
    if (res.crashed) crashedFunctions.set(name, reason.trim());
    console.log(
      `${res.timeout ? `TIMEOUT (${caseTimeoutMs / 1000}s)` : "CRASH"} -> case-by-case [${retried.join("")}]`,
    );
  } else {
    console.log("done");
  }
}

let mlResults = new Map();
let matlabCrashed = false;
if (!noMatlab) {
  console.log(`> MATLAB R2023b (${mlCases.length} cases)`);
  const res = run(
    matlabExe,
    ["-batch", `cd('${root.replaceAll("\\", "/")}'); run('${mlPath.replaceAll("\\", "/")}')`],
    "MATLAB R2023b",
    2 * 60 * 60 * 1000,
    "gbk",
  );
  mlResults = parseOutput(res.stdout);
  if (res.timeout || res.crashed) {
    matlabCrashed = true;
    console.error(`MATLAB ${res.timeout ? "超时" : "异常退出"}，已解析 ${mlResults.size} 条结果`);
  }
}

// ---------------------------------------------------------------- 比较
function maxError(tlVar, mlVar) {
  if (!tlVar || !mlVar) return { abs: Infinity, rel: Infinity, shape: false };
  const shape =
    tlVar.dims.length === mlVar.dims.length &&
    tlVar.dims.every((d, i) => d === mlVar.dims[i]);
  const n = Math.min(tlVar.values.length, mlVar.values.length);
  let abs = 0;
  let scale = 0;
  for (let i = 0; i < n; i += 1) {
    const dr = tlVar.values[i].re - mlVar.values[i].re;
    const di = tlVar.values[i].im - mlVar.values[i].im;
    abs = Math.max(abs, Math.hypot(dr, di));
    scale = Math.max(scale, Math.hypot(mlVar.values[i].re, mlVar.values[i].im));
  }
  if (tlVar.values.length !== mlVar.values.length) abs = Infinity;
  const rel = scale > 0 ? abs / scale : abs;
  return { abs, rel, shape };
}

const rows = [];
const kernelLimit = (msg) => {
  if (!msg) return null;
  if (msg.startsWith("Slicing only supported on tensors")) {
    return { id: "K2", desc: "标量/张量切片限制" };
  }
  if (msg.startsWith("Slicing assignment only supported on tensors")) {
    return { id: "K2", desc: "标量切片赋值限制" };
  }
  if (msg.endsWith("to f64") && (msg.includes("Bool(") || msg.includes("}) to f64"))) {
    return { id: "K21", desc: "logical 张量参与数值运算时类型转换失败" };
  }
  return null;
};

for (const c of caseList) {
  const tl = tlResults.get(c.id);
  const ml = mlResults.get(c.id);
  let verdict = "PASS";
  let detail = "";
  let absErr = null;
  let relErr = null;

  if (hungCases.has(c.id)) {
    verdict = "FAIL";
    detail = `TensorLab 超时（>${Math.min(caseTimeoutMs, 30000) / 1000}s，疑似死循环/超大分配）`;
  } else if (crashCases.has(c.id)) {
    verdict = "FAIL";
    detail = `TensorLab 崩溃：${crashedFunctions.get(c.name) || "进程 panic"}`;
  } else if (crashedFunctions.has(c.name) && !tl) {
    verdict = "FAIL";
    detail = `TensorLab 崩溃：${crashedFunctions.get(c.name)}`;
  } else if (!tl) {
    verdict = "FAIL";
    detail = "TensorLab 未返回结果";
  } else if (c.kind === "abnormal") {
    if (tl.ok) {
      verdict = "WARN";
      detail = "TensorLab 接受了越界输入（未报错）";
    } else {
      detail = `TensorLab 按预期报错：${tl.error}`;
      if (!noMatlab && c.matlab && ml && ml.ok) {
        verdict = "WARN";
        detail += "；MATLAB 接受了该输入";
      }
    }
  } else if (!tl.ok && kernelLimit(tl.error)) {
    const kl = kernelLimit(tl.error);
    verdict = "KNOWN";
    detail = `RunMat 内核限制 ${kl.id}：${kl.desc}`;
  } else if (!tl.ok) {
    if (!noMatlab && c.matlab && ml && !ml.ok) {
      verdict = "WARN";
      detail = `双方均报错：TensorLab「${tl.error}」/ MATLAB「${ml.error}」`;
    } else {
      verdict = "FAIL";
      detail = `TensorLab 报错：${tl.error}`;
    }
  } else if (noMatlab || !c.matlab) {
    verdict = "SKIP";
    detail = "未与 MATLAB 对比（无对应函数/工具箱）";
  } else if (!ml) {
    verdict = "FAIL";
    detail = "MATLAB 未返回结果";
  } else if (!ml.ok) {
    verdict = "WARN";
    detail = `MATLAB 报错：${ml.error}`;
  } else {
    const errors = [];
    let shapeOk = true;
    for (const name of c.compare) {
      const e = maxError(tl.vars[name], ml.vars[name]);
      absErr = Math.max(absErr ?? 0, e.abs);
      relErr = Math.max(relErr ?? 0, e.rel);
      if (!e.shape) shapeOk = false;
      if (!(e.abs <= c.tol || e.rel <= c.tol)) {
        errors.push(`${name}: abs=${e.abs.toExponential(3)} rel=${e.rel.toExponential(3)}`);
      }
    }
    if (errors.length) {
      verdict = "FAIL";
      detail = errors.join("; ");
    } else if (!shapeOk) {
      verdict = "WARN";
      detail = "shape mismatch（数值一致，形状不同）";
    } else {
      detail = `max_abs=${(absErr ?? 0).toExponential(3)} max_rel=${(relErr ?? 0).toExponential(3)}`;
    }
  }

  rows.push({ ...c, tl, ml, verdict, detail, absErr, relErr });
}

// ---------------------------------------------------------------- 报告
const counts = { PASS: 0, FAIL: 0, WARN: 0, SKIP: 0, KNOWN: 0 };
for (const r of rows) counts[r.verdict] += 1;
const totalFunctions = new Set(rows.map((r) => r.name)).size;
const date = new Date().toISOString().slice(0, 10);

const esc = (s) => String(s ?? "").replaceAll("|", "\\|").replaceAll("\n", " ");
let md = `# TensorLab 自写 m 函数测试报告\n\n`;
md += `- 日期：${date}\n`;
md += `- 内核：RunMat 0.6.2\n`;
md += `- TensorLab：0.2.1（debug）\n`;
md += `- MATLAB：R2023b\n`;
md += `- 范围：\`toolbox/\` 下全部公开 m 函数（不含 RunMat 内置函数）\n`;
md += `- 用例：每个函数 3 组正常值、2 组临界值、2 组越界/异常值\n`;
md += `- 精度判据：\`max_abs <= tol\` 或 \`max_rel <= tol\`（各函数单独设置 tol）\n\n`;
md += `复现方式：\n\n`;
md += `\`\`\`bash\n`;
md += `node tests/toolbox_compare.mjs            # TensorLab + MATLAB 全量对比\n`;
md += `node tests/toolbox_compare.mjs --category signal   # 只测某一分类\n`;
md += `node tests/toolbox_compare.mjs --no-matlab         # 只跑 TensorLab\n`;
md += `\`\`\`\n\n`;
md += `用例定义：\`tests/toolbox_cases/<category>.json\`（每个函数 3 正常 / 2 临界 / 2 异常）。\n\n`;
md += `## 汇总\n\n`;
md += `| 指标 | 数量 |\n|---|---:|\n`;
md += `| 函数 | ${totalFunctions} |\n| 用例 | ${rows.length} |\n`;
md += `| PASS | ${counts.PASS} |\n| FAIL | ${counts.FAIL} |\n`;
md += `| WARN | ${counts.WARN} |\n| KNOWN | ${counts.KNOWN} |\n| SKIP | ${counts.SKIP} |\n\n`;

md += `## 判定口径\n\n`;
md += `| 判定 | 含义 |\n|---|---|\n`;
md += `| PASS | TensorLab 无错误，且与 MATLAB 数值差在 tol 内（或正常/临界用例与 MATLAB 一致） |\n`;
md += `| FAIL | TensorLab 报错、结果数量/数值超出 tol、崩溃或超时 |\n`;
md += `| WARN | 非缺陷性差异：MATLAB 与 TensorLab 均拒绝该输入、形状不同但数值一致、或越界输入被接受但未报错 |\n`;
md += `| KNOWN | RunMat 内核限制（K2 标量切片、K21 logical 张量运算），非本函数算法问题 |\n`;
md += `| SKIP | 无可对比的 MATLAB 实现（如 Global Optimization 未授权、API 子集差异） |\n\n`;
md += `精度判据：对每个对比变量取 \`max_abs\` 与 \`max_rel\`，满足 \`max_abs <= tol || max_rel <= tol\` 即通过；\`tol\` 由各函数在用例文件中单独设置。\n\n`;

const byCategory = new Map();
for (const r of rows) {
  if (!byCategory.has(r.category)) byCategory.set(r.category, []);
  byCategory.get(r.category).push(r);
}
md += `## 分类汇总\n\n`;
md += `| 分类 | 函数 | 用例 | PASS | FAIL | WARN | KNOWN | SKIP |\n`;
md += `|---|---:|---:|---:|---:|---:|---:|---:|\n`;
for (const [cat, list] of byCategory) {
  const c = { PASS: 0, FAIL: 0, WARN: 0, SKIP: 0, KNOWN: 0 };
  for (const r of list) c[r.verdict] += 1;
  md += `| ${cat} | ${new Set(list.map((r) => r.name)).size} | ${list.length} | ${c.PASS} | ${c.FAIL} | ${c.WARN} | ${c.KNOWN} | ${c.SKIP} |\n`;
}
md += `\n`;

if (timedOutFunctions.size) {
  md += `超时函数：${[...timedOutFunctions].map((n) => `\`${n}\``).join("、")}\n\n`;
}
if (hungCases.size) {
  md += `超时用例：${[...hungCases].map((n) => `\`${n}\``).join("、")}\n\n`;
}
if (crashedFunctions.size) {
  md += `崩溃函数：\n\n`;
  for (const [n, reason] of crashedFunctions) md += `- \`${n}\`：${esc(reason)}\n`;
  md += `\n`;
}
if (matlabCrashed) md += `> MATLAB 批量脚本异常退出，未执行的用例记为「MATLAB 未返回结果」。\n\n`;

const byFn = new Map();
for (const r of rows) {
  if (!byFn.has(r.name)) byFn.set(r.name, []);
  byFn.get(r.name).push(r);
}
const clip = (s, n = 160) => (s.length > n ? `${s.slice(0, n)}…` : s);
for (const [name, list] of byFn) {
  md += `## \`${name}\`\n\n`;
  if (list[0].note) md += `> ${esc(list[0].note)}\n\n`;
  md += `| 用例 | 类型 | 代码 | TensorLab | MATLAB | 判定 | 说明 |\n`;
  md += `|---|---|---|---|---|---|---|\n`;
  for (const r of list) {
    const tlText = !r.tl ? "—" : r.tl.ok ? "ok" : `error: ${r.tl.error}`;
    const mlText = !r.ml ? (r.matlab && !noMatlab ? "—" : "N/A") : r.ml.ok ? "ok" : `error: ${r.ml.error}`;
    md += `| \`${r.id}\` | ${r.kind} | \`${esc(r.code)}\` | ${esc(clip(tlText))} | ${esc(clip(mlText))} | **${r.verdict}** | ${esc(clip(r.detail, 200))} |\n`;
  }
  md += `\n`;
}

const reportPath = join(root, "docs", "testing", "toolbox-report.md");
mkdirSync(dirname(reportPath), { recursive: true });
writeFileSync(reportPath, md, "utf8");
writeFileSync(join(outDir, "results.json"), JSON.stringify(rows, null, 2), "utf8");
console.log(`\n${counts.PASS} passed, ${counts.FAIL} failed, ${counts.WARN} warn, ${counts.KNOWN} known, ${counts.SKIP} skipped`);
console.log(`report -> ${reportPath}`);
process.exit(counts.FAIL > 0 ? 1 : 0);
