//! TensorLab L0 — 显示格式化 + UI 载荷类型。
//!
//! 值本体来自内核 `runmat_value::Value`；本层负责：
//! 1. MATLAB 风格的回显/预览格式化（short/long）
//! 2. 跨 IPC 的 serde 载荷（变量面板、图窗、运行报告）

pub mod display;
pub mod payload;

pub use display::{body_text, class_name, compact, echo_line, size_of};
pub use payload::{DiagnosticPayload, FigurePayload, RunReport, StreamEntryPayload, VariableInfo};
