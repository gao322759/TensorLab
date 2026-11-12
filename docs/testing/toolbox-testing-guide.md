# Toolbox 白盒测试指南（MATLAB 对齐）

> 适用范围：`toolbox/` 下全部自写 m 函数（不含 RunMat 内置函数）。
> 本指南沉淀自 2026-10-18 全量对齐工程（103 函数 × 7 用例 = 721 例，FAIL 77 → 0），
> 是后续所有 toolbox 函数开发的**强制规范**（已写入 `AGENTS.md` 改动检查清单）。

## 0. 政策：强制白盒测试

1. **每个新增/修改的 `toolbox/*.m` 函数必须同步提交用例**：3 正常 + 2 临界 + 2 异常，写入对应分类的 `tests/toolbox_cases/<category>.json`。
2. 用例必须与 MATLAB 对比数值（`matlab: true`）；MATLAB 无对应实现时才允许 `matlab: false`（判定 SKIP）。
3. 合入标准：`node tests/toolbox_compare.mjs --category <分类>` 无新增 FAIL；全量运行无新增 FAIL。
4. 行为变化导致 Rust 对抗性测试断言失效时，**先确认新行为与 MATLAB 一致，再更新断言**——不把错误行为写进断言。
5. 语义有争议时：先黑盒实验取证（§6 方法论），再决定修复或在报告中记录差异。

## 1. 测试基础设施

```
tests/toolbox_cases/*.json         用例清单（每函数 3+2+2，五个分类文件）
tests/toolbox_compare.mjs          运行器
docs/testing/toolbox-report.md     每次运行整体重新生成的报告
target/toolbox-tests/results.json  程序化查询用结果（逐例 verdict/detail/误差）
```

工作原理：runner 把 JSON 清单编译成两份 m 脚本——TensorLab 侧**按函数拆分**（隔离死循环/崩溃），MATLAB 侧单脚本（`-batch`）。两进程各自以 `CASE/BEGIN/D/V/END` 多行协议输出每个变量的形状与数值（`%.17g` 保精度），runner 逐例计算 `max_abs/max_rel` 并与 `tol` 比较。

```bash
node tests/toolbox_compare.mjs                     # 全量（TensorLab + MATLAB R2023b）
node tests/toolbox_compare.mjs --category signal   # 单分类（日常开发用这个）
node tests/toolbox_compare.mjs --no-matlab         # 只跑 TensorLab（快速冒烟）
node tests/toolbox_compare.mjs --timeout 120       # 单函数超时秒数（默认 120）
```

判定口径：

| 判定 | 含义 | 处置 |
|---|---|---|
| PASS | 双端数值差在 tol 内 | — |
| FAIL | TensorLab 报错 / 超 tol / 崩溃 / 超时 | **必须修** |
| WARN | 非缺陷差异：双端均报错、形状不同但数值一致、越界输入被接受未报错 | 逐例评估 |
| KNOWN | RunMat 内核限制（K2 标量切片 / K21 logical 运算），非算法问题 | 记录；内核升级后复测 |
| SKIP | MATLAB 无对应实现 / 未授权工具箱 | 记录 |

> 报告每次运行会被 runner **整体重写**，「结论与主要问题」段需手动重插；经验与结论的 canonical 版本维护在本指南。

## 2. 新函数接入流程

1. 实现函数。文件名**全小写**（RunMat 不识别首字母大写的函数名）。
2. 在分类 JSON 添加条目：

```json
{
  "name": "myfunc",
  "setup": "x=[1 2 3;4 5 6];",
  "compare": ["y"],
  "matlab": true,
  "tol": 1e-9,
  "normal":   ["y=myfunc(x,1);", "y=myfunc(x,2);", "y=myfunc(x,3);"],
  "boundary": ["y=myfunc(x,0);", "y=myfunc(ones(100,100),1);"],
  "abnormal": ["y=myfunc(x,-1);", "y=myfunc([],1);"],
  "note": "可选：已知差异/边界说明"
}
```

3. `node tests/toolbox_compare.mjs --category <分类>`。
4. 修复 FAIL：**优先怀疑语义差异（§5 经验库），其次才是数值精度**；WARN 逐例判断是否需对齐。
5. `cargo test -p tl-commands`：对抗性测试（`crates/tl-commands/tests/adversarial.rs`）同步维护。
6. 全量 `node tests/toolbox_compare.mjs` + 重插报告结论段。

tol 经验值：

| 场景 | tol | 例 |
|---|---|---|
| 二值/整数输出 | 0 | edge |
| 解析算法（可逐位一致） | 1e-12 | convn、wiener2 |
| 常规数值 | 1e-9 ~ 1e-10 | czt、dct |
| IIR 滤波 / 重采样（系数链敏感） | 1e-6 | decimate、imresize |
| 迭代优化 / 频谱栅格 | 1e-4 ~ 1e-3 | imadjust、fir2 |

