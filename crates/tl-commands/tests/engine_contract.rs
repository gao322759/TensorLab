//! 跨层集成测试（L0-L8 贯通）：只使用 `tl-commands` 的公开 API，
//! 验证「内核执行 → RunReport → serde JSON」这条端到端契约。
//!
//! 与 `src/lib.rs` 里的单元测试互补：这里不碰任何私有实现，
//! 站在调用方（CLI / 桌面端 / MCP）视角断言可观测行为。

use tl_commands::{host_command, repl_line, run_all_demos, CommandResult, Engine};

fn engine() -> Engine {
    Engine::new().expect("engine init")
}

// ---------------- RunReport JSON 契约 ----------------

#[test]
fn run_report_json_keys_are_camel_case() {
    let mut e = engine();
    let report = e.run("t", "x = 1:3;");
    let v = serde_json::to_value(&report).expect("serialize RunReport");

    for key in [
        "events",
        "resultText",
        "error",
        "warnings",
        "variables",
        "figures",
        "durationMs",
        "usedJit",
    ] {
        assert!(v.get(key).is_some(), "RunReport missing key `{key}`: {v}");
    }
}

#[test]
fn variable_entries_expose_camel_case_fields() {
    let mut e = engine();
    let report = e.run("t", "x = 1:3;");
    let v = serde_json::to_value(&report).unwrap();
    let vars = v["variables"].as_array().expect("variables array");
    let x = vars
        .iter()
        .find(|it| it["name"] == "x")
        .expect("x must be listed");

    assert_eq!(x["className"], "double", "field must be camelCase: {x}");
    assert_eq!(x["shape"], serde_json::json!([1, 3]));
    assert!(x["isGpu"].is_boolean());
    assert!(x["preview"].is_string());
    assert!(x.get("class_name").is_none(), "snake_case must not leak");
}

