# CLI 参考（`tensorlab`）

二进制：`target/debug/tensorlab`（Windows 为 `.exe`），来自 `apps/tl-cli`。

## 用法

```
tensorlab                     进入交互式 REPL
tensorlab run <script.m>      执行脚本文件
tensorlab bench               JIT/性能基准（重复执行以进入 JIT 层）
tensorlab help | --help | -h  列出宿主命令
tensorlab version | --version 打印版本
```

未知子命令 → 退出码 1，stderr 输出 `unknown command '<x>'. usage: tensorlab [run <script.m> | bench | help | version]`。

## `run` 子命令

```bash
tensorlab run demos/fft_demo.m
```

- 脚本内容作为一次 `Engine::run` 执行
- 控制台输出 → **stdout**
- `[VM 27.0 ms]` / `[JIT ...]` 状态行 → **stderr**（便于 `tensorlab run x.m > out.txt` 只取纯输出）
- 出现运行时错误：错误消息 → stderr，**退出码 1**
- 文件不存在：退出码非 0
- 缺少 `<script.m>` 参数：stderr 输出 `usage: tensorlab run <script.m>`，退出码非 0

### 退出码

| 码 | 含义 |
|---|---|
| 0 | 成功（含脚本无输出、REPL 正常退出） |
| 1 | 运行时错误 / 文件不存在 / 用法错误 / 未知子命令 |

## `bench` 子命令

```bash
tensorlab bench            # JIT on
tensorlab bench --no-jit   # 解释器-only 基线
```

在同一个 `Engine` 会话内重复执行固定 workload：前 10 次预热（触发 RunMat tiering），
后 5 次取中位数作为 steady-state；结束后打印内核统计
（`jit_compiled` / `interpreter_fallback` / `osr` / `vectorized_regions`）。
输出示例（Windows debug 构建）：

```
workload             cold_ms  steady_ms    tier  speedup
script_loop_5k         1297.0      315.0     JIT     4.12x
function_calls_100     1553.0     1514.0     JIT     1.03x
matmul_300              564.0      378.0     JIT     1.49x
fft_16k                 165.0       74.0     JIT     2.23x
vector_math_100k        292.0      154.0     JIT     1.90x

注：JIT 采用分层编译，前 10 次为预热；steady_ms 取最后 5 次的中位数。
内核统计：executions=77 jit_compiled=30 interpreter_fallback=47 osr=0 vectorized_regions=20400
```

循环 workload 的上界带 `rand()` 依赖（`n = 5000 + (rand() > 2)`），
避免 MATLAB 侧把常量循环整体折叠掉；两侧使用同一写法。

MATLAB 对比脚本见 [../bench/matlab_bench.m](../bench/matlab_bench.m)，方法与基线见
[../docs/jit-benchmarks.md](jit-benchmarks.md)。

## REPL

```bash
$ tensorlab
TensorLab 0.2.1 (kernel: RunMat 0.6.2) — type 'help' for host commands
>> x = 42
x = 42
>> vars
  NAME           SIZE            BYTES  CLASS
  ------------  ---------------  --------------- 
  x             1x1                  8  double
[VM 12.3 ms]
>>
```

行为：

- 提示符 `>> `
- 输入按行处理：**宿主命令**（`vars` / `clear` / `help` / `figures` / `version`）优先匹配，
  其余交给内核执行（见 [architecture.md](architecture.md) 的路由规则）
- `exit` / `quit` / `exit()` / `quit()` 退出；**stdin EOF（Ctrl+Z / Ctrl+D）也正常退出**（退出码 0）
- 某行出错不终止会话，可继续输入
- 空行忽略
- `help <topic>` 由宿主层拦截（内核 0.6.2 无 `help` 内置函数，见 [decisions.md](decisions.md) ADR-008）

### 路由规则

`tl-commands::repl_line` 判定顺序：

1. 空行 → 无输出
2. 以 `help` 开头且后面有主题（非 `commands`）→ 宿主 `help_topic`
3. 行内**不含** `=` / `(` / `;` 且不以 `'` 结尾 → 尝试宿主命令注册表
4. 否则 → 交给内核执行 m 代码

因此 `version` 走宿主命令，而 `version('-release')` 走内核。

## 示例

```bash
# 跑全部验收 demo（应全部退出码 0）
for d in hello_world matrix_ops plot_demo fft_demo solve_demo; do
  tensorlab run demos/$d.m > /dev/null || echo "FAILED: $d"
done

# REPL 单行执行
echo "disp(magic(3))" | tensorlab
```
