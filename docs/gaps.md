# 已知差距与限制

> v0.1 的诚实清单。每条标注：影响、原因、计划。功能对标状态见 [parity-checklist.md](parity-checklist.md)。

## A. 与计划文档不一致的实现选择

计划文档写的是"Monaco 编辑器 + Plotly 绘图"，实际采用自研实现。**功能可用，但字面偏离**。

| # | 项 | 实际实现 | 偏离原因 | 计划 |
|---|---|---|---|---|
| A1 | 编辑器 | 自研 overlay 高亮（textarea + `pre`） | Monaco 约 5MB 且通常需 CDN/npm，与"离线、无 CDN、包体可控"冲突 | 可复议，见 [decisions.md](decisions.md) ADR-005 |
| A2 | 绘图 | 自研 FigureScene→SVG 渲染器 | Plotly.js 全量约 3.5MB | 可复议，见 ADR-004 |

**A1 带来的具体缺失**：无自动补全、无括号匹配、无多光标、无内联错误波浪线、无查找替换、无 minimap。
**A2 带来的具体缺失**：无缩放/平移/hover tooltip/图例点击开关；大数据（>10k 点）无 WebGL 加速；
3D 图（`surf`/`mesh`/`plot3`/`contour`）未实现渲染。

## B. 功能缺口

| # | 缺口 | 影响 | 计划 |
|---|---|---|---|
| B1 | 5 个工具箱未完整 | `toolbox/` 共 103 个纯 m 函数（Signal 19 / Image 28 / Stats 17 / Control 29 / Optimization 10）；各领域可用函数（内核 + 工具箱）已达 20–30 个，Optimization 共 20 个（`quadprog` 仍受 K13 阻塞） | v0.2 已交付；剩余项见各工具箱文档 |
| B2 | MEX 仅能加载/探测 | v0.2 用 `libloading` 实现真实加载 + `mexFunction` 入口探测（`mex load <path>`，Windows/Linux/macOS 均可）；透明 `foo(x)` 调用仍受 K12 阻塞 | v0.3（待内核注册 API） |
| B4 | `.mat` 读写**部分**可用 | 实测 `save`/`load` 与 MATLAB R2023b **v7 双向兼容**（TensorLab 存的文件 MATLAB 可读，MATLAB 默认存的 v7 文件 TensorLab 可读）；`-v7.3`（HDF5）报错 `MATLAB v7.3 MAT-files are HDF5-backed and are not supported yet` | v7.3 按需支持 |
| B5 | 文件 IO **部分**可用 | 实测可用：`fopen`/`fprintf`/`fread`/`fwrite`/`fclose`/`textscan`；缺失：`fscanf`（`Undefined function`）等 | 按需补齐 |
| B6 | 3D/图像类绘图未渲染 | 内核能导出对应 plot 类型，桌面端渲染器只支持 line/bar/stem/scatter/stairs/area/errorbar/histogram（2D），`surf`/`mesh`/`plot3`/`contour`/`imagesc` 未渲染 | v0.3 候选 |
| B8 | 变量预览上限 | ≤4096 元素的 CPU 变量预览已提升到 **128** 个数值；>4096 元素仍用快照预览（16 个）；非数值类型显示 `—` | 按需继续提升（变量编辑器） |
| B9 | `help` 只有 **9** 个主题 | `fft`/`plot`/`eig`/`inv`/`det`/`magic`/`linspace`/`fprintf`/`strcat`；非 MATLAB 全文档 | 随内置函数增长同步维护 |
| B10 | 表达式语句的匿名结果值不单独暴露 | `RunReport.resultText` 恒为 `None`（`outcome_to_report` 固定写入 `None`；控制台文本里含 `ans = ...`） | 按需 |
| B11 | 调试器为宿主层语句级实现 | 断点/单步基于**源码插桩 + 阻塞式内置函数**（内核无调试 API）：粒度到语句行（`else`/`case`/`end` 等结构行不暂停）；暂停期间变量面板不刷新；无 `dbstop in <fn>`、条件断点、`dbstack`；调试路径仅在**当前文件**生效（进入其它 .m 函数不暂停）。见 [ui-desktop.md](ui-desktop.md) §四 | 待内核提供调试钩子 |

## C. 测试覆盖缺口

| # | 缺口 | 影响 | 计划 |
|---|---|---|---|
| C2 | 图窗渲染无像素级验证 | GUI 断言到**结构级**（轴数量、折线点数、刻度标签、viewBox），不能确认"画对了"（颜色/像素位置） | v0.3 |
| C4 | 仅 Windows 验证 | macOS/Linux 行为未知（大栈线程、路径分隔、HDF5 构建方式） | v0.3 |
| C5 | 大量内核能力"未验证" | [parity-checklist.md](parity-checklist.md) 中仍有 **38 处 🟡**（内核声明支持但未端到端验证） | v0.3 优先清 |

