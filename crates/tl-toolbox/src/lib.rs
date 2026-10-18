//! TensorLab L7 — 宿主命令注册表（PhotoCraft 风格）。
//!
//! **Everything is a command.** GUI / CLI / MCP 面向宿主的操作（非 m 代码）
//! 统一为 `Command` trait + `inventory` 自动注册。m 语言的内置函数
//! （sin/fft/plot/…）由内核提供、不经本表；本表只收**宿主命令**。

use tl_kernel::{variable_table, Engine};
use tl_types::RunReport;

/// 宿主命令（区别于 m 语言函数）
pub trait Command: Sync + Send {
    /// 命令名（小写；CLI/MCP/GUI 共用）
    fn name(&self) -> &'static str;
    /// 分组：workspace / figures / session / help / app
    fn category(&self) -> &'static str;
    /// 一行描述（help 列表与 UI 提示）
    fn description(&self) -> &'static str;
    /// 执行；args 为已去引号的参数列表
    fn run(&self, engine: &mut Engine, args: &[String]) -> CommandResult;
}

/// 命令输出：文本 / 完整 RunReport（前端据此更新面板）/ 无输出
#[derive(Debug)]
pub enum CommandResult {
    Text(String),
    Report(RunReport),
    None,
}

/// 注册表条目（inventory 需要 Sized 类型）
pub struct CommandEntry(pub &'static dyn Command);

inventory::collect!(CommandEntry);

/// 遍历注册表（按 name 排序）
pub fn all_commands() -> Vec<&'static dyn Command> {
    let mut all: Vec<&'static dyn Command> =
        inventory::iter::<CommandEntry>().map(|e| e.0).collect();
    all.sort_by_key(|c| c.name());
    all
}

pub fn find(name: &str) -> Option<&'static dyn Command> {
    inventory::iter::<CommandEntry>()
        .map(|e| e.0)
        .find(|c| c.name() == name)
}

/// help 总览
pub fn help_overview() -> String {
    let mut out = String::from("TensorLab host commands:\n\n");
    for c in all_commands() {
        out.push_str(&format!("  {:<12}  {}\n", c.name(), c.description()));
    }
    out.push_str("\nMATLAB builtins: run help('<topic>') in the console, e.g. help('fft').\n");
    out
}

// ---------------- 内置宿主命令 ----------------

struct HelpCmd;
impl Command for HelpCmd {
    fn name(&self) -> &'static str {
        "help"
    }
    fn category(&self) -> &'static str {
        "help"
    }
    fn description(&self) -> &'static str {
        "List host commands; help <name> for details"
    }
    fn run(&self, _engine: &mut Engine, args: &[String]) -> CommandResult {
        match args.first().map(String::as_str) {
            None | Some("commands") => CommandResult::Text(help_overview()),
            Some(name) => match find(name) {
                Some(c) => CommandResult::Text(format!(
                    "{} — {}\n(category: {})",
                    c.name(),
                    c.description(),
                    c.category()
                )),
                None => CommandResult::Text(format!(
                    "Unknown host command '{name}'. Try 'help' to list commands."
                )),
            },
        }
    }
}
inventory::submit! {
    CommandEntry(&HelpCmd)
}

struct VarsCmd;
impl Command for VarsCmd {
    fn name(&self) -> &'static str {
        "vars"
    }
    fn category(&self) -> &'static str {
        "workspace"
    }
    fn description(&self) -> &'static str {
        "Show workspace variables (whos-style table)"
    }
    fn run(&self, engine: &mut Engine, _args: &[String]) -> CommandResult {
        let vars = engine.variables();
        if vars.is_empty() {
            return CommandResult::Text("No variables in the workspace.".to_string());
        }
        CommandResult::Text(variable_table(&vars))
    }
}
inventory::submit! {
    CommandEntry(&VarsCmd)
}

struct ClearCmd;
impl Command for ClearCmd {
    fn name(&self) -> &'static str {
        "clear"
    }
    fn category(&self) -> &'static str {
        "workspace"
    }
    fn description(&self) -> &'static str {
        "Clear all workspace variables (host-level reset)"
    }
    fn run(&self, engine: &mut Engine, _args: &[String]) -> CommandResult {
        engine.clear();
        CommandResult::None
    }
}
inventory::submit! {
    CommandEntry(&ClearCmd)
}

struct FiguresCmd;
impl Command for FiguresCmd {
    fn name(&self) -> &'static str {
        "figures"
    }
    fn category(&self) -> &'static str {
        "figures"
    }
    fn description(&self) -> &'static str {
        "List open figures (handle + title if any)"
    }
    fn run(&self, engine: &mut Engine, _args: &[String]) -> CommandResult {
        // 通过 m 代码取 figure 句柄；无图窗时 findobj 返回空
        let report = engine.run("figures", "fprintf('%d\\n', findobj('Type', 'figure'));");
        let text = report.console_text();
        if text.trim().is_empty() {
            CommandResult::Text("No open figures.".to_string())
        } else {
            CommandResult::Text(format!("Open figures:\n{text}"))
        }
    }
}
inventory::submit! {
    CommandEntry(&FiguresCmd)
}

struct VersionCmd;
impl Command for VersionCmd {
    fn name(&self) -> &'static str {
        "version"
    }
    fn category(&self) -> &'static str {
        "session"
    }
    fn description(&self) -> &'static str {
        "Show TensorLab and kernel versions"
    }
    fn run(&self, engine: &mut Engine, _args: &[String]) -> CommandResult {
        let report = engine.run("version", "version('-release')");
        CommandResult::Text(format!(
            "TensorLab 0.1.0 (kernel: RunMat 0.6.2)\n{}",
            report.console_text()
        ))
    }
}
inventory::submit! {
    CommandEntry(&VersionCmd)
}

