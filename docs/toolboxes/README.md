# 工具箱（Toolboxes）

> TensorLab 的 **L7 层**（`tl-toolbox`）按 MATLAB 领域习惯划分为 5 个工具箱。本目录记录每个工具箱的功能清单与 **MATLAB R2026a 对标状态**。
>
> 这是 **v0.2 开发的主要载体**：本目录的清单即 v0.2 的实现任务分解。

## 一、为什么是这 5 个

对应 MATLAB 中最常用、且能覆盖 TensorLab 目标用户（工程 / 科研 / 教学）的 5 个领域：

| 工具箱 | 目录文档 | MATLAB 对应 | 计划函数数 |
|--------|----------|-------------|------------|
| 信号处理 | [signal.md](signal.md) | Signal Processing Toolbox | 20–30 |
| 图像处理 | [image.md](image.md) | Image Processing Toolbox | 20–30 |
| 优化 | [optimization.md](optimization.md) | Optimization Toolbox | 20–30 |
| 统计与机器学习 | [statistics-ml.md](statistics-ml.md) | Statistics and ML Toolbox | 20–30 |
| 控制系统 | [control.md](control.md) | Control System Toolbox | 20–30 |

## 二、v0.1 实测基线

> 数据来源：用 `exist('<fn>')` 对 246 个 R2026a 目标函数逐一探测（见下方"探测方法"）。**内核 = RunMat 内置**，不是 TensorLab 自己实现。

| 工具箱 | 探测总数 | 内核已提供 | 本版已补齐（`toolbox/`） | 剩余缺口 |
|--------|---------:|-----------:|------------------------:|---------:|
| 信号处理 | 50 | 26 | 11 | 13 |
| 图像处理 | 44 | 15 | 25 | 4 |
| 优化 | 28 | 10 | 2 | 16 |
| 统计与 ML | 60 | 41 | 7（+`range`/`chi2inv` 额外） | 12 |
| 控制系统 | 64 | 17 | 18 | 29 |
| **合计** | **246** | **109** | **63** | **74** |

结论：基线内 **172 个可用（70%）**（内核 109 + 工具箱 63）；另有 `range`、`chi2inv` 两个基线外函数。
文件清单见 [`toolbox/`](../../toolbox)（v0.3-dev 共 104 个 `.m`）。

> v0.3-dev 追加（不计入上表 v0.2 基线）：图像 `strel`/`bwconncomp`/`regionprops` 已按 MATLAB 返回 struct 交付（`struct` 实测可用，[gaps.md](../gaps.md) K5 已关闭）。

### 探测方法（可复现）

```matlab
% 对每个候选函数名探测是否存在（5=builtin, 2=file, 0=不存在）
ns = {'fft','ifft','xcorr','imresize','pca','bode', ...};
for k = 1:numel(ns)
    fprintf('%s %d\n', ns{k}, exist(ns{k}));
end
```

```powershell
target\debug\tensorlab.exe run probe.m
```

> `exist` 对内置函数返回 5、对 `.m` 文件函数返回 2、不存在返回 0（已实测确认）。
> ⚠️ 但 `exist` 对部分**算子式内置函数**返回 0 而实际可用（实测 `abs`），
> 因此"缺口"数量可能略偏保守，见 [kernel-notes.md](../kernel-notes.md) 坑 18。

## 二·五、v0.2 实现策略（关键架构决策）

**缺口函数不 fork RunMat，而是以纯 m-script 工具箱文件交付。**

原因：

1. RunMat 的内置函数注册表（`runmat_runtime::builtins::wasm_registry`）无公开的"注入自定义函数" API；
2. RunMat 支持 `addpath` / `genpath` / `which`（实测 `exist=5`），且 `addpath` 加入的 `.m` 文件函数**在后续执行中立即可调用、并跨 REPL 持久**；
3. 纯 m 实现天然是"工具箱增量"，**零内核改动、零 fork、clean-room 友好**（挂载点复用 L2 Engine 的会话引导）。

落地方式（v0.2 首个增量，**已实现**）：

- 新增仓库根 `toolbox/` 目录，内含 `signal/ image/ optimization/ stats/ control/` 子目录，每个函数一个 `.m` 文件；
- **`tl-kernel`（L2）在 `Engine::new()` 内枚举 `toolbox/` 子目录并逐目录 `addpath`**（不再用 `genpath`），把纯 m 函数挂到内核搜索路径；
- 各 App（CLI/desktop/MCP）经 Engine 引导自动挂载，无需各自处理；
- 目录缺失时静默跳过（发行物可只带二进制）；
- 防回归测试：`tl-commands` 的 `toolbox_m_scripts_are_mounted`。

> 实现落在 L2 而非最初设想的 L7，因为 `Engine` 定义在 `tl-kernel`；L7 `tl-toolbox` 仍是宿主命令注册表。
> 分层不受影响（无新增 crate 依赖，`cargo xtask layers` 仍为 43 条边）。

**编码约束（实测）**：文件函数**不能**用 `nargin`/`varargin`（内核限制，见 [kernel-notes.md](../kernel-notes.md) 坑 14），
**统一用 `arguments` 块**声明可选参数与默认值（R2019b+ 风格，实测可用）：

```matlab
function y = myfun(x, p)
    arguments
        x
        p double = 1     % 默认值；省略时生效
    end
    y = x * p;
end
```

> `arguments` 块同时提供了类型/尺寸校验，与 MATLAB R2026a 的推荐写法一致。

详见 [decisions.md](../decisions.md) ADR-012（草案）。

统计见 [parity-checklist.md](../parity-checklist.md)。

## 三、每个工具箱文档的结构

每份工具箱文档包含：

1. **范围与目标**：涵盖哪些子领域，明确不做什么
2. **函数清单表**：`函数名 | 签名 | 语义要点 | R2026a 对标状态`
3. **状态图例**：✅ 已实现 / 🟡 部分实现 / ⬜ 计划中 / ➖ 明确不做
4. **实现笔记**：算法出处（教科书 / 公开论文，clean-room 要求）、数值陷阱
5. **验收脚本**：对应的 `demos/` 或测试

## 四、状态图例

| 标记 | 含义 |
|------|------|
| ✅ | 已实现且有测试 |
| 🟡 | 部分实现（缺选项 / 边界情形） |
| ⬜ | v0.2 计划实现 |
| ➖ | 明确不做（依赖 Simulink / 专有数据等） |

## 五、实现规范

所有工具箱函数**必须**：

- 是 `tl_toolbox` 内实现 `tl_runtime::Command` 的类型，经 `inventory` 注册
- 提供 `name` / `category`（= 工具箱名）/ `description`
- 错误走 `tl_types::TlError`
- 有 `engine_contract.rs` 中的端到端断言
- 算法可追溯到教科书 / 公开论文（clean-room）

规范细节见 [contributing.md](../contributing.md) §四，分层约束见 [architecture.md](../architecture.md)。
