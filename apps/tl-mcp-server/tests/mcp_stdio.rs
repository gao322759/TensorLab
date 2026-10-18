//! tl-mcp-server 黑盒测试：以独立进程启动 MCP 服务器，
//! 通过 stdin 发送 JSON-RPC 2.0 请求、从 stdout 读取响应。
//!
//! 覆盖 MCP 2025-06-18 握手、tools/list、tools/call（run/eval/plot/vars/reset）
//! 以及协议健壮性（坏 JSON、通知不回包）。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn server_bin() -> PathBuf {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_tl-mcp-server") {
        return PathBuf::from(p);
    }
    // 回退：<target>/<profile>/deps/<test>.exe → <target>/<profile>/tl-mcp-server.exe
    let exe = std::env::current_exe().expect("current exe");
    let dir = exe
        .parent()
        .and_then(|p| p.parent())
        .expect("target profile dir");
    dir.join(format!("tl-mcp-server{}", std::env::consts::EXE_SUFFIX))
}

/// 启动一次服务器会话：写入全部请求 → 关闭 stdin → 收集所有响应行
fn session(requests: &[String]) -> Vec<serde_json::Value> {
    let mut child = Command::new(server_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mcp server");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        for r in requests {
            writeln!(stdin, "{r}").expect("write request");
        }
    }

    let out = child.wait_with_output().expect("wait for server");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("non-JSON line {l:?}: {e}")))
        .collect()
}

fn req(id: u32, method: &str) -> String {
    format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"{method}"}}"#)
}

fn call(id: u32, tool: &str, args_json: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{tool}","arguments":{args_json}}}}}"#
    )
}

/// 取 tools/call 响应里的文本内容
fn tool_text(resp: &serde_json::Value) -> String {
    resp["result"]
        .as_array()
        .unwrap_or_else(|| panic!("expected content array: {resp}"))
        .first()
        .and_then(|c| c["text"].as_str())
        .unwrap_or_else(|| panic!("expected text content: {resp}"))
        .to_string()
}

fn demo_path(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("demos")
        .join(name)
        .to_string_lossy()
        // JSON 里用正斜杠，免去反斜杠转义
        .replace('\\', "/")
}

// ---------------- 握手 ----------------