## 3. 用例设计规范

- **防作弊**：每函数至少两组不同输入产生不同输出（JSON 层 + Rust `anti_cheat_two_inputs_change_outputs` 双保险）；不接受"有输出即通过"，不给可被硬编码常量满足的断言。
- 正常值覆盖主路径与可选参数分支（method / direction / shape 等）。
- 临界值 = 定义域边界（0、1、空、全同值、单位尺寸、Inf）；异常值 = 越界与类型错误，断言**报错行为**而非逐字消息（错误文案不必与 MATLAB 一致，identifier 语义对齐即可）。
- 多输出函数把副输出（如 edge 的 thresh）也列入 compare。
- 已知不可对齐点（编译库噪声等）在 `note` 记录，并把用例参数移到非敏感点（例：imrotate 30°→60°，见 §7）。

## 4. RunMat 限制与 m 语言陷阱（写 toolbox 函数必读）

| 陷阱 | 症状 | 规避 |
|---|---|---|
| K2 标量切片 | `Slicing only supported on tensors`（对标量做 `x(:)`） | `reshape(x, numel(x), 1)` |
| K21 logical 参与数值运算 | `... to f64` 类型转换错误 | 先 `double(...)` |
| 大写函数名 | `does not define` | 文件名全小写 |
| 分支内首次赋值 | `local may be read before assignment on some control-flow paths` | **所有局部变量在 if/switch 前预初始化**。注意：一个文件触发会让同批挂载的函数调用集体报错（症状指向被调函数，排查时先看最近改过的文件） |
| 列向量范围索引 | 切片结果变行向量 | 切片后 `reshape(...)` 回列 |
| 3-D 标量切片赋值 | panic | 先建 1×N 向量再 `reshape(..., 1, 1, N)` |
| 深递归 | 栈溢出 | Engine 已用 64 MiB 大栈线程承载；函数内仍避免无界递归 |

## 5. MATLAB 语义对齐经验库

本轮全部修复项的浓缩。**新函数开发前先查本表**——同类语义大概率复现。

### 5.1 signal

| 函数 | 关键语义 |
|---|---|
| `decimate` | Chebyshev I 默认 8 阶 / 0.05 dB / 0.8/R，但阶数需按通带边缘容差**逐阶降阶**（`abs(H_db(0.8/R)+0.05) > 1e-6` 时 nfilt−1）；取样起点 `nbeg = r - (r*ceil(N/r) - N)`（不是从 r 开始）；filtfilt 奇延拓 + 稳态初值 |
| `fir2` | 频率栅格 `f_k = 2k/nfft`（Nyquist=1）；ifft 取实部后**循环移位**；Hamming 窗 |
| `xcorr` / `xcov` | 不等长先补零到 `max(M,N)`，输出 `2*max-1`，方向随输入 |
| `dct` / `idct` | 结果保持输入行/列方向；切片后 reshape 回列 |
| `czt` | `z_k = a * w^-(k-1)`（W 指数为**负**） |

### 5.2 image

| 函数 | 关键语义 |
|---|---|
| `imresize` | contributions 算法五要素：`u = i/s + 0.5*(1-1/s)`；`left = floor(u - kw/2)`；`P = ceil(kw)+2` 抽头；权重**按行归一化**；越界索引用周期 2N 对称镜像 `aux = [1:N, N:-1:1]`。缩小且非 nearest 时核宽 ×(1/s) 抗混叠；输出尺寸 **ceil**（非 round） |
| `edge` | Sobel/Prewitt 梯度需归一化（/8、/6）+ replicate 边界；强度 `b = bx²+by²`（在平方域比较）；自动 cutoff = scale·mean(b)（sobel/prewitt scale=4、roberts=6），显式阈值取其平方；NMS 沿量化梯度方向，邻点比较容差 100·eps |
| `wiener2` | 局部统计**零填充**（不是对称填充）；噪声功率 = 全部局部方差的均值；`f = max(0, (va-noise)/va)` |
| `medfilt2` | 偶数窗锚定 `i..i+M-1`；越界零填充 |
| `ordfilt2` | 越界零填充（MATLAB 默认） |
| `mat2gray` | 输出裁剪到 [0,1]；`hi==lo` 返回全 1 |
| `rgb2ycbcr` / `ycbcr2rgb` | **studio swing**（16/255 偏移）；反变换裁剪 |
| `imgaussfilt` | replicate 边界；sigma≤0 报错 |
| `imclose` / `imopen` | 先按 SE 半径补零，膨胀→腐蚀，最后裁剪 |
| `imtranslate` | 亚像素双线性，零填充 |
| `label2rgb` | MATLAB jet 色图、白背景、uint8 输出 |
| `imcrop` | `x1=ceil(xmin)`、`x2=floor(xmin+w)`（像素中心语义） |
| `imrotate` | 画布 ceil(旋转包围盒)；最近邻 `floor(x+0.5)`；平局像素差异属编译库噪声（§7） |

