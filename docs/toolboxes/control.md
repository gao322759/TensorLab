# 控制系统工具箱（Control System）

> 对标 **MATLAB R2026a — Control System Toolbox**。v0.2 目标：核心 20–30 函数可用。
> `category` = `"control"`。

## 一、范围与目标

涵盖：LTI 模型（传递函数/状态空间/零极点）、模型互联、时域响应、频域响应、稳定性与裕度、极点配置与 LQR。

**不做**（➖）：Simulink、Robust Control / MPC 工具箱、`frd` 的完整频响数据对象、`tunablePID` 自动整定（`pidtune` 可选后置）。

## 二、函数清单

### 模型构造

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `tf` | `sys = tf(num,den)` | 传递函数模型 | ✅ | ✅ 内核 |
| `ss` | `sys = ss(A,B,C,D)` | 状态空间模型 | ✅ | ✅ 内核 |
| `zpk` | `sys = zpk(z,p,k)` | 零极点增益模型 | ✅ | ⬜ v0.2（高优先） |
| `frd` | `sys = frd(resp,freq)` | 频响数据模型 | ✅ | ➖ |
| `pid` | `C = pid(Kp,Ki,Kd)` | PID 控制器 | ✅ | ⬜ v0.2（高优先） |
| `pidstd` | `C = pidstd(Kp,Ti,Td)` | 标准型 PID | ✅ | ⬜ v0.2 |

### 模型互联

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `feedback` | `sys = feedback(sys1,sys2[,sign])` | 负/正反馈互联 | ✅ | ✅ 内核 |
| `series` | `sys = series(sys1,sys2)` | 串联 | ✅ | ✅ `toolbox/control/series.m`（conv 多项式乘） |
| `parallel` | `sys = parallel(sys1,sys2)` | 并联 | ✅ | ✅ `toolbox/control/parallel.m` |
| `append` | `sys = append(sys1,sys2)` | 块对角组合 | ✅ | ✅ 内核 |
| `connect` | `sys = connect(sys,...)` | 任意互联 | ✅ | ➖ |
| `sumblk` | `sum = sumblk(...)` | 求和块 | ✅ | ➖ |

### 时域响应

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `step` | `[y,t] = step(sys[,t])` | 阶跃响应 | ✅ | ✅ 内核 |
| `impulse` | `[y,t] = impulse(sys)` | 冲激响应 | ✅ | ✅ 内核 |
| `initial` | `[y,t] = initial(sys,x0)` | 零输入响应 | ✅ | ✅ `toolbox/control/initial.m`（RK4） |
| `lsim` | `[y,t] = lsim(sys,u,t[,x0])` | 任意输入响应 | ✅ | ✅ `toolbox/control/lsim.m`（RK4，分段常值输入） |
| `stepinfo` | `S = stepinfo(sys)` | 阶跃指标（上升/超调/稳定时间） | ✅ | ✅ 内核 |

### 频域响应

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `bode` | `[mag,phase,w] = bode(sys)` | 伯德图 | ✅ | ✅ `toolbox/control/bode.m`（数值，默认对数网格） |
| `bodemag` | `[mag,w] = bodemag(sys)` | 幅频 | ✅ | ✅ `toolbox/control/bodemag.m` |
| `nyquist` | `[re,im,w] = nyquist(sys)` | 奈奎斯特图 | ✅ | ✅ 内核（数值） |
| `nichols` | `nichols(sys)` | 尼科尔斯图 | ✅ | ✅ `toolbox/control/nichols.m`（数值 dB/deg，不绘图） |
| `margin` / `allmargin` | `[Gm,Pm,Wcg,Wcp] = margin(sys)` | 幅值/相位裕度 | ✅ | ✅ `toolbox/control/{margin,allmargin}.m`（`allmargin` 返回数值向量，非 struct） |
| `freqresp` | `H = freqresp(sys,w)` | 频率响应求值 | ✅ | ✅ `toolbox/control/freqresp.m` |
| `evalfr` | `H = evalfr(sys,s)` | 单点求值 | ✅ | ✅ `toolbox/control/evalfr.m` |
| `bandwidth` | `fb = bandwidth(sys)` | 带宽 | ✅ | ✅ 内核 |

### 极点/零点与稳定性

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `pole` / `zero` | `p = pole(sys)` | 极点/零点 | ✅ | ✅ 内核 |
| `pzmap` / `pzplot` | `[p,z] = pzmap(sys)` | 零极点图 | ✅ | ✅ 内核（数值） |
| `dcgain` | `k = dcgain(sys)` | 直流增益 | ✅ | ✅ 内核 |
| `damp` | `[wn,zeta,p] = damp(sys)` | 自然频率/阻尼 | ✅ | ✅ 内核 |
| `isstable` | `tf = isstable(sys)` | 稳定性 | ✅ | ✅ 内核 |
| `isct` / `isdt` | `tf = isct(sys)` | 连续/离散判别 | ✅ | ✅ `toolbox/control/{isct,isdt}.m` |
| `minreal` | `sys = minreal(sys)` | 最小实现（消零极点对消） | ✅ | ✅ `toolbox/control/minreal.m` |
| `balreal` | `[sys,g] = balreal(sys)` | 平衡实现 | ✅ | ➖ |