## D. 工程与环境

| # | 项 | 说明 |
|---|---|---|
| D1 | 仓库已初始化 git，但终端无 `git` 可执行文件 | `.git` 隐藏目录存在（仓库已初始化）；开发终端 PATH 中没有 `git`，无法执行 `git status/log/tag`，因此冻结仍以文件标记落地、无法核对 tag | 将 git 加入 PATH 后补 tag |
| D2 | 开发机杀软会锁死新编译 exe | 见 [decisions.md](decisions.md) ADR-009；实测运行中的 `tl-mcp-server.exe` 也会被短暂占用（`The process cannot access the file...`） | 复制副本运行 / 等待后重试 |
| D3 | 冷构建耗时 | debug 全量分钟级；单独重编 `tl-desktop` 约 7 分钟；release + LTO 约 60 分钟；全量测试约 40 分钟（见 [jit-benchmarks.md](jit-benchmarks.md) 与 [testing.md](testing.md)） | v0.3 测试提速 |
| D4 | 需要 VS 自带 CMake | 系统 CMake 不识别 VS 2026 生成器，见 [development.md](development.md) 坑 2 | — |
| D5 | release 二进制约 148–156 MB | 实测：`tensorlab-desktop.exe` 156,038,656 B、`tensorlab.exe` 148,066,816 B、`tl-mcp-server.exe` 148,102,656 B（约 141–149 MiB），与"包体 < 30MB"目标差距大 | strip/LTO/裁剪内核特性 |
| D6 | 无 CI | 无 `.github/workflows`，无自动构建/测试流水线 | v0.3 |
| D7 | 反复运行 `tensorlab.exe` 会偶发残留占用 | 表现为 `failed to remove file ... (os error 5)` 或 `The process cannot access the file because it is being used by another process`；`taskkill /F` 偶发 `Access is denied`；见 [development.md](development.md) 坑 4 变体 | 复制副本运行 / 重试 |
| D8 | `cargo xtask` 依赖 `.cargo/config.toml` 别名 | 已补上（`xtask = "run --package xtask --"`）；缺失时报 `no such command: xtask`，见 [development.md](development.md) 坑 7 | — |

## D2. 内核能力缺口（限制工具箱实现）

由 RunMat 0.6.2 缺失导致的实现受限项（详见 [kernel-notes.md](kernel-notes.md) 坑 14/17/19/20/22）：

