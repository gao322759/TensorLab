//! TensorLab L2 — 内核引擎门面。
//!
//! 封装 `runmat_core::RunMatSession`：
//! - 阻塞执行 m 源码（内部驱动 async，同步语义）
//! - 事件提取（streams / display_events / diagnostics）
//! - 变量快照（workspace_snapshot → VariableInfo）
//! - 图窗导出（figures_touched → FigureScene JSON）
//!
//! 三个 App（CLI / Tauri / MCP）都经由本层，绝不直接触碰 runmat 类型。

use runmat_core::abi::{
    DiagnosticSeverity, DisplayLabel, ExecutionRequest, HostExecutionPolicy, SourceInput,
};
use runmat_core::abi::{ExecutionOutcome, RuntimeDiagnostic};
use runmat_core::{
    CompatMode, ExecutionStreamKind, NumericPreviewValue, RunError, RunMatSession,
    WorkspaceMaterializeOptions, WorkspaceMaterializeTarget,
};
use std::time::Instant;
use tl_types::{
    body_text, echo_line, DiagnosticPayload, FigurePayload, RunReport, StreamEntryPayload,
    VariableInfo,
};

/// 变量面板单次最多展示的数值个数（快照默认只有 16 个）。
const WORKSPACE_PREVIEW_ELEMENTS: usize = 128;
/// 超过该元素数不再逐变量物化，避免大矩阵拖慢每次 run。
const WORKSPACE_MATERIALIZE_LIMIT: usize = 4096;

pub struct Engine {
    session: RunMatSession,
}

/// 内核执行统计（JIT/VM 诊断用，不进入冻结的 IPC 契约）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KernelStats {
    pub total_executions: usize,
    pub jit_compiled: usize,
    pub interpreter_fallback: usize,
    pub native_osr_transfers: usize,
    pub vectorized_native_regions: u64,
}

impl Engine {
    pub fn new() -> anyhow::Result<Self> {
        Self::with_jit(true)
    }

    /// 创建引擎并显式选择是否启用 Cranelift JIT。
    ///
    /// - `true`：默认路径，RunMat tiering 在重复执行后进入 JIT 层；
    /// - `false`：解释器-only，用于 VM/JIT 性能对照与回归测试。
    pub fn with_jit(enable_jit: bool) -> anyhow::Result<Self> {
        let session = RunMatSession::with_options(enable_jit, false)?;
        let mut engine = Engine { session };
        engine.mount_toolbox();
        Ok(engine)
    }

    /// 把仓库根 `toolbox/` 挂到内核搜索路径（见 docs/decisions.md ADR-012）。
    ///
    /// 目录下的纯 m-script 函数（`toolbox/<domain>/<fn>.m`）经 `addpath(genpath(...))`
    /// 后即可被 m 代码调用，用于补齐内核未提供的 R2026a 函数。
    /// 找不到目录时静默跳过（发行物可能不含源目录）。
    fn mount_toolbox(&mut self) {
        if let Some(dir) = toolbox_dir() {
            let path = dir.replace('\\', "/");
            let src = format!("addpath(genpath('{path}'));");
            let report = self.run("<toolbox>", &src);
            if report.error.is_some() {
                // 非致命：工具箱缺失不应阻断引擎可用性
                let _ = std::io::Write::write_all(
                    &mut std::io::stderr(),
                    b"warning: failed to mount m-script toolbox directory\n",
                );
            }
        }
    }

    /// 执行一段 m 源码（脚本或单行），返回完整运行报告。
    pub fn run(&mut self, source_name: &str, source: &str) -> RunReport {
        let started = Instant::now();
        let mut report = self.run_inner(source_name, source);
        if report.duration_ms == 0.0 {
            report.duration_ms = started.elapsed().as_secs_f64() * 1000.0;
        }
        report
    }

