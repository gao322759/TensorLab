//! TensorLab MCP stdio 服务器。
//!
//! JSON-RPC 2.0 over stdio（MCP 2025-06-18 协议）。
//! 工具：run / eval / plot / vars / reset。

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tl_commands::Engine;

const PROTOCOL_VERSION: &str = "2025-06-18";

fn main() -> anyhow::Result<()> {
    // RunMat 内核深递归需要大栈：在 64 MiB 栈的工作线程上运行
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(serve)
        .expect("spawn engine thread")
        .join()
        .map_err(|_| anyhow::anyhow!("engine thread panicked"))?
}

fn serve() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut engine = Engine::new()?;

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => {
                flush_error(&stdout, Value::Null, -32700, "Parse error")?;
                continue;
            }
        };
        let id = req.get("id").cloned();
        let Some(method) = req.get("method").and_then(Value::as_str) else {
            if let Some(id) = id {
                flush_error(&stdout, id, -32600, "Invalid Request")?;
            }
            continue;
        };
        // JSON-RPC 2.0：通知（无 id）不得产生响应。
        let Some(id) = id else {
            continue;
        };
        let params = req.get("params").cloned().unwrap_or(json!({}));

        let result: Result<Value, (i64, String)> = match method {
            "initialize" => Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "tensorlab", "version": env!("CARGO_PKG_VERSION") }
            })),
            "tools/list" => Ok(json!({
                "tools": [
                    { "name": "run", "description": "Run a .m script from disk", "inputSchema": {
                        "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] } },
                    { "name": "eval", "description": "Evaluate MATLAB code", "inputSchema": {
                        "type": "object", "properties": { "code": { "type": "string" } }, "required": ["code"] } },
                    { "name": "plot", "description": "Evaluate code and return figure data (FigureScene JSON)", "inputSchema": {
                        "type": "object", "properties": { "code": { "type": "string" } }, "required": ["code"] } },
                    { "name": "vars", "description": "List workspace variables", "inputSchema": { "type": "object", "properties": {} } },
                    { "name": "reset", "description": "Clear the workspace and figures", "inputSchema": { "type": "object", "properties": {} } }
                ]
            })),
            "tools/call" => {
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                Ok(match name {
                    "run" => {
                        let path = args.get("path").and_then(Value::as_str).unwrap_or("");
                        match run_path(&mut engine, path) {
                            Ok(text) => tool_text(text),
                            Err(e) => tool_err(e.to_string()),
                        }
                    }
                    "eval" => {
                        let code = args.get("code").and_then(Value::as_str).unwrap_or("");
                        let report = engine.run("mcp_eval", code);
                        tool_text(report_text(&report))
                    }
                    "plot" => {
                        let code = args.get("code").and_then(Value::as_str).unwrap_or("");
                        let report = engine.run("mcp_plot", code);
                        let figures: Vec<Value> = report
                            .figures
                            .iter()
                            .map(|f| f.scene_json.clone())
                            .collect();
                        tool_text(format!(
                            "{}\nfigures: {}",
                            report_text(&report),
                            serde_json::to_string(&figures)?
                        ))
                    }
                    "vars" => {
                        let vars = engine.variables();
                        tool_text(serde_json::to_string_pretty(&vars)?)
                    }
                    "reset" => {
                        engine.run("mcp_reset", "close all; clear;");
                        engine.clear();
                        tool_text("workspace reset.".to_string())
                    }
                    other => tool_err(format!("unknown tool: {other}")),
                })
            }
            other => Err((-32601, format!("Method not found: {other}"))),
        };

        match result {
            Ok(value) => flush_result(&stdout, id, value)?,
            Err((code, message)) => flush_error(&stdout, id, code, &message)?,
        }
    }
    Ok(())
}

fn run_path(engine: &mut Engine, path: &str) -> anyhow::Result<String> {
    let src = std::fs::read_to_string(path)?;
    let report = engine.run_path(path, &src);
    Ok(report_text(&report))
}

fn report_text(report: &tl_types::RunReport) -> String {
    let mut out = report.console_text();
    if let Some(err) = &report.error {
        out.push_str(&format!("ERROR: {}\n", err.message));
    }
    for w in &report.warnings {
        out.push_str(&format!("WARNING: {}\n", w.message));
    }
    out
}

fn tool_text(text: String) -> Value {
    json!([{ "type": "text", "text": text }])
}

fn tool_err(msg: String) -> Value {
    json!([{ "type": "text", "text": format!("error: {msg}") }])
}

fn flush_result(mut out: &io::Stdout, id: Value, result: Value) -> anyhow::Result<()> {
    let obj = json!({ "jsonrpc": "2.0", "id": id, "result": result });
    serde_json::to_writer(&mut out, &obj)?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

fn flush_error(mut out: &io::Stdout, id: Value, code: i64, message: &str) -> anyhow::Result<()> {
    let obj = json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    });
    serde_json::to_writer(&mut out, &obj)?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}
