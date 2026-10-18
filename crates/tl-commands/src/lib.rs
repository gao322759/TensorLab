//! TensorLab L8 — 应用门面。
//!
//! 三个 App（CLI / Tauri / MCP）的唯一入口：
//! - `run` / `eval`：m 代码执行
//! - `host_command`：宿主命令分派（help/vars/clear/figures/version）
//! - `demos`：演示脚本验收（demos are the spec）
//!
//! 使用方**只** import 本 crate（+tl-types 载荷），不触碰下层。

pub use tl_kernel::Engine;
pub use tl_toolbox::CommandResult;
pub use tl_types::RunReport;

/// 宿主命令执行入口
pub fn host_command(engine: &mut Engine, name: &str, args: &[String]) -> Option<CommandResult> {
    tl_toolbox::find(name).map(|cmd| cmd.run(engine, args))
}

/// 运行 m 源码（脚本）
pub fn run_source(engine: &mut Engine, name: &str, source: &str) -> RunReport {
    engine.run(name, source)
}

/// 运行 m 文件
///
/// 走 `SourceInput::Path` 请求，使内核能加载同目录的 `classdef` 类文件。
pub fn run_file(engine: &mut Engine, path: &str) -> anyhow::Result<RunReport> {
    let src = std::fs::read_to_string(path)?;
    Ok(engine.run_path(path, &src))
}

/// 一行 REPL 输入：宿主命令（无括号、非 m 语法）或 m 代码
pub fn repl_line(engine: &mut Engine, line: &str) -> CommandResult {
    let trimmed = line.trim();
    // 空行
    if trimmed.is_empty() {
        return CommandResult::None;
    }
    // 第一段（宿主命令名候选）
    let first_word: String = trimmed
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    // help 主题查询：help fft / help('fft') —— 内核无 help 内置函数，宿主层拦截
    // （裸 `help` / `help commands` 走下方宿主命令路径）
    if first_word == "help" {
        if let Some(rest) = trimmed.strip_prefix("help") {
            let rest = rest.trim();
            // `help commands` 与裸 `help` 一样列出宿主命令
            if !rest.is_empty() && rest != "commands" {
                let topic = rest
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .trim()
                    .trim_matches(['\'', '"']);
                // 宿主命令优先（mex/vars/help/...），否则回落到内核内置主题表
                if tl_toolbox::find(topic).is_some() {
                    if let Some(result) = host_command(engine, "help", &[topic.to_string()]) {
                        return result;
                    }
                }
                return CommandResult::Text(tl_interpreter::help_topic(topic));
            }
        }
    }
    let looks_like_code = trimmed.contains('=')
        || trimmed.contains('(')
        || trimmed.contains(';')
        || trimmed.ends_with('\'');
    if !looks_like_code {
        if let Some(result) = host_command(engine, &first_word, &split_args(trimmed)) {
            return result;
        }
    }
    // 否则交给内核
    CommandResult::Report(engine.run("repl", trimmed))
}

fn split_args(line: &str) -> Vec<String> {
    let mut words = line.split_whitespace();
    words.next(); // 命令名
    words.map(str::to_string).collect()
}

/// demo 目录约定（工作区根 / demos/）
pub const DEMOS: &[(&str, &str)] = &[
    ("hello_world", "demos/hello_world.m"),
    ("matrix_ops", "demos/matrix_ops.m"),
    ("plot_demo", "demos/plot_demo.m"),
    ("fft_demo", "demos/fft_demo.m"),
    ("solve_demo", "demos/solve_demo.m"),
    ("signal_toolbox", "demos/signal_toolbox.m"),
    ("image_toolbox", "demos/image_toolbox.m"),
    ("stats_toolbox", "demos/stats_toolbox.m"),
    ("optim_demo", "demos/optim_demo.m"),
    ("control_demo", "demos/control_demo.m"),
];