### 5.3 control / stats

| 函数 | 关键语义 |
|---|---|
| `kalman` | R≤0 报错（不静默返回） |
| `lsim` | RK4 + 输入线性插值；TF 模型忽略 x0 |
| `nichols` | `mag = abs(h)`（绝对幅值） |
| `freqresp` / `bode` | 返回 **1×1×N**（三维） |
| `zp2tf` | num 补前导零至 den 长度 |
| `pca` | 最多 `min(N-1, P)` 个主成分 |
| `anova2` | 输出 `[p_columns p_rows]`；REPS=1 只 2 个 |
| `nlinfit` | Levenberg-Marquardt；模型输出长度 ≠ y 报错 |

### 5.4 通用模式

- **边界填充没有统一规则**：replicate（imgaussfilt/edge）、零填充（wiener2/ordfilt2/medfilt2/imclose）、对称镜像（imresize 索引镜像）——各函数以实测为准。写函数前先用小矩阵在 MATLAB 探边界行为，不要按"合理"猜。
- 默认参数常带历史包袱（studio swing、threshold scale=4/6、Keys 核 a=−0.5、8 阶降阶）——同样以实测为准。
- 输出形状约定优先于数值（行/列方向、三维性、输出 class）——形状不对直接 FAIL，数值再准也没用。

## 6. 差异排查方法论（黑盒探针技术）

遇到 FAIL 且原因不明时，按以下顺序排查：

1. **看差模式**：系统性偏移/大面积差异 → 查坐标约定、旋转中心、边界规则；孤立 tie 像素（坐标恰为 .5）→ 浮点平局问题（§7）。
2. **双引擎切换**：MATLAB 常有编译加速路径，可用输入类型或偏好切换对比：
   - `imrotate`：double 走 IPP 编译核；int32 输入或 `iptsetpref('UseIPPL', false)` 走 imwarp 纯 m 路径。
   - 两条官方路径输出不同 = MathWorks 自己都不保证逐位一致，差异大概率是库噪声。
3. **坐标探针**：让插值引擎"交代"连续源坐标——

```matlab
% R 通道=行号、G 通道=列号；bilinear 的输出值就是连续源坐标
R = repmat((1:m)', 1, n);  G = repmat(1:n, m, 1);
O = imrotate(double(cat(3, R, G, zeros(m, n))), 30, 'bilinear');
src_row = O(i, j, 1);  src_col = O(i, j, 2);   % 精确到 ulp
```

4. **常数扰动网格搜索**（可辨识性分析）：把嫌疑常数（如三角函数值）在 ±k ulp 内网格扰动，看是否存在一组值能复刻目标输出。无解 → 结构性差异；多解（imrotate 30° 有 119 组等效偏移）→ 不可辨识，拟合无意义。
5. **纯数学不可能性证明**：若某平局坐标在任何运算顺序（含 FMA）下都精确（如 `0*ct + (-1)*0.5 + 2 ≡ 1.5`），而目标行为不同 → 对方用了不同的常数，不是舍入规则差异。
6. **clean-room 边界**：默认黑盒反推。读 MATLAB 源码需项目所有者对**具体函数**明确豁免，且先只读分析、不改代码（本轮先例：decimate / imresize / imrotate / edge 四函数）。算法实现必须能追溯到教科书 / 公开论文。

## 7. 何时停止追差异（不可复刻判据）

同时满足以下条件即判定"编译库浮点噪声"，记录后关闭，不再投入：

1. 差异只出现在浮点平局点（坐标恰为 .5）；
2. MATLAB 双引擎（IPP vs imwarp）自身在该点不一致；
3. 常数扰动网格搜索无解或解严重不唯一；
4. MathWorks 文档声明跨后端/硬件结果可能不同。

处置：实现保持**精确几何 + 文档化舍入规则**（不拟合噪声）；用例参数避开噪声敏感点；报告记录完整分析链。范例：`toolbox-report.md`「剩余差异」节的 imrotate 条目——差异定性（IPP 坐标链带 ±1~2 ulp 噪声 vs 我们精确）、可修正性评估（四处证据）、处置（30°→60°）。

## 8. 基础设施已知行为

- 杀毒软件锁定新构建 exe（EPERM）→ runner 已内建 3 次重试，无需干预。
- MATLAB 控制台输出为 GBK → runner 已按 gbk 解码；探针脚本错误打 `err.identifier`（英文），不打本地化消息。
- MATLAB 探针脚本技巧：cell 逐条 try/catch；局部函数必须放文件末尾；`[m,n]=size(...)` 不能对 cell 数组解构。
- 单函数超时/崩溃 → runner 自动逐用例重跑定位（输出 H/C 标记），结果仍进报告。
- 查询结果用 `target/toolbox-tests/results.json`（PowerShell: `ConvertFrom-Json` 过滤 `verdict -eq 'FAIL'`）。