#[test]
fn initialize_handshake_returns_protocol_and_server_info() {
    let resps =
        session(&[r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.to_string()]);
    assert_eq!(resps.len(), 1, "{resps:?}");
    let r = &resps[0];
    assert_eq!(r["jsonrpc"], "2.0");
    assert_eq!(r["id"], 1);
    assert_eq!(r["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(r["result"]["serverInfo"]["name"], "tensorlab");
    assert!(r["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn notification_does_not_produce_a_response() {
    let resps = session(&[
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#.to_string(),
        req(11, "tools/list"),
    ]);
    assert_eq!(resps.len(), 1, "JSON-RPC 通知不得回包，实际收到 {resps:?}");
    assert_eq!(resps[0]["id"], 11, "唯一响应应对应 tools/list");
}

// ---------------- tools/list ----------------

#[test]
fn tools_list_exposes_all_five_tools_with_schemas() {
    let resps = session(&[req(2, "tools/list")]);
    let tools = resps[0]["result"]["tools"]
        .as_array()
        .expect("tools array")
        .clone();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for want in ["run", "eval", "plot", "vars", "reset"] {
        assert!(names.contains(&want), "missing tool `{want}`: {names:?}");
    }
    for t in &tools {
        assert!(
            t["description"]
                .as_str()
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "tool {t} must have a description"
        );
        assert!(
            t["inputSchema"]["type"] == "object",
            "tool {t} must declare an object inputSchema"
        );
    }
}

// ---------------- tools/call ----------------

#[test]
fn eval_tool_returns_the_result_text() {
    let resps = session(&[call(3, "eval", r#"{"code":"2 + 3*4"}"#)]);
    let text = tool_text(&resps[0]);
    assert!(text.contains("14"), "eval text: {text:?}");
}

#[test]
fn eval_tool_surfaces_errors_as_text() {
    let resps = session(&[call(4, "eval", r#"{"code":"not_defined_xyz()"}"#)]);
    let text = tool_text(&resps[0]);
    assert!(text.contains("ERROR"), "expected ERROR marker: {text:?}");
}

#[test]
fn vars_tool_returns_json_and_reset_clears_it() {
    let resps = session(&[
        call(1, "eval", r#"{"code":"zz = 5;"}"#),
        call(2, "vars", "{}"),
        call(3, "reset", "{}"),
        call(4, "vars", "{}"),
    ]);

    let after_set = tool_text(&resps[1]);
    let parsed: serde_json::Value =
        serde_json::from_str(&after_set).expect("vars must return valid JSON");
    assert!(parsed.is_array(), "{after_set}");
    assert!(after_set.contains("zz"), "zz should be listed: {after_set}");

    assert!(
        tool_text(&resps[2]).contains("reset"),
        "reset should acknowledge"
    );

    let after_reset = tool_text(&resps[3]);
    assert!(
        !after_reset.contains("zz"),
        "reset must clear the workspace: {after_reset}"
    );
}

#[test]
fn plot_tool_returns_figure_scene_json() {
    let resps = session(&[call(5, "plot", r#"{"code":"figure; plot(1:5);"}"#)]);
    let text = tool_text(&resps[0]);
    assert!(text.contains("figures:"), "{text}");

    let json_part = text
        .split("figures:")
        .nth(1)
        .expect("figures section")
        .trim();
    let arr: serde_json::Value =
        serde_json::from_str(json_part).expect("figure scene must be valid JSON");
    let arr = arr.as_array().expect("scene array");
    assert!(
        !arr.is_empty(),
        "plot must export at least one figure: {text}"
    );
    assert!(
        arr[0].get("figure").is_some() || arr[0].get("plots").is_some(),
        "scene should be a FigureScene envelope or body: {}",
        arr[0]
    );
}

#[test]
fn run_tool_reads_a_script_from_disk() {
    let req_json = call(
        7,
        "run",
        &format!(r#"{{"path":"{}"}}"#, demo_path("hello_world.m")),
    );
    let resps = session(&[req_json]);
    let text = tool_text(&resps[0]);
    assert!(text.contains("Hello from TensorLab!"), "{text}");
}

#[test]
fn run_tool_missing_file_reports_error_text() {
    let req_json = call(8, "run", r#"{"path":"no/such/file_xyz.m"}"#);
    let resps = session(&[req_json]);
    let text = tool_text(&resps[0]);
    assert!(text.starts_with("error:"), "expected error text: {text:?}");
}

#[test]
fn unknown_tool_reports_error_text() {
    let resps = session(&[call(9, "no_such_tool", "{}")]);
    let text = tool_text(&resps[0]);
    assert!(text.contains("unknown tool"), "{text:?}");
}

// ---------------- 协议健壮性 ----------------

#[test]
fn malformed_json_line_is_ignored_and_session_survives() {
    let resps = session(&[
        "this is not json".to_string(),
        "{also: not, json}".to_string(),
        req(12, "tools/list"),
    ]);
    assert_eq!(resps.len(), 1, "坏 JSON 不应产生响应: {resps:?}");
    assert_eq!(resps[0]["id"], 12);
    assert!(resps[0]["result"]["tools"].is_array());
}

#[test]
fn unknown_method_returns_a_response_without_crashing() {
    let resps = session(&[req(13, "no/such/method")]);
    assert_eq!(resps.len(), 1);
    assert_eq!(resps[0]["jsonrpc"], "2.0");
    // 方法未知 → result 为 null（当前实现），关键是进程不崩且给出响应
    assert!(resps[0].get("result").is_some(), "{:?}", resps[0]);
}

#[test]
fn blank_lines_are_ignored() {
    let resps = session(&["".to_string(), "   ".to_string(), req(14, "tools/list")]);
    assert_eq!(resps.len(), 1);
    assert_eq!(resps[0]["id"], 14);
}

#[test]
fn multiple_requests_are_answered_in_order_with_matching_ids() {
    let resps = session(&[req(20, "tools/list"), req(21, "tools/list")]);
    assert_eq!(resps.len(), 2);
    assert_eq!(resps[0]["id"], 20);
    assert_eq!(resps[1]["id"], 21);
}
