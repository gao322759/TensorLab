//! TensorLab L6 — 原生扩展（MEX 风格）动态库桥。
//!
//! ## 为什么不是"直接跑 MATLAB 的 .mex*"
//!
//! 1. **二进制 ABI 不公开**：MATLAB 的 `mxArray` 内存布局只存在于 MathWorks 的私有头文件中，
//!    照抄违反本项目的 clean-room 红线（见 AGENTS.md §九）。
//! 2. **内核无函数注入入口**：RunMat 0.6.2 的内置函数注册表
//!    （`runmat_runtime::builtins::wasm_registry`）没有公开的"注册宿主函数"API，
//!    因此外部原生函数无法被 m 代码按名字直接调用（已实测 `Value::ExternalFunctionHandle`
//!    只是类方法句柄的名称载体，不是宿主函数注册）。
//!
//! ## 本层提供什么
//!
//! 一个**真实可用**的动态库加载 / 入口探测桥，供宿主层（CLI / 桌面 / MCP）使用：
//! - `MexLibrary::load(path)` — 通过 `libloading` 真正 dlopen/LoadLibrary；
//! - `MexLibrary::has_mex_entry()` — 探测标准入口符号 `mexFunction`；
//! - `MexLibrary::entry_pointer()` — 取入口地址（供将来接内核注册 API）；
//! - `MexLibrary::describe()` — 人类可读的探测报告。
//!
//! 宿主命令 `mex load <path>`（`tl-toolbox`）即基于本层。
//!
//! ## 后续（需要内核支持）
//!
//! 一旦 RunMat 提供"注册宿主函数"的公开 API，本层即可把 `mexFunction` 包装成内核可调用函数，
//! 届时 `foo(x)` 形式的透明调用才成立。当前状态记录于 `docs/gaps.md`（K8 邻近条目）。

use std::path::{Path, PathBuf};

/// MEX 约定的入口符号名。
pub const MEX_ENTRY: &str = "mexFunction";

/// 已加载的原生动态库。
pub struct MexLibrary {
    path: PathBuf,
    lib: libloading::Library,
}

impl MexLibrary {
    /// 加载动态库（`.dll` / `.so` / `.dylib`）。
    ///
    /// `path` 可以是完整路径，也可以是系统库名（交给平台加载器解析）。
    /// 失败时返回带路径的错误，绝不 panic。
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        // SAFETY: 加载任意动态库都会执行其初始化代码；调用方应只加载可信文件。
        let lib = unsafe { libloading::Library::new(&path) }
            .map_err(|e| anyhow::anyhow!("mex: failed to load {}: {e}", path.display()))?;
        Ok(Self { path, lib })
    }

    /// 库文件路径。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 是否导出了标准 MEX 入口 `mexFunction`。
    pub fn has_mex_entry(&self) -> bool {
        self.entry_pointer().is_some()
    }

    /// 返回 `mexFunction` 的地址（未导出时为 `None`）。
    pub fn entry_pointer(&self) -> Option<*const ()> {
        // SAFETY: 只解析符号，不调用。
        unsafe {
            self.lib
                .get::<*const ()>(MEX_ENTRY.as_bytes())
                .ok()
                .map(|s| *s)
        }
        .map(|p| p as *const ())
    }

    /// 人类可读的探测报告（供 `mex load` 输出）。
    pub fn describe(&self) -> String {
        match self.entry_pointer() {
            Some(p) => format!(
                "mex: loaded {}\n  entry `{MEX_ENTRY}` found at {:p}\n  note: 透明 m 代码调用需要内核函数注册 API（当前未提供）",
                self.path.display(),
                p
            ),
            None => format!(
                "mex: loaded {}\n  entry `{MEX_ENTRY}` NOT found\n  note: 该库不是 MEX 兼容库（或入口名不同）",
                self.path.display()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_library_reports_path() {
        let err = match MexLibrary::load("definitely_not_a_real_library_1234.dll") {
            Ok(_) => panic!("loading a bogus library must fail"),
            Err(e) => e,
        };
        let msg = err.to_string();
        assert!(msg.contains("failed to load"), "unexpected message: {msg}");
        assert!(msg.contains("definitely_not_a_real_library_1234"));
    }

    #[cfg(windows)]
    #[test]
    fn loads_system_library_and_reports_no_mex_entry() {
        let lib = MexLibrary::load("kernel32.dll").expect("load kernel32");
        assert!(!lib.has_mex_entry(), "kernel32 must not export mexFunction");
        assert!(lib.describe().contains("NOT found"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn loads_system_library_and_reports_no_mex_entry() {
        let lib = MexLibrary::load("libm.so.6").expect("load libm");
        assert!(!lib.has_mex_entry());
    }
}