### 设计（极点配置 / 最优）

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `lqr` / `dlqr` | `[K,S,e] = lqr(A,B,Q,R)` | LQR 最优增益（Riccati） | ✅ | ✅ 内核（`lqr`）；✅ `toolbox/control/dlqr.m`（值迭代 Riccati） |
| `place` | `K = place(A,B,p)` | 极点配置 | ✅ | ✅ `toolbox/control/place.m`（单输入；多输入 ⬜） |
| `acker` | `K = acker(A,B,p)` | Ackermann 公式 | ✅ | ✅ `toolbox/control/acker.m` |
| `rlocus` | `rlocus(sys)` | 根轨迹 | ✅ | ✅ 内核（数值） |
| `rlocfind` | `[k,poles] = rlocfind(sys)` | 交互选点 | ✅ | ➖（无交互） |
| `kalman` | `[kest,L,P] = kalman(sys,Q,R)` | 卡尔曼滤波增益 | ✅ | ✅ `toolbox/control/kalman.m`（Hamilton 子空间；返回 L/P，不返回 estimator 对象） |
| `pidtune` | `C = pidtune(sys,type)` | PID 自动整定 | ✅ | ➖ |

> `kalman` 的解析样例：1×1 系统 `A=-1, C=1, Q=1, R=1` → `L=0.2929, P=0.4142`
> （`adversarial.rs` 断言）；`control_demo.m` 用的是 2×2 系统，
> `L(1,1)=0.4019, P(1,1)=0.6720`（golden 快照断言）。

### 离散化与模型转换

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `c2d` | `sysd = c2d(sys,Ts[,method])` | 连续→离散（ZOH/Tustin） | ✅ | ✅ `toolbox/control/c2d.m`（zoh + tustin） |
| `d2c` / `d2d` | `sys = d2c(sysd)` | 离散→连续/重采样 | ✅ | ✅ `toolbox/control/{d2c,d2d}.m`（Tustin；`d2c`/`d2d` 需显式给旧采样周期） |
| `ss2tf` / `tf2ss` | `[num,den] = ss2tf(A,B,C,D)` | 状态空间↔传递函数 | ✅ | ✅ `toolbox/control/{ss2tf,tf2ss}.m`（SISO，Faddeev–LeVerrier） |
| `tf2zp` / `zp2tf` | `[z,p,k] = tf2zp(num,den)` | 传递函数↔零极点 | ✅ | ✅ `toolbox/control/{tf2zp,zp2tf}.m` |
| `ss2zp` / `zp2ss` | 状态空间↔零极点 | ✅ | ✅ `toolbox/control/{ss2zp,zp2ss}.m` |
| `tfdata` / `ssdata` / `zpkdata` | `[num,den] = tfdata(sys)` | 提取模型数据 | ✅ | ✅ `toolbox/control/{tfdata,ssdata}.m`（`zpkdata` 需 `zpk` class，⬜） |

## 三、v0.2 实现清单（缺口）

v0.2 已交付：`bode`、`bodemag`、`margin`、`allmargin`、`nichols`、`series`、`parallel`、`evalfr`、
`freqresp`、`place`、`acker`、`isct`/`isdt`、`tfdata`、`ssdata`、`minreal`、`dlqr`、`kalman`、
`c2d`、`d2c`、`d2d`、`ss2zp`、`zp2ss`（共 20+ 个可用函数）。

仍缺（诚实清单）：

1. **`zpk` / `pid` / `pidstd`**：需要内核提供对应 class（v0.2 无），无法返回模型对象。
2. **`zpkdata`**：依赖 `zpk` class。
3. **`c2d` 的 `zoh`**：要求极点互异且非零；否则自动回退 Tustin（文档化）。
4. **不做**：`frd`、`connect`、`sumblk`、`rlocfind`、`pidtune`、`balreal`。

## 四、实现笔记（clean-room 出处）

| 算法 | 参考出处 |
|------|----------|
| 状态空间/传递函数变换 | Kailath, *Linear Systems*, 1980 |
| LQR / Riccati | Anderson & Moore, *Optimal Control: Linear Quadratic Methods*, 1990 |
| 极点配置（Ackermann） | Ackermann, 1972；Kailath ch.6 |
| 离散化（ZOH/Tustin/匹配） | Franklin, Powell & Workman, *Digital Control of Dynamic Systems*, 3rd ed. |
| 伯德/奈奎斯特/裕度 | Ogata, *Modern Control Engineering*, 5th ed. |
| 卡尔曼滤波器 | Kalman, 1960；Anderson & Moore, *Optimal Filtering*, 1979 |

**数值陷阱**：`tf` 的有理多项式需做约分（`minreal` 语义）；`margin` 的相位穿越判定（360° 回绕）；`bode` 的频率向量自动选择需与 MATLAB 对齐；状态空间的可控/可观性判定容差。

## 五、验收

- `demos/control_demo.m`（v0.2 新增）：`tf`→`step`/`bode`/`margin`→`lqr`/`place`→`c2d`。
- 用标准二阶系统（ωn=1, ζ=0.5）断言阶跃指标 `stepinfo` 与理论值一致。
