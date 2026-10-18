# 优化工具箱（Optimization）

> 对标 **MATLAB R2026a — Optimization Toolbox**（含部分 Global Optimization Toolbox）。v0.2 目标：核心 20–30 函数可用。
> `category` = `"optimization"`。

## 一、范围与目标

涵盖：无约束/有约束非线性、线性/二次规划、最小二乘、方程求根、全局优化（启发式）。

**不做**（➖）：`optimproblem`/`optimvar` 的建模框架（依赖 OOP）、`ga` 的并行/混合整数完整版、自动微分、C 代码生成。

## 二、函数清单

### 无约束非线性 / 求根

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `fminunc` | `x = fminunc(fun,x0[,options])` | 无约束（拟牛顿/信赖域） | ✅ | ✅ 内核 |
| `fminsearch` | `x = fminsearch(fun,x0[,options])` | Nelder–Mead 单纯形 | ✅ | ✅ `toolbox/optimization/fminsearch.m`（要求 `numel(x0)≥2`，1-D 用 `fminbnd`） |
| `fminbnd` | `x = fminbnd(fun,x1,x2)` | 有界一维标量最小化 | ✅ | ✅ 内核 |
| `fzero` | `x = fzero(fun,x0)` | 标量方程求根（Brent） | ✅ | ✅ 内核 |
| `fsolve` | `x = fsolve(fun,x0[,options])` | 非线性方程组 | ✅ | ✅ 内核 |
| `lsqnonlin` | `x = lsqnonlin(fun,x0[,lb,ub])` | 非线性最小二乘 | ✅ | ✅ 内核 |
| `lsqcurvefit` | `x = lsqcurvefit(fun,x0,xdata,ydata)` | 曲线拟合 | ✅ | ✅ 内核 |
| `lsqnonneg` | `x = lsqnonneg(C,d)` | 非负最小二乘 | ✅ | ✅ `toolbox/optimization/lsqnonneg.m`（Lawson–Hanson） |

### 约束非线性

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `fmincon` | `x = fmincon(fun,x0,A,b,Aeq,beq,lb,ub,nonlcon)` | 约束（SQP/内点） | ✅ | ✅ `toolbox/optim_fmincon/fmincon.m`（外罚函数 + Hooke–Jeeves；线性约束/边界；`nonlcon` 未支持） |
| `fminimax` | `x = fminimax(fun,x0)` | 极小化最大 | ✅ | ➖ |
| `fgoalattain` | `x = fgoalattain(fun,x0,goal,weight)` | 目标达到 | ✅ | ➖ |
| `fseminf` | `x = fseminf(fun,x0,...)` | 半无限约束 | ✅ | ➖ |

### 线性 / 二次 / 整数规划

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `linprog` | `x = linprog(f,A,b,Aeq,beq,lb,ub)` | 线性规划（内点/单纯形） | ✅ | ✅ 内核 |
| `quadprog` | `x = quadprog(H,f,A,b,Aeq,beq)` | 二次规划（有效集） | ✅ | ⬜ **被内核 bug 阻塞**（见 [gaps.md](../gaps.md) K13：实现文件会污染 `fminsearch` 的编译元数据） |
| `lsqlin` | `x = lsqlin(C,d,A,b,...)` | 有约束线性最小二乘 | ✅ | ✅ `toolbox/optim_lsqlin/lsqlin.m`（有效集 + KKT） |
| `intlinprog` | `x = intlinprog(f,intcon,A,b,...)` | 混合整数线性规划 | ✅ | ✅ `toolbox/optim_batch/intlinprog.m`（分支定界 + 内核 `linprog`） |
| `coneprog` | `x = coneprog(f,socConstraints)` | 二阶锥规划 | ✅ | ✅ 内核 |
| `mldivide` 基座 | `x = A\b` | 线性方程组 | ✅ | ✅ 内核（CLI/demo 验证） |

