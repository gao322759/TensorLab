//! TensorLab 桌面端 — Tauri 2 应用。
//!
//! 架构：UI（HTML/JS）通过 `run`/`vars`/`stop` 命令与引擎线程通信。
//! Engine 独占一条 64 MiB 大栈工作线程（RunMat 内核 parser/eval 深递归
//! 需要大栈；主线程与 Tauri 线程池默认栈不够），Tauri 命令经 mpsc 转发。
//! 图窗由前端用内联 SVG 渲染器绘制 FigureScene（不依赖外部 CDN）。

use std::sync::mpsc::{self, Sender};
use tauri::{Emitter, State};
use tl_commands::{repl_line_at, repl_line_at_debug, CommandResult, Engine};
use tl_types::{DiagnosticPayload, RunReport, StreamEntryPayload};

/// 发往引擎线程的请求（reply 用于同步取回结果）
enum EngineRequest {
    Run {
        source: String,
        /// 编辑器当前文件的磁盘路径；有值时用同名 Text 请求加载同目录 classdef。
        path: Option<String>,
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
            let mut engine = match Engine::new() {
                Ok(engine) => engine,
                Err(err) => {
                    // 初始化失败时保留线程存活，把错误回传给每个请求，避免静默空白。
                    let message = format!("engine init failed: {err}");
                    for req in rx {
                        match req {
                            EngineRequest::Run { reply, .. } => {
                                let _ = reply.send(engine_error_report(&message));
                            }
                            EngineRequest::Vars { reply } => {
                                let _ = reply.send(Vec::new());
                            }
                            EngineRequest::Stop => {}
                        }
                    }
                    return;
                }
            };
            for req in rx {
                match req {
                    EngineRequest::Run {
                        source,
                        path,
                        reply,
                    } => {
                        // 存在断点时走插桩调试路径（run_debugged*），否则零开销普通执行
                        let debug = tl_commands::debug::has_breakpoints();
                        let result = if debug {
                            repl_line_at_debug(&mut engine, &source, path.as_deref())
                        } else {
                            repl_line_at(&mut engine, &source, path.as_deref())
                        };
                        let report = match result {
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

/// 执行编辑器缓冲。
///
/// 用 `async` + `spawn_blocking`：命令体要阻塞等引擎线程回包，若占用 Tauri 的
/// 同步命令线程，`debug_resume` / `stop` 可能排不到线程而看起来"死机"。
#[tauri::command]
async fn run(
    state: State<'_, AppState>,
    source: String,
    path: Option<String>,
) -> Result<RunReport, String> {
    // 调试暂停期间引擎线程被 __tl_debug__ 钩子占住，新请求会死等 —— 直接拒绝。
    if tl_commands::debug::is_paused() {
        return Ok(engine_error_report(
            "debugger is paused (dbcont/dbstep/dbquit to resume)",
        ));
    }
    let tx = state.tx.clone();
    let joined = tauri::async_runtime::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        let _ = tx.send(EngineRequest::Run {
            source,
            path,
            reply: reply_tx,
        });
        match reply_rx.recv() {
            Ok(report) => report,
            Err(_) => engine_error_report("engine thread stopped"),
        }
    })
    .await;
    Ok(joined.unwrap_or_else(|e| engine_error_report(&format!("run task failed: {e}"))))
}

/// 设置断点（编辑器 gutter 点击后整表同步）。
#[tauri::command]
fn debug_set_breakpoints(lines: Vec<u32>) {
    tl_commands::debug::set_breakpoints(lines);
}

/// 当前暂停行号（1-based）；未暂停为 `null`。
///
/// 前端在运行期间轮询本命令驱动调试 UI —— 不依赖事件通道是否可用，
/// 保证"引擎已暂停"一定被 UI 感知（否则用户只看到卡住的 Run）。
#[tauri::command]
fn debug_state() -> Option<u32> {
    tl_commands::debug::paused_line()
}

/// 调试恢复：continue（dbcont）/ step（dbstep）/ stop（dbquit）。
#[tauri::command]
fn debug_resume(mode: String) {
    let mode = match mode.as_str() {
        "step" => tl_commands::debug::ResumeMode::Step,
        "stop" => tl_commands::debug::ResumeMode::Stop,
        _ => tl_commands::debug::ResumeMode::Run,
    };
    tl_commands::debug::resume(mode);
}

#[tauri::command]
fn stop(state: State<AppState>) {
    // 调试暂停时引擎线程被调试钩子占住，Stop 必须走 dbquit 通道才生效
    if tl_commands::debug::is_paused() {
        tl_commands::debug::resume(tl_commands::debug::ResumeMode::Stop);
        return;
    }
    let _ = state.tx.send(EngineRequest::Stop);
}

#[tauri::command]
fn vars(state: State<AppState>) -> Vec<tl_types::VariableInfo> {
    let (reply_tx, reply_rx) = mpsc::channel();
    let _ = state.tx.send(EngineRequest::Vars { reply: reply_tx });
    match reply_rx.recv() {
        Ok(vars) => vars,
        Err(_) => {
            eprintln!("warning: engine thread stopped; variable list unavailable");
            Vec::new()
        }
    }
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

fn engine_error_report(message: &str) -> RunReport {
    RunReport {
        events: vec![StreamEntryPayload::Output {
            text: format!("engine error: {message}"),
        }],
        error: Some(DiagnosticPayload {
            identifier: Some("TensorLab:Engine".to_string()),
            message: message.to_string(),
            line: None,
            callstack: Vec::new(),
            severity: "error".to_string(),
        }),
        ..Default::default()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn start_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 调试暂停事件：内核 __tl_debug__ 钩子在引擎线程上通知，转发给前端
            let handle = app.handle().clone();
            tl_commands::debug::set_listener(std::sync::Arc::new(move |ev| {
                let _ = handle.emit("debug-paused", ev.line);
            }));
            Ok(())
        })
        .manage(AppState {
            tx: spawn_engine_thread(),
        })
        .invoke_handler(tauri::generate_handler![
            run,
            stop,
            vars,
            app_info,
            read_text_file,
            write_text_file,
            debug_set_breakpoints,
            debug_state,
            debug_resume
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
