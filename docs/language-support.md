# m-script 语言支持范围

TensorLab 不自研解析器/求值器 —— 语言能力**全部来自 RunMat 0.6.2 内核**（兼容目标 R2026a）。
本文只描述"经我们验证可用"的范围，以及宿主层造成的差异。

> 验证口径：`demos/*.m`（5 个）+ `cargo test`（94 项）覆盖到的语法与函数。
> 未在本文出现的语法**不代表不支持**，而是**未经验证**。

## 一、已验证的语法

| 类别 | 已验证用法 | 出现位置 |
|---|---|---|
| 变量赋值 | `x = 2 + 3`、`y = x * 4` | hello_world |
| `;` 抑制回显 | `a = 1;` | 全部 demo |
| 矩阵字面量 | `[3 1 -1; 2 5 2; 1 -1 4]` | solve_demo |
| 矩阵/向量构造 | `magic(5)`、`eye(5)`、`ones(5,1)`、`zeros`、`linspace(0,4*pi,200)` | matrix_ops / plot_demo |
| 冒号范围 | `0:1/fs:1-1/fs`、`1:5`、`(0:999)`、`x(1:10:end)` | fft_demo / plot_demo |
| `end` 关键字（索引） | `x(2:end-1)`、`P(1:floor(length(P)/2)+1)` | fft_demo |
| 矩阵运算 | `*`、`\`、`'`（转置/共轭转置）、`A * Ai` | matrix_ops / solve_demo |
| 逐元素运算 | `.*`（`cos(x) .* exp(-x/6)`）、`.*`、`.^` | plot_demo |
| 函数调用与多返回 | `[peak_val, peak_idx] = max(P(2:end))` | fft_demo |
| 字符串/字符 | `'Tensor'`（char）、`"TensorLab"`（string）、`strcat('Tensor','Lab')` | hello_world |
| 格式化输出 | `fprintf('x = %d, y = %d\n', x, y)`（`%s`/`%d`/`%.1f`/`%.3f`/`%.6f`/`%.2e`） | 全部 demo |
| 显示 | `disp(A)` | matrix_ops / solve_demo |
| 绘图 | `figure`、`plot`、`subplot`、`stem`、`bar`、`title`、`xlabel`、`ylabel`、`grid on`、`legend`、`hold` | plot_demo / fft_demo |
| 内省 | `length`、`size`、`max`、`abs`、`floor`、`findobj('Type','figure')` | 多处 |
| 版本 | `version('-release')` → `R2026a` | host `version` 命令 |
| 注释 | `%` 整行与行尾；`%%` 段落 | 全部 demo |

## 二、明确的行为差异（宿主层造成）

以下与 MATLAB 不同，均由宿主层决定，不是内核问题：

### 1. 没有 `help` 内置函数

```
>> help fft          → 宿主主题文档表（覆盖 10 个主题）
>> help('fft')       → 同上
>> help              → 宿主命令总览
unknown_topic        → "No documentation for 'x'."
```

RunMat 0.6.2 内核**没有** `help`；直接让内核执行会触发深递归栈溢出。见 [decisions.md](decisions.md) ADR-008。

### 2. 字符串拼接不支持 `+`

```
>> "a" + "b"          → 错误（内核 0.6.2 的 + 只接受 numeric/logical）
>> strcat('Ten','sor') → 'Tensor'   ✅ 用这个
```

`'a' + 'b'` 在 MATLAB 语义里是**字符数组逐元素加法**（不是拼接）；`magic(4)` 的行列式为 0（真 MATLAB 同样拒绝 `inv(magic(4))`）。

### 3. 回显格式由内核决定

```
>> x = 42        →  x = 42          （单行，与 MATLAB 一致）
>> A = magic(3)  →  A =             （矩阵另起一行，与 MATLAB 一致）
                    <grid>
```

宿主**不**自行渲染回显（否则重复），因此 `format short/long` 等由内核控制。
见 [decisions.md](decisions.md) ADR-003。

### 4. REPL 区分宿主命令与 m 代码

`vars` 走宿主命令；`vars('-release')` 之后的行会走内核。判定规则见 [cli.md](cli.md)。

## 三、未验证 / 不确定

| 项目 | 状态 |
|---|---|
| `classdef` / OOP | 内核可能有部分实现；**不在 v0.x 目标内** |
| Cell / struct 的字面量构造 | 内核存在类型（`Value::Cell` / `Value::Struct`），但未经脚本验证 |
| 用户函数文件（`function` 定义） | 内核声明支持；本项目**未**做端到端验证 |
| `try/catch`、`switch`、`while`、`for` 循环 | 内核声明支持；**未**在 demo/测试中验证 |
| 复杂控制流的 JIT 路径 | 未知（宿主只读到 `usedJit` 标志） |
| `eval` / `feval` / 动态函数句柄 | 未验证 |
| 文件 IO（`load` / `save` / `fopen`） | 未验证 |
| 全局变量 `global` / `persistent` | 未验证 |

> 建议：v0.2 起为上述每一项补一个 `demos/` 或集成测试用例，把"未验证"逐个变成"已验证"或"已知不支持"。
> 见 [gaps.md](gaps.md)。