### 全局优化（启发式）

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `ga` | `x = ga(fun,nvars[,A,b,...])` | 遗传算法 | ✅ | ✅ `toolbox/optim_batch/ga.m`（实数编码、锦标赛、blend 交叉、精英保留） |
| `simulannealbnd` | `x = simulannealbnd(fun,x0,lb,ub)` | 模拟退火 | ✅ | ✅ `toolbox/optim_batch/simulannealbnd.m` |
| `particleswarm` | `x = particleswarm(fun,nvars,lb,ub)` | 粒子群 | ✅ | ✅ `toolbox/optim_batch/particleswarm.m` |
| `patternsearch` | `x = patternsearch(fun,x0)` | 模式搜索 | ✅ | ✅ `toolbox/optim_batch/patternsearch.m` |
| `gamultiobj` / `paretosearch` | 多目标 | ✅ | ➖ |

### 选项

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `optimoptions` / `optimset` | `opts = optimoptions(solver,...)` | 求解器选项 | ✅ | ✅ 内核 |
| `optimget` | `val = optimget(opts,name)` | 读选项 | ✅ | ✅ `toolbox/optim_extra/optimget.m`（支持常用字段） |

## 三、v0.2 实现清单（缺口）

v0.2 已交付：`fminsearch`、`lsqnonneg`、`fmincon`、`lsqlin`、`intlinprog`、`ga`、
`simulannealbnd`、`particleswarm`、`patternsearch`、`optimget`（总计 20 个可用函数）。

仍缺（诚实清单）：

1. **`quadprog`**：实现与数值结果均已验证，但受 [K13](../gaps.md) 目录级解析污染阻塞，暂不交付。
   最小复现（一个目录 + 两个 `.m`）与上游 issue 草稿见 [../../repro/README.md](../../repro/README.md)；
   触发条件已定位为同目录 3 个语义函数定义。canary：
   `cargo test -p tl-kernel --test kernel_canaries k13_quadprog_pollutes_same_directory_fminsearch`。
2. **`fmincon` 的 `nonlcon`**：非线性约束需要把 `[c,ceq]` 回调进惩罚项。v0.2 记录的
   "工具箱函数内闭包丢环境"（kernel-notes 坑 27 / K15）在 RunMat 0.6.2 上**已不复现**
   （canary `k15_closure_keeps_captured_environment`）；当前实现仍只支持线性约束与边界，
   下一步可恢复闭包写法并补齐 `nonlcon`。
3. **不做**：`fminimax`、`fgoalattain`、`fseminf`、`gamultiobj`、`paretosearch`、`optimvar`/`optimproblem`。
4. **`intlinprog` 的不可行约束**：P1 对抗性测试发现，不可行 LP 松弛/上下界冲突时可能长时间不返回
   （`crates/tl-commands/tests/adversarial.rs` 因此只测合法分支与非法 `intcon`）。需要补
   infeasible 早退/节点上限诊断。
5. **`dlqr(...,R=0)`**：未校验奇异 R，当前返回有限值；测试固定现状并标注待补校验。

## 四、实现笔记（clean-room 出处）

| 算法 | 参考出处 |
|------|----------|
| Nelder–Mead | Nelder & Mead, *Computer Journal* 7(4), 1965 |
| 拟牛顿（BFGS） | Nocedal & Wright, *Numerical Optimization*, 2nd ed., ch.6 |
| 信赖域 | Nocedal & Wright ch.4 |
| 单纯形法 | Dantzig, 1947 |
| 内点法 | Karmarkar, *Combinatorica* 4, 1984 |
| 有效集 QP | Goldfarb & Idnani, *Math. Prog.* 27, 1983 |
| Brent 求根 | Brent, *Algorithms for Minimization without Derivatives*, 1973 |
| Levenberg–Marquardt | Moré, *Lecture Notes in Math* 630, 1978 |
| 遗传算法 | Holland, *Adaptation in Natural and Artificial Systems*, 1975 |
| 模拟退火 | Kirkpatrick et al., *Science* 220, 1983 |
| 粒子群 | Kennedy & Eberhart, 1995 |

**数值陷阱**：`fminunc` 的有限差分梯度步长；`fmincon` 的约束容差（`TolCon`）默认值；`quadprog` 在非严格凸 H 下的行为；启发式算法的 RNG 复现性（需支持 `rng` 种子）。

## 五、验收

- `demos/optim_demo.m`（v0.2 新增）：Rosenbrock（`fminunc`/`fminsearch`）、LP、QP、曲线拟合各 1 例，断言解接近 MATLAB。
- 测试数据用已知解析解的标准问题（如 Rosenbrock 最优 `[1,1]`）。
