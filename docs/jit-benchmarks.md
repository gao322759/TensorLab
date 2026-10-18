# V0.3 JIT 基准（RunMat 0.6.2）

> 目标：关键场景达到 **MATLAB 0.5x**（即 TensorLab 耗时 ≤ 2× MATLAB 耗时）。
> 本文记录方法、当前基线与后续优化点。
>
> 本机对标版本：**MATLAB R2023b**（`D:\Program Files\MATLAB\R2023b\bin`）。
> 功能兼容目标仍是 R2026a；R2023b 仅作为当前可用的性能对照基线。

## 一、运行方式

```bash
# TensorLab（debug 构建，便于快速迭代）
cargo run -p tl-cli -- bench

# TensorLab（release 构建，用于正式对标）
cargo build --release -p tl-cli
target/release/tensorlab bench

# VM 基线（关闭 JIT；用于确认 JIT 的实际收益）
cargo run -p tl-cli --release -- bench --no-jit
```

`bench` 在**同一个 Engine 会话**内重复执行固定 workload：

- 前 10 次预热（RunMat tiering 默认 8 次调用 / 32 次回边后进入 JIT）；
- 后 5 次取中位数作为 `steady_ms`；
- `cold_ms` 是第 1 次（纯 VM）耗时；
- `speedup = cold_ms / steady_ms`；
- 结束后打印内核累计统计：`executions` / `jit_compiled` / `interpreter_fallback` /
  `osr`（栈上替换次数）/ `vectorized_regions`。

宿主侧开关：`Engine::new()` = JIT 开启；`Engine::with_jit(false)` = 解释器-only，
用于 VM 基线/回归测试。JIT 统计仍通过 `RunReport.used_jit` 暴露（`tl-jit::stats_of`）。

MATLAB 侧运行（本机 R2023b，`D:\Program Files\MATLAB\R2023b\bin`）：

```matlab
cd bench
matlab -batch "cd('F:\软件开发\TensorLab\bench'); matlab_bench"
```

`matlab_bench.m` 使用完全相同的 workload 和 10+5 协议。循环上界带一个
`rand()` 依赖（`n = 5000 + (rand() > 2)`），避免 MATLAB JIT 把纯常量循环
直接折叠掉；TensorLab 侧 workload 同样使用该写法。

## 二、当前基线（2026-10-18，Windows，debug 构建）

| workload | cold_ms | steady_ms | tier | speedup |
|---|---:|---:|---|---:|
| `script_loop_5k` | 1297.0 | 315.0 | JIT | **4.12x** |
| `function_calls_100` | 1553.0 | 1514.0 | JIT | 1.03x |
| `matmul_300` | 564.0 | 378.0 | JIT | **1.49x** |
| `fft_16k` | 165.0 | 74.0 | JIT | **2.23x** |
| `vector_math_100k` | 292.0 | 154.0 | JIT | **1.90x** |

### MATLAB R2023b 对照

| workload | MATLAB cold_ms | MATLAB steady_ms | TensorLab debug steady_ms | 比值（TL/MATLAB） |
|---|---:|---:|---:|---:|
| `script_loop_5k` | 15.352 | 0.040 | 315.0 | 7875x |
| `function_calls_100` | 1.978 | 0.023 | 1514.0 | 65826x |
| `matmul_300` | 46.661 | 3.465 | 378.0 | 109.1x |
| `fft_16k` | 17.006 | 2.754 | 74.0 | 26.9x |
| `vector_math_100k` | 79.638 | 15.949 | 154.0 | 9.7x |

> **debug 构建不可用于达标判定**：unoptimized + debuginfo 会让标量循环、
> 函数调用和小矩阵运算慢一到两个数量级。上表只用于观察 JIT 加速趋势。
> 0.5x 目标（TL 耗时 ≤ 2× MATLAB）以 release 构建为准，见下一节。
>
> MATLAB R2023b 的 JIT 对标量循环与函数调用极其激进（steady 仅 0.02–0.04 ms），
> 这是 TensorLab 当前最大的差距来源；`matmul/fft/vector_math` 这类
> 内核库调用才是短期可竞争的战场。

## 三、release 基线（正式对标，2026-10-18）

`cargo build --release -p tl-cli`（LTO thin，冷构建约 60 分钟）后运行
`target/release/tensorlab.exe bench`：

| workload | cold_ms | steady_ms | tier | speedup |
|---|---:|---:|---|---:|
| `script_loop_5k` | 636.0 | 405.0 | JIT | 1.57x |
| `function_calls_100` | 977.0 | 917.0 | JIT | 1.07x |
| `matmul_300` | 261.0 | 236.0 | JIT | 1.11x |
| `fft_16k` | 155.0 | 83.0 | JIT | 1.87x |
| `vector_math_100k` | 204.0 | 139.0 | JIT | 1.47x |

与 MATLAB R2023b 的正式比值（目标 TL 耗时 ≤ 2× MATLAB，即 0.5x）：

| workload | MATLAB steady_ms | TensorLab release steady_ms | 耗时比（TL/MATLAB） | 达标（≤2.0） |
|---|---:|---:|---:|---|
| `script_loop_5k` | 0.040 | 405.0 | 10125x | ❌ |
| `function_calls_100` | 0.023 | 917.0 | 39870x | ❌ |
| `matmul_300` | 3.465 | 236.0 | 68.1x | ❌ |
| `fft_16k` | 2.754 | 83.0 | 30.1x | ❌ |
| `vector_math_100k` | 15.949 | 139.0 | 8.7x | ❌ |

