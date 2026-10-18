//! TensorLab L4 — 运行结果后处理。
//!
//! 内核已完成解析/求值（runmat VM/JIT）；本层负责宿主侧语义：
//! - 报告修饰（help 主题提取、错误行号合成）
//! - 演示脚本验收器（demos are the spec）
//! - REPL 行预处理（命令语法降级为函数调用的提示等）

use tl_kernel::Engine;
use tl_types::{RunReport, StreamEntryPayload};

/// 运行一个脚本文件并断言零错误（demo 验收 / CI 用）
pub fn run_file_ok(engine: &mut Engine, path: &str) -> anyhow::Result<RunReport> {
    let src = std::fs::read_to_string(path)?;
    let name = std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    let report = engine.run_path(path, &src);
    if let Some(err) = &report.error {
        anyhow::bail!(
            "demo {} failed at line {:?}: {}",
            name,
            err.line,
            err.message
        );
    }
    Ok(report)
}

/// `help <topic>`：TensorLab 宿主层主题文档。
///
/// RunMat 0.6.2 内核未提供 `help` 内置函数（调用未定义函数会触发内核
/// 深递归栈溢出），因此宿主层拦截并返回内置文档表。
pub fn help_topic(topic: &str) -> String {
    match topic.trim().to_lowercase().as_str() {
        "fft" => concat!(
            " FFT Discrete Fourier transform.\n",
            "    Y = fft(X) returns the discrete Fourier transform of X.\n",
            "    Y = fft(X, n) pads or truncates X to length n.\n",
            "    See demos/fft_demo.m for a full spectrum example."
        )
        .to_string(),
        "plot" => concat!(
            " PLOT 2-D line plot.\n",
            "    plot(Y) plots the columns of Y against their indices.\n",
            "    plot(X, Y) plots Y versus X; plot(X, Y, 'r--') styles the line.\n",
            "    See demos/plot_demo.m for sine/line styling examples."
        )
        .to_string(),
        "eig" => concat!(
            " EIG Eigenvalues and eigenvectors.\n",
            "    E = eig(A) returns eigenvalues of square matrix A.\n",
            "    [V, D] = eig(A) returns eigenvectors V and diagonal D.\n",
            "    See demos/matrix_ops.m."
        )
        .to_string(),
        "inv" => " INV Matrix inverse. X = inv(A) errors if A is singular.\n    Prefer the backslash solver: x = A\\b. See demos/matrix_ops.m.".to_string(),
        "det" => " DET Matrix determinant. d = det(A). See demos/matrix_ops.m.".to_string(),
        "magic" => " MAGIC Magic square. M = magic(n); magic(4) is singular, odd n is not.".to_string(),
        "linspace" => " LINSPACE Linearly spaced vector. y = linspace(a, b, n) with n points.".to_string(),
        "fprintf" => " FPRINTF Write formatted text to console. fprintf('x = %d\\n', x).".to_string(),
        "strcat" => " STRCAT Concatenate strings horizontally. s = strcat('Ten', 'sor').".to_string(),
        other => format!(
            "No documentation for '{other}'.\n\n\
             Type 'help' to list TensorLab host commands;\n\
             MATLAB builtins are provided by the RunMat kernel."
        ),
    }
}

/// 评估一行并返回结果文本（无分号回显、disp、错误统一取 console_text）
pub fn eval_to_text(engine: &mut Engine, line: &str) -> RunReport {
    engine.run("repl", line)
}

/// 汇总报告为人类可读的失败信息（测试诊断）
pub fn describe_failure(report: &RunReport) -> Option<String> {
    report.error.as_ref().map(|e| {
        let line = e.line.map(|l| format!(" (line {l})")).unwrap_or_default();
        format!(
            "{}{}{}",
            e.identifier.as_deref().unwrap_or("error"),
            line,
            e.message
        )
    })
}

/// 是否产生了任何图窗
pub fn has_figures(report: &RunReport) -> bool {
    !report.figures.is_empty()
}

/// 输出事件数量（除 ClearConsole）
pub fn output_event_count(report: &RunReport) -> usize {
    report
        .events
        .iter()
        .filter(|e| !matches!(e, StreamEntryPayload::ClearConsole))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_lists_topics() {
        // help 由宿主层提供（内核 0.6.2 无 help 内置函数）
        let text = help_topic("fft");
        assert!(
            text.to_lowercase().contains("fft"),
            "help fft should mention fft"
        );
        let unknown = help_topic("__nope__");
        assert!(unknown.contains("No documentation"));
    }

    #[test]
    fn demo_runner_reports_errors() {
        let mut engine = Engine::new().expect("engine");
        let report = engine.run("bad.m", "this_function_does_not_exist_xyz();");
        assert!(report.error.is_some());
        assert!(describe_failure(&report).is_some());
    }
}