| 编号 | 缺失/受限 | 影响 |
|------|-----------|------|
| K1 | 文件函数不支持 `nargin`/`varargin`/`varargout` | 工具箱必须用 `arguments` 块声明可选参数 |
| K2 | 标量不是张量（`x(:)`/切片赋值报错） | 工具箱入口需显式向量化；`fminsearch` 要求 `numel(x0)≥2` |
| K3 | `reshape` 在个别函数体内触发栈溢出 | 需规避并单独实跑验证 |
| K4 | `min`/`max` 偶发编译期误判（`reduction dimensions use one-based indexing`） | 索引/边界钳制一律用标量 `if` |
| K6 | 无 `expm` | state-space `c2d` 不可行（tf 路径可后续实现） |
| K7 | 无 `padarray`/`im2col`/`colfilt`/`nlfilter`/`im2col` | 形态学/滤波用显式循环实现（性能较低） |
| K8 | `classdef` 依赖路径源身份 | 0.6.2 解析器支持 classdef，且会按源路径发现**同目录** class 文件（`Vec2` 实测通过）。v0.3-dev 已新增 `Engine::run_path`（读盘）与 `Engine::run_at`（执行缓冲区文本 + 用磁盘路径做 companion 发现）；`run_file`/demo/MCP `run` 走 Path，桌面编辑器在文件已保存时经 `run_at` 支持 classdef。未保存缓冲区、CLI REPL、MCP `eval` 无路径，仍为 Text。`zpk`/`pid` 等模型对象尚待实现 |
| K9 | `exist()` 对算子式内置函数返回 0 | 覆盖统计不能只看 `exist`（如 `abs` 实际可用） |
| K10 | `filtfilt` 要求信号长度 > `3*(nfilt-1)` | 短信号需回退 `filter` |
| K11 | 无 `chi2inv`/`gammainc`/`erfc`/`betainc`/`poly` | 已用 `chi2cdf`+二分补齐 `chi2inv`；其余按需自实现 |
| K12 | 无宿主函数注册 API（MEX 透明调用受阻） | `Value::ExternalFunctionHandle` 只是类方法句柄名，不能注册宿主函数；`tl-mex` 因此实现为**动态库加载/入口探测桥**（`mex load <path>`），MATLAB MEX 的 `foo(x)` 透明调用需等待内核提供注册 API。另：`mxArray` 二进制布局未公开，照抄违反 clean-room 红线 |
| K13 | **目录级函数解析污染**（`quadprog` 阻塞） | **已定位最小触发条件**：同一目录下语义函数定义达到 **3 个**时解析表错位。最小复现 `repro/k13-quadprog-pollution/`（`fminsearch.m` + `quadprog.m`，一个目录 + 两个 `.m`）：`fminsearch` 报 `Not enough input arguments`；只放 `fminsearch.m` 时正常。错位也可能表现为 `shape mismatch for slice assign` 或静默错误数值。`quadprog` 实现已验证正确（解 `[1.5,2.5]`、fval −12.5），但为避免破坏 `fminsearch` **暂不交付**【上游 issue 草稿见 `repro/README.md`，待提交】 |
| K14 | **目录级污染（编译错误错位归属）** | 向已有工具箱目录新增 `.m` 会让同目录旧函数编译失败，甚至把调用错误绑定到同目录另一个函数（`semantic function lsqnonneg expected 2 inputs, got 9`）；典型报错会把兄弟文件的错误归到 `lsqlin.m`（`local may be read before it is assigned`）。最小复现 `repro/k14-lsqlin-pollution/`（一个目录 + 两个 `.m`）。规避：新增复杂函数放独立子目录（`toolbox/optim_*`、`toolbox/stats_extra/*`）；详见 kernel-notes 坑 26 |
| K16 | **切片后缀转置被忽略** | **仍复现**：`x(2:end-1)'` 返回列向量（`30x1`），完整 `x'` 正常（`1x32`），导致隐式广播成矩阵。规避：逐元素循环或显式构造；canary `k16_slice_suffix_transpose_is_ignored`；坑 28 |
| K18 | **struct 的残留限制** | 无 `isstruct`/`iscell`（用 `strcmp(class(x),...)`）；结构体数组不能自动扩展（`s(2).a=2` 报 `Struct subscript out of bounds`），只能用 `struct('f',{...})` 构造；坑 31 |
| K19 | **性能上限由 RunMat 执行层决定** | release 对标 MATLAB R2023b：`script_loop_5k` 405ms vs 0.04ms、`function_calls_100` 917ms vs 0.023ms、`matmul_300` 236ms vs 3.5ms、`fft_16k` 83ms vs 2.8ms、`vector_math_100k` 139ms vs 16ms。诊断：`used_jit=true`、`native_osr_transfers=0`（顶层脚本循环从未 OSR）、`vectorized_native_regions>0`；release/debug steady 几乎相同；20 次预热（越 specialized 阈值 16）无改善。函数调用路径拆解：跨文件空函数约 **4.4ms/次**，同文件局部 helper 省约 25%，`@` 句柄无收益；根因是 `session/run.rs` 每次调用都做文件搜索 + 读源码 + project/companion 发现。TensorLab 无法替换内置实现（K12）且 tiering 配置仅 `pub(crate)` 测试接口，因此标量循环/数值内核的进一步优化需上游改动。详见 [jit-benchmarks.md](jit-benchmarks.md) §四/§4.1 |
| K20 | **文件函数内 `max(向量, 标量)` 被降维解释** | 编译报 "reduction dimensions use one-based indexing"（脚本上下文同名调用正常，见 K4 的表亲）。规避：循环截断。canary `k20_max_vector_scalar_fails_to_compile_in_file_function`；2026-11-04 阶段 15 实证 |
| K21 | **布尔/字符与字面量的 `==`/`~=` 比较强转 f64 失败** | `b == true` 报 "cannot convert Bool(true) to f64"；`s ~= '6H'` 报 "cannot convert CharArray..."。规避：`~x` / `strcmp`。canary `k21_bool_and_char_comparison_coerce_to_f64`；2026-11-04 阶段 13/15 实证 |
| K22 | **循环内对 struct 字段读取值做预分配索引存储失败** | `s(k) = a.f`（循环内）报 "Slicing assignment only supported on tensors"；算术物化 `*1`/`+0` 无效。规避：concat 累积 `s = [s, a.f]`。canary `k22_indexed_store_of_struct_field_value_in_loop_fails`；2026-11-04 阶段 16 实证 |
| K23 | **字段 + end/区间切片复合读取失败** | `a.f(:, end - 1)` 报 "getfield: invalid index element"（普通矩阵同形态正常）。规避：先取局部变量再切片。canary `k23_field_end_slice_read_fails`；2026-11-04 阶段 16 实证 |
| K24 | **文件函数内循环花括号生长被编译期拒绝** | `c = {}` + 循环内 `c{k} = ...` 报 "index for dimension 1 is outside the proven bound 0"。注意：列区间切片赋值 `P(:, a:b) = ...` 与单次字面索引花括号存储在**隔离目录**下正常（sic3d 首版观测到的对应失败是 K13/K14 污染假象）。规避：行块堆叠矩阵 + concat。canary `k24_cell_growth_and_column_range_assign_fail_in_file_functions`；2026-11-04 阶段 16 实证 |
| K25 | **脚本上下文包调用的降级受同脚本后续语句影响** | `a1 = pkg.f(...)` 的可编译性取决于 `a1.phi` 后续是否被读入局部变量（sicpvt.asymmetry.solve 的 probe16c/j 复现；无泛化最小复现）。规避：包调用返回值字段立即读入局部变量。canary `k25_package_result_fields_read_into_locals_work`（仅兼容模式）；2026-11-04 阶段 16 实证 |
| K26 | **转置结果的二维（双索引）读取越界** | `V = v1'; V(idx, :)` 报 "Index out of bounds"；`V(idx)` 一维正常；普通矩阵双索引正常；转置 operand 参与矩阵乘正常。规避：`reshape(v, 1, n)` 物化代替转置。canary `k26_transposed_value_two_index_read_fails`；2026-11-04 v0.4 numerics 实证 |
| K27 | **列向量的 1-D 区间读取归一化为行向量** | `c = (1:5)'; c(1:3)` 返回 1x3（MATLAB 应为 3x1）；且竖向 concat 生长的列向量做 1-D 区间读取直接报 "cannot convert Tensor to f64"（横向 concat 行向量正常）。规避：数值算法内部统一行主序存储 + 横向 concat + `reshape(., n, 1)` 物化。canary `k27_column_vector_range_read_normalises_to_row`；2026-11-04 v0.4 numerics 实证 |
| K28 | **`continue` 被编译为 break** | 循环内 `continue` 终止整个循环（sicem.eddy_axisym 的 q_surf 累计全零根因；`doe_run.m` 的 continue 从未实际执行故此前未暴露）。规避：嵌套 if 改写。2026-11-05 v0.5-C stage18 实证；Rust canary 待固化 |
| K29 | **包级 lowering 连坐（非字面量 arguments 默认值）** | 包目录任一文件带 `opts = struct()` 类默认值 → 整包所有函数报 "unsupported arguments default value"，错误归属到被调用的兄弟函数名（stage11c 报 heater_electrical.m、stage12 报 skin_depth.m）。规避：arguments 默认值只允许字面量/`[]`。2026-11-05 v0.5-C 回归实证（stage11c/stage12 连坐失败→修复后补跑 PASS） |
| K30 | **栈溢出组合触发** | `turns=0 + 非零 a_bc Dirichlet + N=234` 同现时 VM 线程溢出（0xC00000FD）；各条件单独或 N≈100 均正常。规避：组合规避；最小化复现待做。2026-11-05 v0.5-C stage18 制造解探针实证 |
| K31 | **稠密复数回除性能塌方** | N=234 单解 ~3.5 min、N=414 ~30 min（实/复同系统结果逐位一致——正确性无虞，纯性能）；较实数慢 1–2 个量级。规避：验证网格 ≤N≈300；生产路径等 Rust L1 复数求解器（K12 注册前置）。2026-11-05 v0.5-C 实证 |
| K32 | **脚本上下文的字段+区间切片复合读取（K23 家族扩展）** | `design.actual(i,:)`（脚本内）同样报 "getfield: invalid index element"；此前仅记录文件函数上下文。规避不变：字段先取局部再切片。2026-11-05 v0.5-C GSD DOE 实证 |
| K33 | **新增文件触发目录级 lowering 随机失败 + 错误归属漂移（K13/K14 的组合升级）** | 向 toolbox 树新增 sickh 生态 ~24 文件后：同一文件在不同入口脚本下时编译过时编译炸（`local may be read before it is assigned` / `brace indexing requires a cell-like value` / `Slicing assignment` 等文本随机归属到无关文件）。可证伪定位的真毒形态：(a) 函数调用参数语境下的 cell `{k}`/string `(k)` 索引被展开为逗号列表（两步物化规避）；(b) 运行时大小 `cell(n)` 与一切数组增长赋值被拒（固定容量+切片规避）；(c) `reshape(x,1,n)`/空维度 reshape（列向量直传规避）；(d) `struct('f',cellval)` 展开为 struct array 混合体（动态字段赋值规避）；(e) `isfinite([])` 产生 0 元素逻辑数组（numel 前置检查规避）；(f) `reshape(h,[],1)` 空维度（逐元素复制规避）；(g) string 数组字面量构造 `["a","b",...]` 与单值 `for dir = -1`、多输出局部函数在新增文件组合下随机失败（字面量 fprintf 行 + 单 struct 输出 + 预分配方向向量规避）；(h) 参数数组在函数内的 `(i,j)` 双索引循环读取（单列 API + 调用方逐列调用规避）。**架构规避**：治理层函数降为 plain 函数（`sickh_tools/` + 测试伴随目录 `tests/sickh/`），包函数只做装配，循环逻辑脚本侧内联。物理编排层调用图（k3/k5/k6/k7）稳定通过。2026-11-08 sickh M0-M3 实证；最小复现待做 |

