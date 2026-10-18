//! TensorLab 桌面端 — Tauri 2 应用。
//!
//! 架构：UI（HTML/JS）通过 `run`/`vars`/`stop` 命令与引擎线程通信。
//! Engine 独占一条 64 MiB 大栈工作线程（RunMat 内核 parser/eval 深递归
//! 需要大栈；主线程与 Tauri 线程池默认栈不够），Tauri 命令经 mpsc 转发。
//! 图窗由前端用内联 SVG 渲染器绘制 FigureScene（不依赖外部 CDN）。

use std::sync::mpsc::{self, Sender};
use tauri::State;
use tl_commands::{repl_line, CommandResult, Engine};
use tl_types::RunReport;

/// 发往引擎线程的请求（reply 用于同步取回结果）
enum EngineRequest {
    Run {
        source: String,
        reply: mpsc::Sender<RunReport>,
    },
    Vars {
        reply: mpsc::Sender<Vec<tl_types::VariableInfo>>,
    },
    Stop,
}

struct AppState {
    tx: Sender<EngineRequest>,
}

/// 启动引擎工作线程（64 MiB 栈）
fn spawn_engine_thread() -> Sender<EngineRequest> {
    let (tx, rx) = mpsc::channel::<EngineRequest>();
    std::thread::Builder::new()
        .name("tensorlab-engine".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let mut engine = Engine::new().expect("engine init");
            for req in rx {
                match req {
                    EngineRequest::Run { source, reply } => {
                        let report = match repl_line(&mut engine, &source) {
                            CommandResult::Report(r) => r,
                            CommandResult::Text(t) => {
                                let mut r = RunReport::default();
                                r.events
                                    .push(tl_types::StreamEntryPayload::Output { text: t });
                                r
                            }
                            CommandResult::None => RunReport::default(),
                        };
                        let _ = reply.send(report);
                    }
                    EngineRequest::Vars { reply } => {
                        let _ = reply.send(engine.variables());
                    }
                    // 协作式取消：内核在当前语句边界响应
                    EngineRequest::Stop => engine.cancel(),
                }
            }
        })
        .expect("spawn engine thread");
    tx
}

#[tauri::command]
fn run(state: State<AppState>, source: String) -> RunReport {
    let (reply_tx, reply_rx) = mpsc::channel();
    let _ = state.tx.send(EngineRequest::Run {
        source,
        reply: reply_tx,
    });
    reply_rx.recv().unwrap_or_default()
}

#[tauri::command]
fn stop(state: State<AppState>) {
    let _ = state.tx.send(EngineRequest::Stop);
}

#[tauri::command]
fn vars(state: State<AppState>) -> Vec<tl_types::VariableInfo> {
    let (reply_tx, reply_rx) = mpsc::channel();
    let _ = state.tx.send(EngineRequest::Vars { reply: reply_tx });
    reply_rx.recv().unwrap_or_default()
}

/// 应用信息（About 对话框）：版本以 Cargo 包版本为唯一来源。
#[derive(serde::Serialize)]
struct AppInfo {
    version: &'static str,
    kernel: &'static str,
    compat: &'static str,
    license: &'static str,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        kernel: "RunMat 0.6.2",
        compat: "MATLAB R2026a",
        license: "MIT",
    }
}

/// 读取 m 脚本（路径来自原生打开对话框，故直接 std::fs，不走 fs 插件权限）
#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))
}

/// 保存 m 脚本（路径来自原生保存对话框）
#[tauri::command]
fn write_text_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|e| format!("cannot write {path}: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn start_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            tx: spawn_engine_thread(),
        })
        .invoke_handler(tauri::generate_handler![
            run,
            stop,
            vars,
            app_info,
            read_text_file,
            write_text_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