    /// 执行磁盘上的 m 文件（`SourceInput::Path` 请求）。
    ///
    /// 与 [`Engine::run`] 的区别：内核按文件路径解析源码，并自动加载**同目录**的
    /// `classdef` 类文件。RunMat 0.6.2 的 classdef 只在 Path 请求下生效
    /// （见 docs/kernel-notes.md 坑 31）。
    ///
    /// `source` 由调用方读入，仅用于诊断行号换算，避免同一文件读两遍。
    pub fn run_path(&mut self, path: &str, source: &str) -> RunReport {
        let started = Instant::now();
        // 规范化路径，确保内核能定位同目录的 class 文件；失败时退回原路径。
        let canonical = std::fs::canonicalize(path)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string());
        let mut report = self.execute(SourceInput::Path(canonical), source);
        if report.duration_ms == 0.0 {
            report.duration_ms = started.elapsed().as_secs_f64() * 1000.0;
        }
        report
    }

    fn run_inner(&mut self, name: &str, source: &str) -> RunReport {
        self.execute(
            SourceInput::Text {
                name: name.to_string(),
                text: source.to_string(),
            },
            source,
        )
    }

    fn execute(&mut self, input: SourceInput, source: &str) -> RunReport {
        let request = ExecutionRequest::for_source(
            input,
            CompatMode::Matlab,
            HostExecutionPolicy::default(),
            self.session.workspace_handle(),
        );

        let response = futures::executor::block_on(self.session.execute_request(request));
        match response.result {
            Ok(outcome) => self.outcome_to_report(&outcome, source),
            Err(err) => error_report(&err),
        }
    }

    /// 请求取消当前执行（协作式；由内核在语句边界响应）
    pub fn cancel(&self) {
        self.session.cancel_execution();
    }

    /// 读取内核累计执行统计（用于 `tensorlab bench` / 性能诊断）。
    pub fn kernel_stats(&self) -> KernelStats {
        let s = self.session.stats();
        KernelStats {
            total_executions: s.total_executions,
            jit_compiled: s.jit_compiled,
            interpreter_fallback: s.interpreter_fallback,
            native_osr_transfers: s.native_osr_transfers,
            vectorized_native_regions: s.vectorized_native_regions,
        }
    }

    /// 变量面板快照
    ///
    /// 内核 `workspace_snapshot()` 每个 entry 自带最多 16 个数值的预览
    /// （`WORKSPACE_PREVIEW_LIMIT`，见 runmat-core `workspace::emit`）。为了
    /// 让桌面端变量面板足够实用，这里对 **≤4096 元素**的 CPU 变量调用
    /// `materialize_variable`，把预览提升到最多 128 个数值；更大的矩阵或
    /// GPU 变量仍使用快照预览，避免每次 run 都复制大缓冲区。
    pub fn variables(&mut self) -> Vec<VariableInfo> {
        let snap = self.session.workspace_snapshot();
        snap.values
            .into_iter()
            .map(|e| {
                let total_elements = e.shape.iter().product::<usize>().max(1);
                let mut preview = e
                    .preview
                    .as_ref()
                    .map(|p| format_preview(&p.values, p.truncated, total_elements))
                    .unwrap_or_default();

                if !e.is_gpu && total_elements > 16 && total_elements <= WORKSPACE_MATERIALIZE_LIMIT
                {
                    let options = WorkspaceMaterializeOptions {
                        max_elements: WORKSPACE_PREVIEW_ELEMENTS.min(total_elements),
                        slice: None,
                    };
                    if let Ok(materialized) =
                        futures::executor::block_on(self.session.materialize_variable(
                            WorkspaceMaterializeTarget::Name(e.name.clone()),
                            options,
                        ))
                    {
                        if let Some(p) = materialized.preview {
                            preview = format_preview(&p.values, p.truncated, total_elements);
                        }
                    }
                }

                VariableInfo {
                    name: e.name,
                    class_name: e.class_name,
                    shape: e.shape,
                    bytes: e.size_bytes,
                    preview,
                    is_gpu: e.is_gpu,
                }
            })
            .collect()
    }

    /// 清空工作区（clear all 语义）
    pub fn clear(&mut self) {
        self.session.clear_variables();
    }

    /// 导出全部图窗（运行后调用；逐个 handle 走 FigureScene 序列化）
    pub fn export_figures(&mut self, handles: &[u32]) -> Vec<FigurePayload> {
        let mut out = Vec::new();
        for &h in handles {
            let handle = runmat_runtime::builtins::plotting::FigureHandle::from(h);
            let exported = futures::executor::block_on(
                runmat_runtime::builtins::plotting::export_figure_scene(handle),
            );
            if let Ok(Some(bytes)) = exported {
                if let Ok(mut scene) = serde_json::from_slice(&bytes) {
                    strip_redundant_scene_fields(&mut scene);
                    out.push(FigurePayload {
                        handle: h,
                        scene_json: scene,
                    });
                }
            }
        }
        out
    }

    // ---- 内部：Outcome → RunReport ----

    fn outcome_to_report(&mut self, outcome: &ExecutionOutcome, source: &str) -> RunReport {
        let mut events: Vec<StreamEntryPayload> = Vec::new();

        // 1. streams（stdout/stderr/clc），保持时间序
        for s in &outcome.streams {
            match s.stream {
                ExecutionStreamKind::Stdout => events.push(StreamEntryPayload::Output {
                    text: s.text.clone(),
                }),
                ExecutionStreamKind::Stderr => events.push(StreamEntryPayload::Error {
                    text: s.text.clone(),
                }),
                ExecutionStreamKind::ClearScreen => events.push(StreamEntryPayload::ClearConsole),
            }
        }

        // 2. 无分号回显
        //
        // 内核 `console::record_value_output` 已把按 `format short/long` 格式化好的
        // 回显写进 console 缓冲（→ 上面的 streams），那才是权威来源。
        // `display_events` 只携带匿名语句值（无 label）：若两者都渲染，就会出现
        // `x = 42` 后面多一行 `42`、`ans = 2` 后面多一行 `2`。
        // 故仅当本次执行**完全没有** console 输出时才用 display_events 兜底
        // （宿主将来关掉内核回显时，值仍能显示）。
        if events.is_empty() {
            for d in &outcome.display_events {
                let text = match &d.label {
                    // BindingName 是 String 新类型（BindingName(pub String)）
                    DisplayLabel::Binding(name) => echo_line(&name.0, &d.value),
                    DisplayLabel::Literal(name) => echo_line(name, &d.value),
                    DisplayLabel::Anonymous => body_text(&d.value),
                };
                events.push(StreamEntryPayload::Output { text });
            }
        }

        // 3. 诊断分级
        let mut error = None;
        let mut warnings = Vec::new();
        for d in &outcome.diagnostics {
            let payload = diagnostic_payload(d, source);
            match d.severity {
                DiagnosticSeverity::Error => {
                    if error.is_none() {
                        error = Some(payload);
                    }
                }
                DiagnosticSeverity::Warning => warnings.push(payload),
                _ => {}
            }
        }

        // 4. 变量 + 图窗
        let variables = self.variables();
        let figures = self.export_figures(&outcome.figures_touched);

        RunReport {
            events,
            result_text: None,
            error,
            warnings,
            variables,
            figures,
            duration_ms: outcome.execution_time_ms as f64,
            used_jit: outcome.used_jit,
        }
    }
}