## E. 语义层面的注意点

| # | 项 | 说明 |
|---|---|---|
| E1 | 字符串拼接不能用 `+` | `+` 对 char 做**数值加法**（实测 `'a'+'b'` → `195`，与 MATLAB 一致），拼接用 `strcat` |
| E2 | 回显格式由内核决定 | 内核 console streams 是回显的权威来源；`display_events` 仅在本次执行**无 console 输出**时兜底匿名值（避免 `x = 42` 后多一行 `42`）。`format` 由内核实现，宿主不可调整；`format short g;` 写成带分号的复合语句会报语法错（`format` 只接受一个参数） |
| E3 | 图窗 payload 不可 round-trip | 剥离 `*_source` 后无法 `import_figure_scene`（见 ADR-007） |
| E4 | `magic(4)` 奇异 | 与真 MATLAB 一致（实测 `inv(magic(4))` 报 `matrix is singular to working precision`）；demo 因此用 `magic(5)` |
| E5 | `help` 依赖宿主拦截 | 内核**未定义** `help`：绕过 `tl-commands` 直接执行 `help('x')` 得到 `Undefined function: help`（不是栈溢出）；文档必须经宿主 `help_topic` 提供 |

## F. 关闭记录

（每关闭一条，从上面移到此处并注明版本）

