//! 调试支持：断点 / 单步 / 停止（MATLAB dbstop/dbcont/dbstep/dbquit 语义）。
//!
//! RunMat 0.6.2 内核没有调试器 API（只有协作式取消中断标志）。本模块用
//! "源码插桩 + 阻塞式内置函数" 在宿主层实现语句级调试：
//!
//! 1. [`instrument_source`] 用词法扫描器在每个**语句行**行首注入
//!    `__tl_debug__(N);`（N 为 1-based 行号；同行插入保证行号不变）。
//!    注入点跳过：纯注释/空行、括号续行、`...` 续行、块注释、
//!    块结构关键字行（function/end/else/elseif/case/otherwise/catch/arguments）
//!    以及 classdef 文件与 arguments 块内部 —— 这些位置插入语句会破坏语法。
//! 2. `__tl_debug__` 是经 `inventory` 注册进内核 builtin 注册表的真实内置函数：
//!    命中断点或处于单步模式时，先通过 listener 通知宿主（`Paused{line}` 事件），
//!    再在 Condvar 上阻塞等待宿主 resume（continue/step/stop）。
//!    `stop` 会置内核中断标志并以错误终止执行（对齐 dbquit 终止语义）。
//!
//! 已知限制（v0.3）：
//! - 步进粒度是"语句行"（同行多条语句一次暂停，多行语句在其首行暂停），
//!   与 MATLAB F10 行级步进一致；`else/case/end` 等结构行不暂停。
//! - 暂停期间引擎线程被占住，`vars` 无法实时刷新（恢复/结束后更新）。

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use runmat_async::RuntimeError;
use runmat_value::Value;

// ---------------------------------------------------------------- 控制器

/// 宿主发给调试器的恢复命令（对应 dbcont / dbstep / dbquit）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeMode {
    /// 运行到下一个断点（或结束）
    Run,
    /// 单步：下一条语句再暂停
    Step,
    /// 终止执行
    Stop,
}

#[derive(Debug)]
struct Shared {
    breakpoints: BTreeSet<u32>,
    resume: ResumeMode,
    paused: bool,
    paused_line: u32,
    active: bool,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            breakpoints: BTreeSet::new(),
            resume: ResumeMode::Run,
            paused: false,
            paused_line: 0,
            active: false,
        }
    }
}

/// 暂停时发给宿主的事件（桌面端转发为 Tauri `debug-paused` 事件）。
#[derive(Debug, Clone, Copy)]
pub struct PausedEvent {
    pub line: u32,
}

type Listener = Arc<dyn Fn(PausedEvent) + Send + Sync>;

struct Controller {
    inner: Mutex<Shared>,
    cv: Condvar,
    listener: Mutex<Option<Listener>>,
    interrupt: Mutex<Option<Arc<AtomicBool>>>,
}

static CONTROLLER: OnceLock<Controller> = OnceLock::new();

fn controller() -> &'static Controller {
    CONTROLLER.get_or_init(|| Controller {
        inner: Mutex::new(Shared::default()),
        cv: Condvar::new(),
        listener: Mutex::new(None),
        interrupt: Mutex::new(None),
    })
}

/// 注册暂停事件监听者（桌面端在启动时调用）。
pub fn set_listener(l: Listener) {
    *controller().listener.lock().unwrap() = Some(l);
}

/// 注册内核中断标志（Engine 创建时调用；stop 时直接置位实现硬取消）。
pub fn set_interrupt_handle(flag: Arc<AtomicBool>) {
    *controller().interrupt.lock().unwrap() = Some(flag);
}

/// 当前断点集合（快照）。
pub fn breakpoints() -> BTreeSet<u32> {
    controller().inner.lock().unwrap().breakpoints.clone()
}

/// 是否存在断点（决定 run 是否走插桩路径）。
pub fn has_breakpoints() -> bool {
    !controller().inner.lock().unwrap().breakpoints.is_empty()
}

/// 是否处于暂停状态（宿主据此拒绝新的 run 请求，避免引擎死等）。
pub fn is_paused() -> bool {
    controller().inner.lock().unwrap().paused
}

/// 当前暂停行号（1-based）；未暂停返回 `None`。
///
/// 桌面端用轮询读取它驱动 UI —— 比事件监听更可靠（事件权限/时序失败时
/// 不会出现"引擎阻塞但 UI 不知情"的死锁观感）。
pub fn paused_line() -> Option<u32> {
    let g = controller().inner.lock().unwrap();
    if g.paused {
        Some(g.paused_line)
    } else {
        None
    }
}

/// 设置断点集合（整表替换；编辑器每次变更后同步）。
pub fn set_breakpoints(lines: Vec<u32>) {
    let mut g = controller().inner.lock().unwrap();
    g.breakpoints = lines.into_iter().collect();
}