/// 定位仓库根的 `toolbox/`（编译期以 tl-kernel 的 manifest 目录向上两级为工作区根）
fn toolbox_dir() -> Option<String> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("toolbox");
    if path.is_dir() {
        Some(path.to_string_lossy().to_string())
    } else {
        None
    }
}

fn diagnostic_payload(d: &RuntimeDiagnostic, source: &str) -> DiagnosticPayload {
    DiagnosticPayload {
        identifier: Some(d.code.clone()),
        message: d.message.clone(),
        // Span 是字节偏移（Span { start: usize, end: usize }），按换行计数换算 1-based 行号
        line: d.span.as_ref().map(|s| {
            let offset = s.start.min(source.len());
            source.as_bytes()[..offset]
                .iter()
                .filter(|&&b| b == b'\n')
                .count()
                + 1
        }),
        callstack: d.callstack.clone(),
        severity: match d.severity {
            DiagnosticSeverity::Error => "error".into(),
            DiagnosticSeverity::Warning => "warning".into(),
            DiagnosticSeverity::Info => "info".into(),
            DiagnosticSeverity::Hint => "hint".into(),
        },
    }
}

fn error_report(err: &RunError) -> RunReport {
    let mut report = RunReport::default();
    let payload = match err {
        RunError::Runtime(re) => DiagnosticPayload {
            identifier: re.identifier().map(str::to_string),
            message: re.message().to_string(),
            line: None,
            callstack: Vec::new(),
            severity: "error".into(),
        },
        other => DiagnosticPayload {
            identifier: None,
            message: format!("{other}"),
            line: None,
            callstack: Vec::new(),
            severity: "error".into(),
        },
    };
    report.error = Some(payload);
    report
}

