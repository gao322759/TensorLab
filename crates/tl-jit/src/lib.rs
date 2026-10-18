//! TensorLab L5 — JIT 观测。
//!
//! 内核自带 Turbine JIT（Cranelift，feature "jit" 默认开启）。
//! v0.1 本层只负责从 RunReport 提取执行统计；内核 profiling 的深度
//! 提取（热点、编译时间）留到 v0.3 性能阶段实现。

use serde::{Deserialize, Serialize};
use tl_types::RunReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JitStats {
    pub used_jit: bool,
    pub duration_ms: f64,
    /// 状态栏可读文本
    pub summary: String,
}

pub fn stats_of(report: &RunReport) -> JitStats {
    let tier = if report.used_jit { "JIT" } else { "VM" };
    JitStats {
        used_jit: report.used_jit,
        duration_ms: report.duration_ms,
        summary: format!("{tier} · {:.1} ms", report.duration_ms),
    }
}