/// 宿主恢复命令：continue / step / stop。
pub fn resume(mode: ResumeMode) {
    let g = &controller();
    let mut s = g.inner.lock().unwrap();
    s.resume = mode;
    s.paused = false;
    drop(s);
    g.cv.notify_all();
}

/// 一次插桩运行开始前重置状态（Engine::run_debugged* 调用）。
pub fn begin_run() {
    let mut g = controller().inner.lock().unwrap();
    g.resume = ResumeMode::Run;
    g.paused = false;
    g.active = true;
}

/// 运行结束后清理。
fn end_run() {
    let mut g = controller().inner.lock().unwrap();
    g.active = false;
    g.paused = false;
}

/// 运行结束后由 Engine 调用（见 [`crate::Engine::run_debugged`]）。
pub fn finish_run() {
    end_run();
}

fn notify_paused(line: u32) {
    let l = controller().listener.lock().unwrap().clone();
    if let Some(f) = l {
        f(PausedEvent { line });
    }
}

fn set_interrupt() {
    let flag = controller().interrupt.lock().unwrap().clone();
    if let Some(f) = flag {
        f.store(true, Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------- 内置函数

fn debug_call(line: u32) -> Result<Value, RuntimeError> {
    let ctl = controller();
    let mut g = ctl.inner.lock().unwrap();
    let pause = match g.resume {
        ResumeMode::Step => true,
        ResumeMode::Run => g.breakpoints.contains(&line),
        ResumeMode::Stop => false,
    };
    if !pause {
        return Ok(Value::Num(0.0));
    }
    g.paused = true;
    g.paused_line = line;
    drop(g); // 通知外部（Tauri emit）时不持锁，避免与 is_paused/paused_line 互等
    notify_paused(line);
    let mut g = ctl.inner.lock().unwrap();
    while g.paused {
        if g.resume == ResumeMode::Stop {
            drop(g);
            set_interrupt();
            return Err(RuntimeError::new(
                "Execution terminated by debugger (dbquit).",
            ));
        }
        g = ctl
            .cv
            .wait_timeout(g, std::time::Duration::from_millis(200))
            .unwrap()
            .0;
    }
    Ok(Value::Num(0.0))
}

fn tl_debug_impl(args: &[Value]) -> runmat_builtins::BuiltinFuture {
    let line = args
        .first()
        .map(|v| match v {
            Value::Num(n) => *n,
            Value::Int(i) => i.to_i64() as f64,
            _ => 0.0,
        })
        .unwrap_or(0.0) as u32;
    Box::pin(async move { debug_call(line) })
}

// 把 `__tl_debug__` 注册进内核 builtin 注册表。
//
// `inventory::submit!` 在静态初始化期收集（链接即注册，无需显式调用）。
// 用旧的 `BuiltinFunction` 直提路径（runmat-builtins 的 lookup 兼容它），
// 因为 `#[runtime_builtin]` 宏与 catalog 校验器属 runmat-runtime 内部设施。
inventory::submit!(runmat_builtins::BuiltinFunction::new(
    "__tl_debug__",
    "TensorLab debugger hook (internal)",
    "internal",
    "Internal statement-level debug hook. Not part of the m language surface.",
    "",
    vec![runmat_builtins::Type::Num],
    runmat_builtins::Type::Num,
    None,
    tl_debug_impl,
    &[],
    true,
    true,
));

// ---------------------------------------------------------------- 源码插桩

/// 结构性关键字：这些行首不能插入语句（会破坏块语法）。
const STRUCTURAL: &[&str] = &[
    "function",
    "end",
    "else",
    "elseif",
    "case",
    "otherwise",
    "catch",
    "arguments",
    "classdef",
    "properties",
    "methods",
    "events",
    "enumeration",
];

/// 扫描器跨行状态。
#[derive(Default)]
struct ScanState {
    in_block_comment: bool,
    bracket_depth: i32,
    continuation: bool,
    args_depth: i32,
}

/// 判断 `'` 是转置（true）还是字符串开始（false）。
/// 规则：前一个非空白字符是标识符字符/数字/闭括号/点/另一个引号 → 转置。
fn is_transpose(prev: Option<char>) -> bool {
    match prev {
        Some(c) => {
            c.is_alphanumeric()
                || c == '_'
                || c == ')'
                || c == ']'
                || c == '}'
                || c == '.'
                || c == '\''
                || c == '"'
        }
        None => false,
    }
}

/// 行首第一个代码 token（跳过空白；返回 None 表示空行/纯注释）。
/// 同时返回该行是否以 `...` 续行结尾（在字符串/注释外）。
fn scan_line(line: &str, st: &mut ScanState) -> LineInfo {
    let mut chars = line.char_indices().peekable();
    let mut prev: Option<char> = None;
    let mut in_string: Option<char> = None;
    let mut in_comment = false;
    let mut first_code: Option<usize> = None;
    let mut first_word = String::new();
    let mut word_done = false;
    let mut trailing_dot3 = false;
    let mut depth = st.bracket_depth;
    let mut args = st.args_depth > 0;

    while let Some((i, c)) = chars.next() {
        if let Some(q) = in_string {
            if c == '\\' && q == '"' {
                chars.next();
                continue;
            }
            if c == q {
                if q == '\'' && chars.peek().map(|(_, n)| *n) == Some('\'') {
                    // char 字面量内的转义引号 ''：跳过成对的两个引号
                    chars.next();
                    prev = Some(c);
                    continue;
                }
                in_string = None;
            }
            prev = Some(c);
            continue;
        }
        if in_comment {
            continue;
        }
        match c {
            '%' => {
                let rest: String = line[i + 1..].trim().to_string();
                if rest == "{" {
                    st.in_block_comment = true;
                    return LineInfo {
                        first_code: None,
                        first_word: String::new(),
                        trailing_dot3: false,
                        in_args: false,
                        depth,
                    };
                }
                in_comment = true;
            }
            '\'' => {
                if is_transpose(prev) {
                    prev = Some(c);
                } else {
                    in_string = Some('\'');
                    prev = Some(c);
                }
            }
            '"' => {
                in_string = Some('"');
                prev = Some(c);
            }
            '.' if chars.peek().map(|(_, n)| *n) == Some('.') => {
                // `...` 续行（其后都是注释）
                trailing_dot3 = true;
                break;
            }
            '(' | '[' | '{' => {
                depth += 1;
                if first_code.is_none() {
                    first_code = Some(i);
                }
                prev = Some(c);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                if first_code.is_none() {
                    first_code = Some(i);
                }
                prev = Some(c);
            }
            c if c.is_whitespace() => {
                if !first_word.is_empty() {
                    word_done = true;
                }
            }
            _ => {
                if first_code.is_none() {
                    first_code = Some(i);
                }
                if !word_done && first_word.len() < 12 {
                    if c.is_alphabetic()
                        || c == '_'
                        || (!first_word.is_empty() && c.is_alphanumeric())
                    {
                        first_word.push(c);
                    } else if !first_word.is_empty() {
                        word_done = true;
                    }
                }
                prev = Some(c);
            }
        }
    }

    // 块注释结束行（整行 %}）在下方由调用方以 in_block_comment 处理；
    // 这里如果以 %} 开头，first_code 也应为 None。
    if st.in_block_comment {
        let trimmed = line.trim();
        if trimmed.starts_with("%}") {
            st.in_block_comment = false;
        }
        return LineInfo {
            first_code: None,
            first_word: String::new(),
            trailing_dot3: false,
            in_args: false,
            depth,
        };
    }

    if first_word == "arguments" {
        st.args_depth += 1;
        args = true;
    } else if first_word == "end" && st.args_depth > 0 {
        st.args_depth -= 1;
        args = true;
    }

    LineInfo {
        first_code,
        first_word,
        trailing_dot3,
        in_args: args,
        depth,
    }
}

#[derive(Debug, Default)]
struct LineInfo {
    first_code: Option<usize>,
    first_word: String,
    trailing_dot3: bool,
    in_args: bool,
    depth: i32,
}

/// 在每个语句行行首注入 `__tl_debug__(N);`。
///
/// 行号保持不变（同行插入）。classdef 文件整体跳过。
pub fn instrument_source(src: &str) -> String {
    if src
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim_start().starts_with("classdef"))
        .unwrap_or(false)
    {
        return src.to_string();
    }
    let mut st = ScanState::default();
    let mut out = String::with_capacity(src.len() + src.lines().count() * 20);
    for (idx, line) in src.split_inclusive('\n').enumerate() {
        let lineno = idx + 1;
        let nl = line.ends_with('\n');
        let body = if nl { &line[..line.len() - 1] } else { line };
        let mut inject = false;

        if st.in_block_comment {
            // 整行属于块注释（结束行 %} 也在此处理）
            let trimmed = body.trim();
            if trimmed.starts_with("%}") {
                st.in_block_comment = false;
            }
            out.push_str(line);
            continue;
        }
        if st.continuation || st.bracket_depth > 0 || st.args_depth > 0 {
            // 续行 / 括号内换行 / arguments 块：不是语句起点
            let info = scan_line(body, &mut st);
            st.continuation = info.trailing_dot3;
            st.bracket_depth = info.depth;
            push_line(&mut out, body, nl, false, lineno);
            continue;
        }
        let info = scan_line(body, &mut st);
        st.continuation = info.trailing_dot3;
        st.bracket_depth = info.depth;
        if info.first_code.is_some()
            && !info.in_args
            && !STRUCTURAL.contains(&info.first_word.as_str())
        {
            inject = true;
        }
        push_line(&mut out, body, nl, inject, lineno);
    }
    out
}

fn push_line(out: &mut String, body: &str, nl: bool, inject: bool, lineno: usize) {
    if inject {
        let at = body
            .char_indices()
            .find(|(_, c)| !c.is_whitespace())
            .map(|(i, _)| i)
            .unwrap_or(0);
        out.push_str(&body[..at]);
        out.push_str(&format!("__tl_debug__({lineno});"));
        out.push_str(&body[at..]);
    } else {
        out.push_str(body);
    }
    if nl {
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruments_plain_statements_with_line_numbers() {
        let src = "a = 1;\nb = 2\nc = a + b;";
        let out = instrument_source(src);
        assert_eq!(
            out,
            "__tl_debug__(1);a = 1;\n__tl_debug__(2);b = 2\n__tl_debug__(3);c = a + b;"
        );
    }

    #[test]
    fn skips_comments_blanks_and_strings_with_semicolons() {
        let src = "% leading comment\n\ns = 'a; b = 2';\n% another\nx = 1;";
        let out = instrument_source(src);
        assert_eq!(
            out,
            "% leading comment\n\n__tl_debug__(3);s = 'a; b = 2';\n% another\n__tl_debug__(5);x = 1;"
        );
    }

    #[test]
    fn skips_multiline_bracket_expressions() {
        let src = "A = [1 2;\n   3 4];\nb = 1;";
        let out = instrument_source(src);
        assert_eq!(
            out,
            "__tl_debug__(1);A = [1 2;\n   3 4];\n__tl_debug__(3);b = 1;"
        );
    }

    #[test]
    fn skips_continuation_lines() {
        let src = "x = 1 + ...\n    2;\ny = 3;";
        let out = instrument_source(src);
        assert_eq!(
            out,
            "__tl_debug__(1);x = 1 + ...\n    2;\n__tl_debug__(3);y = 3;"
        );
    }

    #[test]
    fn skips_structural_keywords_and_arguments_block() {
        let src = "if x\nelse\nend\nfunction y = f(a)\narguments\n    a double = 1\nend\ny = a;\nend\ncase 1";
        let out = instrument_source(src);
        assert_eq!(
            out,
            "__tl_debug__(1);if x\nelse\nend\nfunction y = f(a)\narguments\n    a double = 1\nend\n__tl_debug__(8);y = a;\nend\ncase 1"
        );
    }

    #[test]
    fn transpose_is_not_a_string() {
        let src = "a = b';\nc = 1;";
        let out = instrument_source(src);
        assert_eq!(out, "__tl_debug__(1);a = b';\n__tl_debug__(2);c = 1;");
    }

    #[test]
    fn block_comments_are_skipped() {
        let src = "%{\nfake; statement;\n%}\nx = 1;";
        let out = instrument_source(src);
        assert_eq!(out, "%{\nfake; statement;\n%}\n__tl_debug__(4);x = 1;");
    }

    #[test]
    fn classdef_files_are_left_alone() {
        let src = "classdef Foo\n    methods\n        function f(obj)\n            disp(1);\n        end\n    end\nend";
        assert_eq!(instrument_source(src), src);
    }

    #[test]
    fn try_catch_is_instrumented_safely() {
        let src = "try\n  z = 1;\ncatch\n  z = 2;\nend";
        let out = instrument_source(src);
        assert_eq!(
            out,
            "__tl_debug__(1);try\n  __tl_debug__(2);z = 1;\ncatch\n  __tl_debug__(4);z = 2;\nend"
        );
    }

    /// 端到端：断点命中 → 暂停事件 → resume(continue) → 运行完成。
    /// 内核状态进程级全局，与其它内核测试一样串行化。
    #[test]
    fn breakpoint_pauses_and_resumes() {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (tx, rx) = std::sync::mpsc::channel();
        set_listener(Arc::new(move |ev| {
            let _ = tx.send(ev.line);
        }));
        set_breakpoints(vec![2]);
        let handle = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(move || {
                let mut engine = crate::Engine::new().expect("engine");
                engine.run_debugged("dbg", "a = 1;\nb = 2;\nc = a + b;")
            })
            .expect("spawn");
        let line = rx
            .recv_timeout(std::time::Duration::from_secs(120))
            .expect("paused event");
        assert_eq!(line, 2, "must pause at breakpoint line 2");
        assert!(is_paused());
        resume(ResumeMode::Run);
        let report = handle.join().expect("run thread");
        assert!(report.error.is_none(), "{:?}", report.error);
        assert!(!is_paused());
        set_breakpoints(Vec::new());
    }
}
