//! 跨 IPC 的 serde 载荷：CLI / Tauri / MCP 共享的运行报告与变量/图窗数据。

use serde::{Deserialize, Serialize};

/// 控制台事件流中的一条输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StreamEntryPayload {
    /// 普通输出（stdout / 回显）
    Output { text: String },
    /// 错误输出（stderr）
    Error { text: String },
    /// clc：清空控制台
    ClearConsole,
}

/// 运行诊断（错误或警告）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticPayload {
    /// MATLAB 错误标识符（如 "MATLAB:minrhs"）
    pub identifier: Option<String>,
    /// 人类可读消息
    pub message: String,
    /// 1-based 源码行（可解析 span 时填充）
    pub line: Option<usize>,
    /// 调用栈
    pub callstack: Vec<String>,
    /// 严重级别：error | warning | info | hint
    pub severity: String,
}

/// 变量面板条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VariableInfo {
    pub name: String,
    pub class_name: String,
    /// [rows, cols]（N-D 取前两维）
    pub shape: Vec<usize>,
    /// 近似占用字节
    pub bytes: Option<u64>,
    /// 单行预览
    pub preview: String,
    pub is_gpu: bool,
}

/// 图窗载荷：FigureScene JSON（camelCase schema，见 docs/kernel-notes.md）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FigurePayload {
    /// 内核 figure handle
    pub handle: u32,
    /// FigureScene 序列化 JSON（前端自行渲染）
    pub scene_json: serde_json::Value,
}

/// 运行报告：一次 run/eval 的全部宿主可见结果（v0.1 冻结契约）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RunReport {
    /// 有序控制台事件
    pub events: Vec<StreamEntryPayload>,
    /// 语句级返回值文本（无分号回显已并入 events；此字段供单行 eval 取值）
    pub result_text: Option<String>,
    /// 错误（首个 Error 级诊断）
    pub error: Option<DiagnosticPayload>,
    /// 警告
    pub warnings: Vec<DiagnosticPayload>,
    /// 变量面板（全量快照）
    pub variables: Vec<VariableInfo>,
    /// 本次运行触及并成功导出的图窗
    pub figures: Vec<FigurePayload>,
    /// 执行耗时
    pub duration_ms: f64,
    /// 是否用到 JIT
    pub used_jit: bool,
}

impl RunReport {
    pub fn ok(events: Vec<StreamEntryPayload>) -> Self {
        Self {
            events,
            ..Default::default()
        }
    }

    /// 全部事件拼接为控制台文本（CLI/MCP 便捷用）
    pub fn console_text(&self) -> String {
        let mut out = String::new();
        for e in &self.events {
            match e {
                StreamEntryPayload::Output { text } => {
                    out.push_str(text);
                    if !text.ends_with('\n') {
                        out.push('\n');
                    }
                }
                StreamEntryPayload::Error { text } => {
                    out.push_str(text);
                    if !text.ends_with('\n') {
                        out.push('\n');
                    }
                }
                StreamEntryPayload::ClearConsole => out.clear(),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 前端（tl-desktop/ui/index.html）按这些 tag 分派控制台渲染。
    /// 改动 tag 命名 = 破坏前端契约（历史 bug：曾因 PascalCase/小写不一致导致
    /// fprintf 等全部 stdout 不显示），故用测试钉死。
    #[test]
    fn stream_entry_tags_are_camel_case() {
        let out = serde_json::to_value(StreamEntryPayload::Output { text: "x".into() }).unwrap();
        assert_eq!(out["kind"], "output");
        assert_eq!(out["text"], "x");

        let err = serde_json::to_value(StreamEntryPayload::Error { text: "e".into() }).unwrap();
        assert_eq!(err["kind"], "error");

        let clc = serde_json::to_value(StreamEntryPayload::ClearConsole).unwrap();
        assert_eq!(clc["kind"], "clearConsole");
    }
}