/// 剥离 FigureScene 中的冗余字段。
///
/// 内核 `export_figure_scene` 为图窗 replay（可无损重建）而导出，每个 plot 同时携带
/// 物化后的 `x`/`y`/`z`… 数组与原始 `*_source`（SceneNumericData），二者内容完全重复：
///   `x = x_data.materialize_f64()` 且 `x_source = Some((&x_data).into())`（runmat-plot
///   `ScenePlot::from_plot_for_export`）。宿主只需渲染，`x`/`y` 已足够，保留 `*_source`
///   会让 IPC payload 体积与 JSON 解析耗时各翻一倍（大数组时是主要瓶颈）。
///
/// 注意：剥离后该 payload **不再可用于 `import_figure_scene` 无损回灌**——它是渲染载荷，
/// 不是 round-trip 载荷。
fn strip_redundant_scene_fields(scene: &mut serde_json::Value) {
    let figure = match scene.get_mut("figure") {
        Some(f) => f,
        None => scene,
    };
    let Some(plots) = figure.get_mut("plots").and_then(|p| p.as_array_mut()) else {
        return;
    };
    for plot in plots {
        if let Some(obj) = plot.as_object_mut() {
            obj.retain(|k, _| !k.ends_with("_source"));
        }
    }
}

/// whos 风格表格（CLI / 面板输出）
pub fn variable_table(vars: &[VariableInfo]) -> String {
    let mut out = String::from("  NAME           SIZE            BYTES  CLASS\n");
    out.push_str(&format!(
        "  {}  {}  {}\n",
        "-".repeat(12),
        "-".repeat(15),
        "-".repeat(15)
    ));
    for v in vars {
        let shape = if v.shape.len() >= 2 {
            format!("{}x{}", v.shape[0], v.shape[1])
        } else if v.shape.len() == 1 {
            format!("{}x1", v.shape[0])
        } else {
            "0x0".into()
        };
        out.push_str(&format!(
            "  {:<14} {:<10} {:>10}  {}\n",
            v.name,
            shape,
            v.bytes.unwrap_or(0),
            v.class_name
        ));
    }
    out
}