| # | 关闭于 | 说明 |
|---|---|---|
| B3 | v0.3-dev | JIT 在宿主层暴露：`Engine::with_jit(false)`、`Engine::kernel_stats()`（jit_compiled / interpreter_fallback / osr / vectorized_regions）与 `tensorlab bench --no-jit` |
| B7 | v0.2.1（2026-10-18） | Windows MSI（84.41 MiB）+ 便携 ZIP 已发行，见 [releases/v0.2.1.md](releases/v0.2.1.md)；打包脚本见 [packaging/](../packaging/) |
| C1 | v0.2 | 桌面端 UI 自动化：`gui_smoke.mjs`（Node + 无头 Edge + CDP，**11 条断言**：高亮、About/F1/Escape、Run 输出/变量表/SVG、图窗结构、REPL、历史、错误样式、右键复制、`Ctrl+L`、`clearConsole`），经 `src-tauri/tests/gui_smoke.rs` 接入 `cargo test` |
| C3 | v0.3-dev | 性能基准：`tensorlab bench`（10+5 协议）、`bench/matlab_bench.m`、`jit_tiering.rs` canary 与 [jit-benchmarks.md](jit-benchmarks.md) |
| K5 | v0.3-dev（2026-10-17） | 「无 struct」结论错误：`struct()` 构造、字段读写、动态字段、`getfield`/`setfield`/`fieldnames`/`isfield`、`struct('f',{...})` 数组均实测可用。已交付 `strel`/`bwconncomp`/`regionprops` 并接入形态学函数；残留限制见 K18 |
| K15 | v0.3-dev（2026-10-17） | **工具箱函数内的匿名闭包丢环境**已不复现：分离 helper / 同文件局部 helper 均正常（解 ≈ `[1.90909, 1]`）；canary `k15_closure_keeps_captured_environment` 防回退；坑 27 |
| K17 | v0.3-dev | **标量归约/单根返回差异已对齐 MATLAB**：`max([1 5 3])` 返回 `1x1` 标量；`roots([1 -2])` 返回 `1x1`，`p(1)` 可用；canary `k17_scalar_reduction_and_single_root_are_matlab_compatible` 防回退；坑 29 |