/// `mex load <path>` —— 原生扩展（MEX 风格）动态库探测。
///
/// 只做**加载 + 入口探测**：`mexFunction` 的透明 m 代码调用需要内核提供函数注册 API，
/// 当前 RunMat 0.6.2 未公开该入口（详见 crates/tl-mex 与 docs/gaps.md）。
struct MexCmd;
impl Command for MexCmd {
    fn name(&self) -> &'static str {
        "mex"
    }
    fn category(&self) -> &'static str {
        "session"
    }
    fn description(&self) -> &'static str {
        "Load and probe a native (MEX-style) shared library: mex load <path>"
    }
    fn run(&self, _engine: &mut Engine, args: &[String]) -> CommandResult {
        match args.first().map(String::as_str) {
            None | Some("status") => CommandResult::Text(
                "mex: no library loaded this session (probe with: mex load <path>)\n\
                 note: transparent m-code calls require a kernel function-registry API (not provided by RunMat 0.6.2)."
                    .to_string(),
            ),
            Some("load") => {
                let Some(path) = args.get(1) else {
                    return CommandResult::Text(
                        "usage: mex load <path-to-dll|so|dylib>".to_string(),
                    );
                };
                match tl_mex::MexLibrary::load(path) {
                    Ok(lib) => CommandResult::Text(lib.describe()),
                    Err(e) => CommandResult::Text(format!("{e}")),
                }
            }
            Some(other) => CommandResult::Text(format!(
                "mex: unknown subcommand '{other}'. usage: mex [status | load <path>]"
            )),
        }
    }
}
inventory::submit! {
    CommandEntry(&MexCmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_core_commands() {
        let names: Vec<&str> = all_commands().iter().map(|c| c.name()).collect();
        for expect in ["help", "vars", "clear", "figures", "version"] {
            assert!(names.contains(&expect), "missing command: {expect}");
        }
    }

    #[test]
    fn help_overview_lists_all() {
        let text = help_overview();
        assert!(text.contains("vars"));
        assert!(text.contains("help"));
    }

    #[test]
    fn command_metadata_is_complete_and_lowercase() {
        for c in all_commands() {
            assert!(!c.name().is_empty(), "empty command name");
            assert_eq!(
                c.name(),
                c.name().to_lowercase(),
                "command name `{}` must be lowercase",
                c.name()
            );
            assert!(!c.category().is_empty(), "{}: empty category", c.name());
            assert!(
                !c.description().is_empty(),
                "{}: empty description",
                c.name()
            );
        }
    }

    #[test]
    fn registry_is_sorted_and_free_of_duplicates() {
        let names: Vec<&str> = all_commands().iter().map(|c| c.name()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted, "all_commands() must be sorted by name");

        let total = sorted.len();
        sorted.dedup();
        assert_eq!(sorted.len(), total, "duplicate command names registered");
    }

    #[test]
    fn find_resolves_known_and_rejects_unknown() {
        assert!(find("vars").is_some());
        assert!(find("__no_such_command__").is_none());
    }

    #[test]
    fn help_overview_mentions_every_registered_command() {
        let text = help_overview();
        for c in all_commands() {
            assert!(
                text.contains(c.name()),
                "help overview missing `{}`\n{text}",
                c.name()
            );
        }
    }

    #[test]
    fn mex_command_reports_status_usage_and_load_failure() {
        let mut engine = Engine::new().expect("engine");
        let mex = find("mex").expect("mex command");
        match mex.run(&mut engine, &[]) {
            CommandResult::Text(t) => assert!(t.contains("no library loaded"), "{t}"),
            other => panic!("expected Text, got {other:?}"),
        }
        match mex.run(&mut engine, &["load".to_string()]) {
            CommandResult::Text(t) => assert!(t.contains("usage"), "{t}"),
            other => panic!("expected Text, got {other:?}"),
        }
        match mex.run(
            &mut engine,
            &["load".to_string(), "__nope__.dll".to_string()],
        ) {
            CommandResult::Text(t) => assert!(t.contains("failed to load"), "{t}"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn help_command_handles_known_and_unknown_topics() {
        let mut engine = Engine::new().expect("engine");
        let help = find("help").expect("help command");

        match help.run(&mut engine, &["__no_such_command__".to_string()]) {
            CommandResult::Text(t) => assert!(t.contains("Unknown host command"), "{t}"),
            other => panic!("expected Text, got {other:?}"),
        }
        match help.run(&mut engine, &["vars".to_string()]) {
            CommandResult::Text(t) => {
                assert!(t.contains("vars"), "{t}");
                assert!(t.contains("category:"), "{t}");
            }
            other => panic!("expected Text, got {other:?}"),
        }
        // 裸 help = 列出全部宿主命令
        match help.run(&mut engine, &[]) {
            CommandResult::Text(t) => assert!(t.contains("TensorLab host commands"), "{t}"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn vars_command_on_empty_workspace() {
        let mut engine = Engine::new().expect("engine");
        let vars = find("vars").expect("vars command");
        match vars.run(&mut engine, &[]) {
            CommandResult::Text(t) => assert!(t.to_lowercase().contains("no variables"), "{t}"),
            other => panic!("expected Text, got {other:?}"),
        }
    }
}