#[test]
fn event_kinds_use_lowercase_camel_tags() {
    let mut e = engine();
    let report = e.run("t", "disp('hello from disp');");
    let v = serde_json::to_value(&report).unwrap();
    let kinds: Vec<String> = v["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|ev| ev["kind"].as_str().unwrap_or("<none>").to_string())
        .collect();

    assert!(
        kinds.iter().any(|k| k == "output"),
        "disp output must be an `output` event, got {kinds:?}"
    );
    for k in &kinds {
        assert!(
            matches!(k.as_str(), "output" | "error" | "clearConsole"),
            "unexpected event kind `{k}`"
        );
    }
}

/// 回归：fprintf 等控制台输出必须真的能取到（曾因 tag 命名不一致全部丢失）
#[test]
fn console_output_reaches_the_report() {
    let mut e = engine();
    let report = e.run("t", "fprintf('peak = %d\\n', 7);");
    let text = report.console_text();
    assert!(text.contains("peak = 7"), "console_text: {text:?}");
}

/// 回归：一条语句的回显只能出现一次。
///
/// 内核已把格式化好的回显写进 console 缓冲（→ streams），宿主的 `display_events`
/// 只携带匿名结果值；曾因两者都渲染，导致 `x = 42` 后多出一行 `42`、
/// `ans = 2` 后多出一行 `2`。
#[test]
fn statement_echo_is_not_duplicated() {
    fn lines(s: &str) -> Vec<String> {
        s.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect()
    }

    let mut e = engine();

    // 无分号赋值：仅一行 `x = 42`
    assert_eq!(lines(&e.run("a", "x = 42").console_text()), vec!["x = 42"]);
    // 表达式语句：仅一行 `ans = 2`
    assert_eq!(lines(&e.run("b", "1 + 1").console_text()), vec!["ans = 2"]);
    // 分号抑制：无输出
    assert!(
        lines(&e.run("c", "y = 7;").console_text()).is_empty(),
        "semicolon must suppress the echo"
    );
    // disp 走 stdout 路径，不受影响
    assert_eq!(lines(&e.run("d", "disp(99)").console_text()), vec!["99"]);
}

#[test]
fn clc_emits_clear_console_event() {
    let mut e = engine();
    let report = e.run("t", "disp('x'); clc;");
    let v = serde_json::to_value(&report).unwrap();
    let kinds: Vec<String> = v["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|ev| ev["kind"].as_str().unwrap_or("<none>").to_string())
        .collect();
    assert!(kinds.iter().any(|k| k == "clearConsole"), "{kinds:?}");
}

// ---------------- 诊断 ----------------

#[test]
fn error_diagnostics_carry_message_severity_and_line() {
    let mut e = engine();
    let report = e.run("t", "a = 1;\nthis_is_not_defined_xyz();\n");
    let err = report.error.as_ref().expect("expected an error");
    assert!(!err.message.is_empty(), "error message must not be empty");
    assert_eq!(err.severity, "error");
    assert!(err.line.is_some(), "error should be located to a line");
}

#[test]
fn successful_run_has_no_error_and_reports_timing() {
    let mut e = engine();
    let report = e.run("t", "y = 2 + 2;");
    assert!(report.error.is_none(), "{:?}", report.error);
    assert!(report.duration_ms >= 0.0);
}

// ---------------- 图窗载荷 ----------------

#[test]
fn figure_payload_is_envelope_and_dedup_free() {
    let mut e = engine();
    let report = e.run("t", "figure; plot(1:10);");
    assert!(!report.figures.is_empty(), "plot must export a figure");

    let v = serde_json::to_value(&report).unwrap();
    let fig = &v["figures"][0];
    assert!(fig["handle"].is_number(), "{fig}");
    let scene = &fig["sceneJson"];
    assert!(scene.is_object(), "sceneJson must be an object: {fig}");

    // envelope: { createdAt, figure: { layout, metadata, plots } }
    let figure = scene.get("figure").unwrap_or(scene);
    let plots = figure["plots"].as_array().expect("plots array");
    assert!(!plots.is_empty());
    for p in plots {
        assert!(
            !p.as_object()
                .unwrap()
                .keys()
                .any(|k| k.ends_with("_source")),
            "`*_source` duplicates must be stripped: {p}"
        );
    }
}

// ---------------- 宿主命令 / REPL 路由 ----------------

#[test]
fn host_command_dispatch_known_and_unknown() {
    let mut e = engine();
    assert!(
        host_command(&mut e, "__no_such_command__", &[]).is_none(),
        "unknown host command must return None"
    );
    match host_command(&mut e, "version", &[]) {
        Some(CommandResult::Text(t)) => {
            assert!(t.contains("TensorLab"), "{t}");
            assert!(t.contains("RunMat"), "{t}");
        }
        other => panic!("expected Text from version, got {other:?}"),
    }
}

#[test]
fn repl_routes_host_commands_and_m_code() {
    let mut e = engine();

    match repl_line(&mut e, "vars") {
        CommandResult::Text(t) => assert!(t.to_lowercase().contains("no variables"), "{t}"),
        other => panic!("expected Text, got {other:?}"),
    }

    match repl_line(&mut e, "z = 6 * 7;") {
        CommandResult::Report(r) => assert!(r.error.is_none()),
        other => panic!("expected Report, got {other:?}"),
    }

    match repl_line(&mut e, "vars") {
        CommandResult::Text(t) => assert!(t.contains('z'), "{t}"),
        other => panic!("expected Text, got {other:?}"),
    }

    // 空行 → None
    assert!(matches!(repl_line(&mut e, "   "), CommandResult::None));
}

#[test]
fn help_topics_are_intercepted_by_the_host_layer() {
    let mut e = engine();
    for line in ["help('fft')", "help fft"] {
        match repl_line(&mut e, line) {
            CommandResult::Text(t) => assert!(t.to_lowercase().contains("fft"), "{line}: {t}"),
            other => panic!("expected Text for `{line}`, got {other:?}"),
        }
    }
}

// ---------------- 文件执行 ----------------

#[test]
fn run_file_executes_from_disk() {
    let dir = std::env::temp_dir().join("tensorlab-integration-tests");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("contract_ok.m");
    std::fs::write(&path, "q = 3^3;\nfprintf('q=%d\\n', q);\n").expect("write script");

    let mut e = engine();
    let report = tl_commands::run_file(&mut e, path.to_str().unwrap()).expect("run_file");
    assert!(report.error.is_none(), "{:?}", report.error);
    assert!(report.console_text().contains("q=27"));
}

#[test]
fn run_file_loads_sibling_classdef() {
    let dir = std::env::temp_dir().join("tensorlab-classdef-file");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(
        dir.join("Vec2.m"),
        "classdef Vec2\n\
         properties\n\
           x\n\
           y\n\
         end\n\
         methods\n\
           function obj = Vec2(x, y)\n\
             obj.x = x;\n\
             obj.y = y;\n\
           end\n\
           function m = magnitude(obj)\n\
             m = sqrt(obj.x * obj.x + obj.y * obj.y);\n\
           end\n\
         end\n\
         end\n",
    )
    .expect("write class");
    let path = dir.join("classdef_main.m");
    std::fs::write(
        &path,
        "p = Vec2(3, 4);\nm = p.magnitude();\nfprintf('%s %g\\n', class(p), m);\n",
    )
    .expect("write script");

    let mut e = engine();
    let report = tl_commands::run_file(&mut e, path.to_str().unwrap()).expect("run_file");
    assert!(
        report.error.is_none(),
        "classdef file failed: {:?}",
        report.error.as_ref().map(|e| &e.message)
    );
    assert!(
        report.console_text().contains("Vec2 5"),
        "console: {}",
        report.console_text()
    );
}

#[test]
fn run_file_on_missing_path_is_an_error() {
    let mut e = engine();
    assert!(tl_commands::run_file(&mut e, "no/such/file_xyz.m").is_err());
}

// ---------------- demo 验收（demos are the spec） ----------------

#[test]
fn all_demos_run_without_errors() {
    // The kernel needs a deep stack; run the demo sweep on a 64 MiB thread
    // (same reason the three apps do, see kernel-notes pitfall 7).
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let mut e = engine();
            let demos = run_all_demos(&mut e).expect("demos must be readable");
            assert_eq!(demos.len(), 10, "v0.2 验收要求 10 个 demo");
            for (name, report) in &demos {
                assert!(
                    report.error.is_none(),
                    "demo `{name}` failed: {:?}",
                    report.error.as_ref().map(|e| &e.message)
                );
                match name.as_str() {
                    "optim_demo" => {
                        assert_golden_lines(
                            name,
                            report,
                            &[
                                "optimget TolX = 1.00e-8",
                                "fminsearch x = 1.000 -2.000 f = 0.0000",
                                "fmincon x = 0.500 0.500 flag = 1",
                                "lsqlin x = 0.500 1.500 resnorm = 1.500",
                                "lsqnonneg x = 0.000 1.500",
                                "intlinprog x = 4",
                                "f = -12.0",
                                "patternsearch x = 1.000 2.000",
                                "optimization toolbox demo OK",
                            ],
                        );
                        // 随机启发式求解器：用 0.05 容差断言关键解，
                        // 不把断言绑死在默认迭代数/RNG 的末位打印上。
                        let text = report.console_text();
                        assert_optimizer_line(&text, "ga x =", [1.0, 2.0], 0.05);
                        assert_optimizer_line(&text, "simulannealbnd x =", [1.0, 2.0], 0.05);
                        assert_optimizer_line(&text, "particleswarm x =", [1.0, 2.0], 0.05);
                    }
                    "control_demo" => assert_golden_lines(
                        name,
                        report,
                        &[
                            "tfdata den = 1.0 2.0 1.0",
                            "bode points = 200",
                            // 重复极点的已知限制：margin 当前报 0°（不是 Inf）。
                            "margin Pm = 0.00 deg",
                            "c2d den = 1.000 -1.810 0.819",
                            "c2d zoh order = 3",
                            "minreal order = 1",
                            "ssdata A11 = 0.0 D = 0.0",
                            "ss2zp zeros = 0 gain = 1.000",
                            "zp2ss A = -1.00 B = 1.00 C = 2.00",
                            "dlqr K = 0.434 1.028",
                            // 2×2 demo 系统；0.2929/0.4142 是 1×1 解析解，
                            // 由 tests/adversarial.rs 的 kalman 用例覆盖。
                            "kalman L = 0.4019 P = 0.6720",
                            "nichols points = 300",
                            "step final = 0.960",
                            "control toolbox demo OK",
                        ],
                    ),
                    _ => {}
                }
            }
        })
        .expect("spawn demo thread")
        .join()
        .expect("demo sweep panicked");
}