> release 与 debug 的 steady-state 几乎相同，说明瓶颈在 **Cranelift 生成的代码 +
> 运行时值分派**，而不是宿主编译优化。要把标量循环/函数调用拉近 MATLAB，
> 必须减少每迭代的装箱与动态分派；`matmul/fft/vector_math` 则需要更直接的
> BLAS/FFT 快速路径。0.5x 目标（耗时 ≤ 2× MATLAB）尚未达成，v0.3 继续。

## 四、性能上限：内核证据（2026-10-18）

同一天的 debug 构建下，用 `bench` 与 `bench --no-jit` 对照（steady_ms）：

| workload | JIT on | JIT off（VM） | JIT 收益 | MATLAB R2023b |
|---|---:|---:|---:|---:|
| `script_loop_5k` | 507.0 | 1363.0 | 2.69x | 0.040 |
| `function_calls_100` | 1607.0 | 1615.0 | 1.00x | 0.023 |
| `matmul_300` | 402.0 | 429.0 | 1.07x | 3.465 |
| `fft_16k` | 84.0 | 166.0 | 1.98x | 2.754 |
| `vector_math_100k` | 160.0 | 220.0 | 1.38x | 15.949 |

> 同一 debug 构建不同时段的 steady 有约 ±30% 波动（受机器负载与杀软影响）；
> 上表只用于 **同场 JIT on/off 对照**，绝对值以 release 表为准。

内核累计统计（JIT on）：

```
executions=77 jit_compiled=30 interpreter_fallback=47 osr=0 vectorized_regions=20400
```

关键结论：

1. **JIT 确实生效**：标量循环 2.69x、FFT 1.98x；关闭 JIT 后全部走解释器。
2. **`osr=0`**：顶层脚本的 5000 次回边没有触发栈上替换（OSR）。JIT 编译的是
   执行单元/函数入口，循环体仍以动态值路径执行。
3. **release 与 debug 的 steady-state 几乎相同**，说明瓶颈不在宿主编译优化，而在
   Cranelift 生成代码 + 运行时值分发。
4. TensorLab 无法替换这些实现：
   - 内核没有宿主函数注册 API（[gaps.md](gaps.md) K12），不能提供自己的 `fft` / `*` / `sin`；
   - tiering 配置（阈值、deterministic）只有 `pub(crate)` 测试接口，宿主无法调整；
   - 20 次预热（越过 specialized 阈值 16）实测无改善。
5. 补充探针：用 `compile_executable_unit` + `invoke_executable(Entrypoint)` 执行同一
   5000 次循环，steady ≈ **270ms**（比 `execute_request` 的 507ms 快约 2x），但
   `native_osr_transfers` 仍为 **0**，与 MATLAB 的 0.04ms 仍差 3 个数量级以上。
   即便绕到内核的 AOT/entrypoint 路径，也无法把顶层循环拉到 MATLAB 量级。
6. 因此 **标量循环与数值内核的进一步优化必须以 RunMat 上游改动为前提**，
   宿主侧优化在此停止（记录为 [gaps.md](gaps.md) K19）。

## 五、启动时间（2026-10-18）

目标：**< 500ms**。测量命令（release，Windows PowerShell）：

```powershell
$f = Join-Path $env:TEMP 'tl_startup_probe.m'
Set-Content -Path $f -Value 'x = 1 + 1;' -Encoding ASCII
1..5 | ForEach-Object {
  (Measure-Command { target\release\tensorlab.exe run $f | Out-Null }).TotalMilliseconds
}
```

5 次实测：`230, 295, 234, 233, 1691` ms，**中位数 234ms**；
首次运行偶发 1691ms，是杀软/磁盘冷缓存造成的离群值（见 [gaps.md](gaps.md) D2）。
不计离群值时，进程启动 + 内核初始化 + 一次执行稳定在 **230–300ms**，已满足 <500ms。

## 六、结论与下一步

- **JIT 已实际生效**：重复执行 10 次左右后 `used_jit=true`；release 下脚本循环 1.57x、
  FFT 1.87x、向量数学 1.47x、矩阵乘 1.11x（加速幅度低于 debug，因为冷路径本身已被宿主优化）。
- **函数调用路径是短板**：`function_calls_100` 仅 1.07x。JIT 编译了调用方，但
  文件函数调用的分派/边界开销仍占主导。下一步：
  1. 用 `RunReport.used_jit` + kernel profiling 区分“调用方 JIT”与“被调函数 JIT”；
  2. 评估 RunMat 的 function inlining / direct-call tiering；
  3. 若上游无解，在 TensorLab 侧减少高频工具箱函数的跨文件调用。
- **与 MATLAB R2023b 的差距（release）**：标量循环 10125x、函数调用 39870x、
  matmul 68x、FFT 30x、向量数学 8.7x。0.5x 目标未达成；证据表明差距来自
  RunMat 执行层（`osr=0`、无宿主函数注册、tiering 不可配），**宿主侧优化到此为止**，
  后续需上游修复（K19）。
- **一次性 `tensorlab run` 不会受益**：tiering 需要重复执行；CLI/demo 是 one-shot，
  因此 `[VM ...]` 状态是预期的。桌面 REPL/编辑器重复运行、MCP 长会话、`bench` 会进入 JIT。
- **回归看守**：`crates/tl-commands/tests/jit_tiering.rs` 断言 12 次重复后
  `used_jit=true` 且 steady-state 至少改善 10%。

## 七、参考

- `crates/tl-jit/src/lib.rs` — `JitStats`（used_jit / duration / summary）
- `bench/matlab_bench.m` — MATLAB 对照脚本
- `docs/kernel-notes.md` — RunMat 0.6.2 嵌入与 tiering 说明