/// 把内核数值预览格式化为 MATLAB 风格单行文本。
///
/// 标量（总元素数 1）→ `3.1416`；向量/矩阵 → `[1 2 3 …]`（被截断时加省略号）。
fn format_preview(
    values: &[NumericPreviewValue],
    truncated: bool,
    total_elements: usize,
) -> String {
    if values.is_empty() {
        return String::new();
    }
    let bracketed = total_elements > 1;
    let mut out = String::new();
    if bracketed {
        out.push('[');
    }
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        match v {
            NumericPreviewValue::Float(f) => out.push_str(&runmat_value::format_number(*f)),
            NumericPreviewValue::Integer(iv) => out.push_str(&iv.decimal_string()),
        }
    }
    if truncated {
        out.push_str(" …");
    }
    if bracketed {
        out.push(']');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::strip_redundant_scene_fields;
    use serde_json::json;

    #[test]
    fn strips_source_duplicates_but_keeps_render_data() {
        let mut scene = json!({
            "createdAt": "t",
            "figure": {
                "layout": { "axesRows": 1, "axesCols": 1 },
                "metadata": { "axesMetadata": [] },
                "plots": [
                    {
                        "kind": "line",
                        "x": [0.0, 1.0, 2.0],
                        "y": [0.0, 1.0, 0.0],
                        "x_source": { "shape": [1, 3], "storage": { "dtype": "f64", "values": [0.0, 1.0, 2.0] } },
                        "y_source": { "shape": [1, 3], "storage": { "dtype": "f64", "values": [0.0, 1.0, 0.0] } },
                        "color_rgba": [0.0, 0.0, 1.0, 1.0],
                        "line_width": 2.0,
                        "line_style": "Solid",
                        "axes_index": 0,
                        "label": "sin(x)",
                        "visible": true
                    }
                ]
            }
        });
        strip_redundant_scene_fields(&mut scene);

        let plot = &scene["figure"]["plots"][0];
        // 渲染所需字段保留
        assert_eq!(plot["x"], json!([0.0, 1.0, 2.0]));
        assert_eq!(plot["y"], json!([0.0, 1.0, 0.0]));
        assert_eq!(plot["kind"], "line");
        assert_eq!(plot["label"], "sin(x)");
        // 冗余字段已剥离
        assert!(plot.get("x_source").is_none());
        assert!(plot.get("y_source").is_none());
        assert!(!plot.to_string().contains("_source"));
    }

    #[test]
    fn strip_is_defensive_on_unexpected_shapes() {
        // 无 envelope / 无 plots / plots 非数组：不得 panic
        let mut bare = json!({ "plots": [] });
        strip_redundant_scene_fields(&mut bare);
        let mut odd = json!({ "figure": { "plots": "not-an-array" } });
        strip_redundant_scene_fields(&mut odd);
        assert!(odd["figure"]["plots"].is_string());
    }

    #[test]
    fn formats_scalar_and_vector_previews() {
        use super::format_preview;
        use runmat_core::NumericPreviewValue;

        // 标量：不加方括号
        let scalar = [NumericPreviewValue::Float(3.14159)];
        let s = format_preview(&scalar, false, 1);
        assert!(!s.starts_with('['), "scalar must not be bracketed: {s}");

        // 向量：方括号 + 空格分隔
        let vec_vals = [
            NumericPreviewValue::Float(1.0),
            NumericPreviewValue::Float(2.0),
            NumericPreviewValue::Float(3.0),
        ];
        assert_eq!(format_preview(&vec_vals, false, 3), "[1 2 3]");

        // 截断：省略号在方括号内
        let truncated = format_preview(&vec_vals, true, 1000);
        assert_eq!(truncated, "[1 2 3 …]");

        // 空预览：空串（非数值类型内核不产出预览）
        assert_eq!(format_preview(&[], false, 1), "");
    }

    #[test]
    fn fresh_engine_has_empty_workspace() {
        let mut engine = super::Engine::new().expect("engine");
        assert!(engine.variables().is_empty());
    }

    #[test]
    fn kernel_stats_report_vm_fallback_when_jit_is_disabled() {
        let mut engine = super::Engine::with_jit(false).expect("engine");
        for _ in 0..3 {
            let report = engine.run("<stats-test>", "x = 1 + 1;");
            assert!(report.error.is_none(), "{:?}", report.error);
        }
        let stats = engine.kernel_stats();
        assert!(stats.total_executions >= 3);
        assert_eq!(stats.jit_compiled, 0);
        assert!(stats.interpreter_fallback >= 3);
    }

    #[test]
    fn variable_preview_expands_small_vectors_beyond_snapshot_limit() {
        let mut engine = super::Engine::new().expect("engine");
        let report = engine.run("<preview-test>", "v = 1:100;");
        assert!(report.error.is_none(), "{:?}", report.error);
        let vars = engine.variables();
        let v = vars.iter().find(|x| x.name == "v").expect("v");
        // 快照默认只给 16 个值；物化后应覆盖完整 1x100。
        assert!(v.preview.contains("100"), "preview: {}", v.preview);
        assert!(v.preview.len() > 200, "preview too short: {}", v.preview);
    }

    #[test]
    fn variable_table_formats_header_shape_and_class() {
        use tl_types::VariableInfo;
        let vars = vec![
            VariableInfo {
                name: "a".into(),
                class_name: "double".into(),
                shape: vec![1, 5],
                bytes: Some(40),
                preview: "[1 2 3 4 5]".into(),
                is_gpu: false,
            },
            VariableInfo {
                name: "b".into(),
                class_name: "char".into(),
                shape: vec![2, 2],
                bytes: Some(4),
                preview: String::new(),
                is_gpu: false,
            },
        ];
        let table = super::variable_table(&vars);
        assert!(table.starts_with("  NAME"), "{table}");
        assert!(table.contains("1x5"), "{table}");
        assert!(table.contains("2x2"), "{table}");
        assert!(table.contains("double"), "{table}");
        assert!(table.contains("char"), "{table}");
        assert!(table.contains('a') && table.contains('b'), "{table}");
    }

    #[test]
    fn variable_table_handles_empty_and_unknown_shapes() {
        assert!(super::variable_table(&[]).contains("NAME"));
        // shape 为空的变量不应 panic
        let vars = vec![tl_types::VariableInfo {
            name: "z".into(),
            class_name: "double".into(),
            shape: vec![],
            bytes: None,
            preview: String::new(),
            is_gpu: false,
        }];
        let table = super::variable_table(&vars);
        assert!(table.contains("0x0"), "{table}");
    }
}