/// Golden 快照：demo 的关键打印行必须逐字出现。
///
/// 这些数值是 v0.2 验收基线（docs/releases/v0.2.md）。内核或工具箱的数值变化
/// 会让测试失败，迫使提交者显式确认这是预期变更并更新基线。
fn assert_golden_lines(demo: &str, report: &tl_commands::RunReport, expected: &[&str]) {
    let text = report.console_text();
    for line in expected {
        assert!(
            text.contains(line),
            "demo `{demo}` golden mismatch: missing {line:?}\nfull console:\n{text}"
        );
    }
}

/// 随机启发式求解器的容差快照：解析打印行里的数值，而不是匹配末位字符。
fn assert_optimizer_line(text: &str, prefix: &str, expected: [f64; 2], tol: f64) {
    let line = text
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("missing line starting with {prefix:?}\nconsole:\n{text}"));
    let values: Vec<f64> = line
        .split_whitespace()
        .filter_map(|token| token.parse::<f64>().ok())
        .collect();
    assert!(
        values.len() >= 2,
        "cannot parse optimizer values from {line:?}"
    );
    assert!(
        (values[0] - expected[0]).abs() <= tol && (values[1] - expected[1]).abs() <= tol,
        "{line:?} outside {expected:?} ± {tol}"
    );
}

#[test]
fn demo_declaration_matches_the_demos_directory() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    for (name, rel) in tl_commands::DEMOS {
        assert!(
            root.join(rel).exists(),
            "DEMOS declares `{name}` -> {rel}, but the file is missing"
        );
    }
}