/// 跑全部 demo（相对工作区根）；返回 (脚本, 报告) 序列
pub fn run_all_demos(engine: &mut Engine) -> anyhow::Result<Vec<(String, RunReport)>> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let mut out = Vec::new();
    for (name, rel) in DEMOS {
        let path = root.join(rel);
        let src = std::fs::read_to_string(&path)?;
        // 文件执行走 Path 请求（demo 也可能依赖同目录 classdef）
        let report = engine.run_path(&path.to_string_lossy(), &src);
        out.push((name.to_string(), report));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 内核的 figure/路径状态是进程级全局，测试并行执行会互相污染
    /// （见 `demo_reports_have_expected_shape` 的注释）。所有创建 Engine 的
    /// 测试都经此串行化，保证 `cargo test` 默认并行时仍然确定性。
    fn locked_engine() -> (std::sync::MutexGuard<'static, ()>, Engine) {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        (guard, Engine::new().expect("engine"))
    }

    #[test]
    fn toolbox_m_scripts_are_mounted() {
        // ADR-012：Engine::new() 应把仓库根 toolbox/ 挂到内核搜索路径，
        // 使纯 m-script 工具箱函数（xcorr/findpeaks/zscore/...）可被调用。
        let (_guard, mut engine) = locked_engine();
        let report = engine.run(
            "toolbox_probe",
            "r = xcorr([1 2 3]); [p, l] = findpeaks([1 3 2 5 4]); \
             q = iqr(1:8); m = moment([1 2 3 4], 2); g = mat2gray([2 4 6]); \
             [xopt, fopt] = fminsearch(@(v) (v(1)-3)^2 + (v(2)+1)^2, [0 0]);",
        );
        assert!(
            report.error.is_none(),
            "toolbox functions failed: {:?}",
            report.error.as_ref().map(|e| &e.message)
        );
        let preview = |name: &str| -> String {
            report
                .variables
                .iter()
                .find(|v| v.name == name)
                .map(|v| v.preview.clone())
                .unwrap_or_default()
        };
        assert!(
            preview("r").contains("14"),
            "xcorr preview: {}",
            preview("r")
        );
        assert_eq!(preview("p"), "[3 5]", "findpeaks peaks");
        assert_eq!(preview("l"), "[2 4]", "findpeaks locs");
        assert_eq!(preview("q").parse::<f64>().unwrap_or(-1.0), 4.0, "iqr");
        assert_eq!(preview("m").parse::<f64>().unwrap_or(-1.0), 1.25, "moment");
        assert!(
            preview("g").contains("0.5"),
            "mat2gray preview: {}",
            preview("g")
        );
        assert!(
            preview("xopt").contains('3'),
            "fminsearch xopt preview: {}",
            preview("xopt")
        );
    }

    #[test]
    fn toolbox_covers_image_and_control_batches() {
        // 回归覆盖：图像批次（imresize/medfilt2/bwlabel/imfill）与
        // 控制批次（acker/margin）+ 优化（lsqnonneg）。
        let (_guard, mut engine) = locked_engine();
        let report = engine.run(
            "toolbox_probe2",
            "B = imresize([1 2 3; 4 5 6; 7 8 9], 2, 'nearest'); \
             M = medfilt2([1 2 3; 4 5 6; 7 8 9]); \
             [L2, nb] = bwlabel([0 0 1 0; 0 0 1 0; 1 0 0 1; 1 0 0 1]); \
             F = imfill([1 1 1; 1 0 1; 1 1 1], 'holes'); \
             Fmid = F(2, 2); \
             K = acker([0 1; 0 0], [0; 1], [-1 -2]); \
             [Gm, Pmv, Wcg, Wcpv] = margin(tf(1, [1 1 0])); \
             xn = lsqnonneg([1 0; 0 1; 1 1], [1; 1; 3]);",
        );
        assert!(
            report.error.is_none(),
            "toolbox batch failed: {:?}",
            report.error.as_ref().map(|e| &e.message)
        );
        let var = |name: &str| report.variables.iter().find(|v| v.name == name);
        assert_eq!(var("B").expect("B").shape, vec![6, 6], "imresize shape");
        assert_eq!(var("nb").expect("nb").preview.parse::<f64>().unwrap(), 2.0);
        assert_eq!(
            var("Fmid").expect("Fmid").preview.parse::<f64>().unwrap(),
            1.0
        );
        let gain: Vec<f64> = var("K")
            .expect("K")
            .preview
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split_whitespace()
            .map(|t| t.parse::<f64>().expect("gain token"))
            .collect();
        assert_eq!(gain, vec![2.0, 3.0], "acker gain");
        let pm: f64 = var("Pmv").expect("Pmv").preview.parse().unwrap();
        assert!((pm - 51.83).abs() < 0.2, "phase margin {pm}");
        assert!(
            var("xn").expect("xn").preview.contains("1.3333"),
            "lsqnonneg {}",
            var("xn").unwrap().preview
        );
        // medfilt2 中心值（3x3 中值 = 5）留作存在性断言
        assert!(!var("M").expect("M").preview.is_empty());
    }

    #[test]
    fn toolbox_mounts_control_optimization_and_signal_batches() {
        // 回归覆盖：控制（lsim/ss2tf/tf2zp）、优化（quadprog）、
        // 图像（bwdist/wiener2）、信号（stft）。
        let (_guard, mut engine) = locked_engine();
        let report = engine.run(
            "toolbox_probe3",
            "t = (0:0.1:5)'; u = ones(numel(t), 1); \
             [y, tt] = lsim(tf(1, [1 1]), u, t); \
             y1 = y(11); \
             [num, den] = ss2tf([0 1; 0 0], [0; 1], [1 0], 0); \
             [z, p, k] = tf2zp([1 2], [1 3 2]); \
             D = bwdist([0 0 0; 0 1 0; 0 0 0]); \
             d11 = D(1, 1); \
             W = wiener2([1 1 1; 1 5 1; 1 1 1]); \
             [S, F, T] = stft(ones(64, 1), 100, 16, 8, 16);",
        );
        assert!(
            report.error.is_none(),
            "toolbox batch 3 failed: {:?}",
            report.error.as_ref().map(|e| &e.message)
        );
        let var = |name: &str| report.variables.iter().find(|v| v.name == name);
        let y1: f64 = var("y1").expect("y1").preview.parse().unwrap();
        assert!((y1 - 0.6321).abs() < 0.01, "lsim step response: {y1}");
        let d11: f64 = var("d11").expect("d11").preview.parse().unwrap();
        assert!((d11 - 1.41421356).abs() < 1e-3, "bwdist corner: {d11}");
        assert_eq!(var("S").expect("S").shape, vec![9, 7], "stft shape");
        assert_eq!(var("num").expect("num").shape, vec![1, 3], "ss2tf num");
    }

    #[test]
    fn toolbox_struct_based_image_functions_are_usable() {
        // RunMat 0.6.2 的 struct 构造器可用（gaps K5 已关闭）：
        // strel 返回 struct，形态学函数接受 struct SE；bwconncomp/regionprops
        // 返回 struct 并可通过 s(k).Field 读取。
        let (_guard, mut engine) = locked_engine();
        let report = engine.run(
            "toolbox_probe_struct",
            "SE = strel('disk', 1); \
             nhood = sum(sum(SE.Neighborhood)); \
             A = [0 1 0; 0 1 0; 0 1 0]; \
             D = imdilate(A, SE); \
             area_d = sum(sum(D)); \
             cc = bwconncomp(A); \
             nobj = cc.NumObjects; \
             st = regionprops(A, 'Area'); \
             a1 = st(1).Area;",
        );
        assert!(
            report.error.is_none(),
            "struct toolbox probe failed: {:?}",
            report.error.as_ref().map(|e| &e.message)
        );
        let number = |name: &str| -> f64 {
            report
                .variables
                .iter()
                .find(|v| v.name == name)
                .map(|v| v.preview.parse().unwrap_or(-1.0))
                .unwrap_or(-1.0)
        };
        assert_eq!(number("nhood"), 5.0, "strel disk neighbourhood");
        assert_eq!(number("area_d"), 9.0, "imdilate with struct SE");
        assert_eq!(number("nobj"), 1.0, "bwconncomp NumObjects");
        assert_eq!(number("a1"), 3.0, "regionprops Area");
    }

    #[test]
    fn help_resolves_host_commands_before_builtin_topics() {
        let (_guard, mut engine) = locked_engine();
        match repl_line(&mut engine, "help mex") {
            CommandResult::Text(t) => {
                assert!(t.contains("mex"), "{t}");
                assert!(
                    t.contains("MEX-style"),
                    "expected host command help, got: {t}"
                );
            }
            other => panic!("expected Text, got {other:?}"),
        }
        // 非宿主命令仍回落到内核主题表
        match repl_line(&mut engine, "help fft") {
            CommandResult::Text(t) => assert!(t.to_lowercase().contains("fft"), "{t}"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn demo_reports_have_expected_shape() {
        let (_guard, mut engine) = locked_engine();
        let demos = run_all_demos(&mut engine).expect("demos");
        assert_eq!(demos.len(), 10, "all 10 demos must be present");
        // 一次 run_all_demos 同时校验：零错误、OK 标记、图窗/变量载荷。
        for (name, report) in &demos {
            assert!(
                report.error.is_none(),
                "demo {name} failed: {:?}",
                report.error.as_ref().map(|e| &e.message)
            );
            if name.ends_with("_toolbox") || name == "optim_demo" || name == "control_demo" {
                let text = report.console_text();
                assert!(
                    text.contains("demo OK"),
                    "v0.2 demo {name} must finish with its OK marker: {text}"
                );
            }
        }
        let plot = demos
            .iter()
            .find(|(n, _)| n == "plot_demo")
            .map(|(_, r)| r)
            .expect("plot_demo");
        assert!(
            !plot.figures.is_empty(),
            "plot_demo must export at least one figure"
        );
        // FigureScene 实测 schema（export_figure_scene 的 replay 载荷）：
        // envelope { createdAt, figure: { layout, metadata, plots } }；
        // layout/metadata 为 camelCase（axesRows/axesMetadata/title/xLabel），
        // plots 为内部标记 + snake_case：{ kind: "line", x, y, color_rgba,
        // line_width, line_style: "Solid", axes_index, label, visible }
        // 注：内核 figure 状态是进程级全局，并行测试可能污染 handle 集合，
        // 因此断言遍历全部导出图、不依赖顺序。
        let mut kinds = Vec::new();
        let mut has_subplot = false;
        for f in &plot.figures {
            let figure = f.scene_json.get("figure").unwrap_or(&f.scene_json);
            let plots = figure
                .get("plots")
                .and_then(|p| p.as_array())
                .expect("scene.figure.plots array");
            for p in plots {
                let obj = p.as_object().expect("plot is an object");
                assert!(
                    obj.get("kind").and_then(|k| k.as_str()).is_some(),
                    "plot must carry a `kind` tag, got keys: {:?}",
                    obj.keys().collect::<Vec<_>>()
                );
                // 冗余字段必须已被 tl-kernel 剥离（否则 IPC payload 翻倍）
                assert!(
                    !obj.keys().any(|k| k.ends_with("_source")),
                    "plot payload must not carry *_source duplicates, got keys: {:?}",
                    obj.keys().collect::<Vec<_>>()
                );
                kinds.push(obj["kind"].as_str().unwrap_or_default().to_string());
            }
            let rows = figure
                .pointer("/layout/axesRows")
                .and_then(|v| v.as_u64())
                .unwrap_or(1);
            let axes_meta = figure
                .pointer("/metadata/axesMetadata")
                .and_then(|m| m.as_array())
                .map(|m| m.len())
                .unwrap_or(0);
            if rows >= 2 || axes_meta >= 2 {
                has_subplot = true;
            }
        }
        assert!(!kinds.is_empty(), "plot_demo must export plot data");
        assert!(
            kinds.iter().any(|k| k == "line"),
            "plot_demo should contain line plots, got {kinds:?}"
        );
        assert!(has_subplot, "subplot(2,1,*) figure should be exported");
        let fft = demos
            .iter()
            .find(|(n, _)| n == "fft_demo")
            .map(|(_, r)| r)
            .expect("fft_demo");
        assert!(!fft.variables.is_empty(), "fft_demo must leave variables");
    }

    #[test]
    fn figure_payload_serializes_camel_case() {
        // 前端读 f.sceneJson：serde 必须输出 camelCase
        let payload = tl_types::FigurePayload {
            handle: 1,
            scene_json: serde_json::json!({ "plots": [] }),
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(json.contains("\"sceneJson\""), "got: {json}");
        assert!(json.contains("\"handle\""));
    }

    #[test]
    fn variable_preview_is_populated() {
        let (_guard, mut engine) = locked_engine();
        let report = engine.run("pv", "a = 1:5; c = 42; s = 'hello';");
        assert!(report.error.is_none());

        // 行向量：方括号 + 数字（内核预览最多 16 个元素）
        let a = report.variables.iter().find(|v| v.name == "a").expect("a");
        assert_eq!(a.shape, vec![1, 5]);
        let nums: Vec<f64> = a
            .preview
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split_whitespace()
            .map(|t| t.parse::<f64>().expect("numeric token"))
            .collect();
        assert_eq!(
            nums,
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            "preview: {}",
            a.preview
        );

        // 标量：不加方括号
        let c = report.variables.iter().find(|v| v.name == "c").expect("c");
        assert!(!c.preview.starts_with('['), "scalar preview: {}", c.preview);
        assert_eq!(c.preview.parse::<f64>().expect("scalar"), 42.0);

        // 非数值类型：内核不产出数值预览 → 面板显示 "—"
        let s = report.variables.iter().find(|v| v.name == "s").expect("s");
        assert!(
            s.preview.is_empty(),
            "char has no numeric preview, got {:?}",
            s.preview
        );
    }

    #[test]
    fn host_commands_dispatch() {
        let (_guard, mut engine) = locked_engine();
        engine.run("setup", "a = 1; b = magic(3);");
        let vars = host_command(&mut engine, "vars", &[]).expect("vars cmd");
        if let CommandResult::Text(t) = vars {
            assert!(t.contains("b"), "vars must list b");
        } else {
            panic!("vars must return Text");
        }
        host_command(&mut engine, "clear", &[]).expect("clear cmd");
        let vars = host_command(&mut engine, "vars", &[]).expect("vars cmd");
        if let CommandResult::Text(t) = vars {
            assert!(t.to_lowercase().contains("no variables"));
        } else {
            panic!("vars must return Text");
        }
    }

    #[test]
    fn repl_routes_host_and_m() {
        let (_guard, mut engine) = locked_engine();
        // m 代码
        match repl_line(&mut engine, "x = 1 + 1") {
            CommandResult::Report(r) => {
                assert!(r.error.is_none());
                assert!(!r.variables.is_empty());
            }
            other => panic!("expected Report, got {other:?}"),
        }
        // 宿主命令
        match repl_line(&mut engine, "vars") {
            CommandResult::Text(t) => assert!(t.contains("x")),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn help_topics_intercepted_at_host_layer() {
        let (_guard, mut engine) = locked_engine();
        // help('fft') / help fft 都走宿主层文档表（内核无 help 内置函数）
        for line in ["help('fft')", "help fft"] {
            match repl_line(&mut engine, line) {
                CommandResult::Text(t) => {
                    assert!(t.to_lowercase().contains("fft"), "line: {line}");
                }
                other => panic!("expected Text for {line}, got {other:?}"),
            }
        }
        // 裸 help 列宿主命令
        match repl_line(&mut engine, "help") {
            CommandResult::Text(t) => assert!(t.contains("host commands")),
            other => panic!("expected Text, got {other:?}"),
        }
    }
}
