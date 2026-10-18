# MATLAB R2026a 兼容性对标清单

> 目标：**功能对齐 MATLAB 最新版（R2026a）**。本文是对标状态的**唯一清单**，随功能推进更新。
> 图例：✅ 已验证可用 · 🟡 部分/未验证 · ⬜ 未实现 · ➖ 明确不做

## 一、语言核心

| 能力 | 状态 | 备注 |
|---|---|---|
| 数值矩阵（double） | ✅ | 列主序、隐式广播（R2016b+ 语义） |
| 1-based 索引 | ✅ | |
| `end` 索引关键字 | ✅ | `x(1:10:end)`、`x(2:end-1)` |
| 冒号范围 `a:s:b` | ✅ | |
| 矩阵字面量 `[1 2; 3 4]` | ✅ | |
| 转置 / 共轭转置 `'` `. '` | ✅ | |
| 逐元素运算 `.*` `./` `.^` | ✅ | |
| 左除 `\` / 右除 `/` | ✅ | |
| `;` 抑制输出 | ✅ | |
| `%` / `%%` 注释 | ✅ | |
| char 与 string 双类型 | ✅ | `'x'` vs `"x"` |
| 多返回值 `[a,b] = f(x)` | ✅ | |
| 复数 | 🟡 | 类型与显示已实现（`Value::Complex`、`ComplexTensor`），脚本层未系统验证 |
| Cell 数组 | ✅ | `c = {1,'x'}`、`c{3} = 5` 动态扩展实测可用 |
| struct | ✅ | `struct()`、字段读写、动态字段、`getfield` 实测可用（[gaps.md](gaps.md) K5 已关闭） |
| 控制流（`if`/`for`/`while`/`switch`/`try`） | 🟡 | 内核声明支持；本项目未做端到端验证 |
| 用户函数文件（`function`） | 🟡 | 内核声明支持；未验证 |
| 匿名函数 `@(x) x.^2` | 🟡 | 捕获外部变量的闭包已由 canary `k15_closure_keeps_captured_environment` 验证；全量语法未系统验证 |
| 函数句柄 / `feval` / `eval` | 🟡 | 函数句柄调用已验证（K13/K15 canary）；`feval`/`eval` 未验证 |
| `global` / `persistent` | ⬜ | |
| 结构体数组 / 逗号分隔列表 `{:}` | 🟡 | `struct('f',{v1,v2})` 可构造；`s(2).a=2` 自动扩展报错；`{:}` 未验证 |
| 表格 `table` / `timetable` | ➖ | 非 v0.x 目标 |
| `classdef` OOP | 🟡 | 0.6.2 仅在 `SourceInput::Path` 请求下可用（`Vec2` 实测通过）；非 v0.x 目标（见 [roadmap.md](roadmap.md)） |
| Live Script / App Designer | ➖ | 非 v0.x 目标 |

## 二、数值与线性代数

| 函数 | 状态 | 备注 |
|---|---|---|
| `inv` / `det` / `eig` / `norm` / `cond` / `rank` / `trace` | ✅ | |
| `\` `mrdivide` 与最小二乘 | ✅ | `Am \ bm` 超定最小二乘已验证 |
| `magic` / `eye` / `ones` / `zeros` / `linspace` / `logspace` | ✅ | |
| `svd` / `lu` / `qr` / `chol` / `pinv` / `null` / `orth` | 🟡 | 内核对标表中存在，未逐项验证 |
| `expm` / `logm` / `sqrtm` | ⬜ | |
| 稀疏矩阵 | 🟡 | 类型存在（`SparseTensor`），脚本层未验证 |

## 三、信号处理

| 函数 | 状态 |
|---|---|
| `fft` / `ifft` / `fft2` / `ifft2` / `fftn` / `ifftn` | ✅（fft 已验证，其余 🟡） |
| `conv` / `filter` / `deconv` | 🟡（conv/filter 内核存在） |
| `hann` / `hamming` / `blackman` | 🟡（内核存在） |
| `kaiser` | ✅ `toolbox/signal/kaiser.m` |
| `spectrogram` / `pwelch` / `periodogram` | 🟡（内核存在） |
| `stft` / `istft` | ✅ `toolbox/signal/{stft,istft}.m`（重建误差 ~1e-16） |
| `butter` / `freqz` | 🟡（内核存在） |
| `cheby1` / `cheby2` / `ellip` / `designfilt` | ⬜（v0.2 未交付） |
| `xcorr` / `xcov` / `findpeaks` | ✅ `toolbox/signal/` |
| `convn` / `fir2` / `freqs` / `upfirdn` / `pspectrum` / `kaiserord` / `czt` / `goertzel` | ✅ `toolbox/signal/`（v0.2） |

> 逐函数状态与 v0.2 实现清单见 [toolboxes/signal.md](toolboxes/signal.md)。

## 四、统计与机器学习

| 函数 | 状态 |
|---|---|
| `mean` / `median` / `std` / `var` / `max` / `min` / `sum` / `prod` / `cumsum` | 🟡 |
| `corrcoef` / `cov` / `corr` | 🟡（内核存在） |
| `histcounts` / `histogram` | 🟡（histogram 绘制可用） |
| `rand` / `randn` / `randi` | ✅ |
| `kmeans` / `fitlm` / `fitctree` / `predict` | 🟡（内核存在） |
| `pca` / `normfit` / `ttest` / `vartest` / `iqr` / `moment` / `zscore` / `range` | ✅ `toolbox/stats/`（v0.1） |
| `chi2pdf` / `mle` / `anova1` / `anova2` / `kruskalwallis` / `glmfit` / `nlinfit` | ✅ `toolbox/stats_extra/` + `stats_pdf/`（v0.2） |
| `fitcsvm` / `fitcknn` / `crossval` | ⬜（依赖模型对象 / `predict`，v0.2 未交付） |

> 逐函数状态见 [toolboxes/statistics-ml.md](toolboxes/statistics-ml.md)。

## 五、优化

| 函数 | 状态 |
|---|---|
| `fminunc` / `fminbnd` / `fzero` / `fsolve` | 🟡（内核存在） |
| `lsqnonlin` / `lsqcurvefit` / `linprog` / `coneprog` | 🟡（内核存在） |
| `fminsearch` / `lsqnonneg` / `fmincon` / `lsqlin` / `intlinprog` | ✅ `toolbox/optim_*`（v0.2） |
| `quadprog` | ⬜（K13 阻塞；最小复现 `repro/k13-quadprog-pollution/`，canary `kernel_canaries`） |
| `ga` / `simulannealbnd` / `particleswarm` / `patternsearch` / `optimget` | ✅ `toolbox/optim_batch/` + `optim_extra/`（v0.2） |
| `optimoptions` / `optimset` | 🟡（内核存在） |

> 逐函数状态见 [toolboxes/optimization.md](toolboxes/optimization.md)。

## 六、控制系统

| 函数 | 状态 |
|---|---|
| `tf` / `ss` / `feedback` / `append` | 🟡（内核存在） |
| `step` / `impulse` / `stepinfo` / `bandwidth` | 🟡（内核存在） |
| `nyquist` / `rlocus` / `pzmap` | 🟡（内核存在，数值） |
| `pole` / `zero` / `dcgain` / `damp` / `isstable` | 🟡（内核存在） |
| `lqr` | 🟡（内核存在） |
| `bode` / `bodemag` / `margin` / `allmargin` / `nichols` / `lsim` / `initial` | ✅ `toolbox/control/` |
| `c2d` / `d2c` / `d2d` / `minreal` / `dlqr` / `kalman` | ✅ `toolbox/control/`（v0.2；`c2d` 支持 zoh/tustin） |
| `zpk` / `pid` / `pidstd` / `zpkdata` | ⬜（需内核 class） |
| `series` / `parallel` / `ss2tf` / `tf2ss` / `tf2zp` / `zp2tf` / `ss2zp` / `zp2ss` / `tfdata` / `ssdata` | ✅ `toolbox/control/` |

> 逐函数状态见 [toolboxes/control.md](toolboxes/control.md)。

## 七、绘图

| 能力 | 状态 | 备注 |
|---|---|---|
| `plot`（含线型/颜色如 `'r--'`） | ✅ | |
| `subplot` 网格 | ✅ | 桌面端按 `axes_index` 分组渲染 |
| `title` / `xlabel` / `ylabel` / `grid` / `legend` / `hold` | ✅ | |
| `stem` / `bar` / `stairs` / `area` | ✅ | |
| `scatter` / `histogram` | 🟡 | 渲染器已支持，脚本未验证 |
| `loglog` / `semilogx` / `semilogy` | ⬜ | |
| `surf` / `mesh` / `contour` / `imagesc` / `quiver` | ⬜ | 内核导出 schema 含这些 plot 类型，桌面渲染器未实现 |
| `plot3` / `scatter3` | ⬜ | 同上 |
| `xlim` / `ylim` / `axis` / `gca` / `gcf` / `close` | 🟡 | |
| `colorbar` / `colormap` | ⬜ | |
| 交互（缩放/平移/hover/图例开关） | ⬜ | 自研 SVG 渲染器无交互，见 [decisions.md](decisions.md) ADR-004 |
| 导出图片（`print` / `saveas` / `exportgraphics`） | ⬜ | |

## 八、IO 与系统

| 函数 | 状态 |
|---|---|
| `fprintf` / `sprintf` / `disp` / `display` | ✅ |
| `num2str` / `str2double` / `str2num` / `mat2str` | 🟡 |
| `strcat` / `strjoin` / `strsplit` / `strcmp` / `strcmpi` / `strncmp` | 🟡 |
| `who` / `whos` / `exist` / `clear` / `clc` | 🟡（`clear`/`vars` 宿主侧可用） |
| `load` / `save`（`.mat`） | ⬜ |
| `fopen` / `fclose` / `fread` / `fwrite` / `textscan` | ⬜ |
| `imread` / `imwrite` / `imshow` / `imfinfo` | 🟡（内核存在，v0.2 归入 Image Processing） |
| `dir` / `cd` / `pwd` / `path` | 🟡（`path`/`addpath`/`which` 内核存在） |
| `input`（交互输入） | ⬜（内核对标表存在 `stdin_events`，未验证） |
| `pause` / `tic` / `toc` / `profile` | ⬜ |
| `dbstop` / `dbstep`（调试器） | ➖ |

## 九、宿主层专有（MATLAB 无）

| 能力 | 说明 |
|---|---|
| 宿主命令 `vars` / `clear` / `figures` / `version` / `help` | 见 [host-commands.md](host-commands.md) |
| MCP 工具 `run` / `eval` / `plot` / `vars` / `reset` | 见 [mcp.md](mcp.md) |
| `help <topic>` 主题文档表 | 10 个主题，替代内核缺失的 `help` |

## 十、对标方法

1. **测试口径**：某项标 ✅ 的依据必须是 `demos/*.m` 或 `cargo test` 中的可执行断言，不是"内核里有这个函数"。
2. **内核覆盖探测**：用 `exist('<fn>')` 批量探测（`5`=builtin / `2`=`.m` 文件 / `0`=不存在），结果标 🟡（内核存在但未端到端验证）。v0.1 对 246 个 R2026a 目标函数探测 → 109 存在。见 [toolboxes/README.md](toolboxes/README.md)。
3. **推进方式**：每补一个函数/语法，同步在本清单改状态，并在 `demos/` 或集成测试里加用例。
4. **差距记录**：无法对齐或暂不做的，写入 [gaps.md](gaps.md) 并注明原因与计划版本。

## 十一、进度统计（v0.1 发布时）

| 状态 | 语言核心 | 数值 | 信号 | 优化 | 统计/ML | 控制 | 绘图 | IO |
|---|---|---|---|---|---|---|---|---|
| ✅ | 11 | 6 | 1 | 0 | 2 | 0 | 5 | 1 |
| 🟡 | 8 | 4 | 14 | 9 | 12 | 16 | 7 | 4 |
| ⬜ | 4 | 2 | 8 | 12 | 9 | 30 | 9 | 6 |
| ➖ | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |

> 目标（v0.3）：m-script 核心语法兼容 70-80%，内置函数 150+。
> 差距主要在 🟡（内核已有、未端到端验证）与 5 个工具箱的 ⬜ 缺口（见 [toolboxes/](toolboxes/)）；
> v0.2 优先把 🟡 批量验证固化，再补 ⬜。
