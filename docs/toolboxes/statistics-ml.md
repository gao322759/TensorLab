# 统计与机器学习工具箱（Statistics and Machine Learning）

> 对标 **MATLAB R2026a — Statistics and Machine Learning Toolbox**。v0.2 目标：核心 20–30 函数可用。
> `category` = `"stats"`。

## 一、范围与目标

涵盖：描述统计、概率分布、假设检验、回归、降维、聚类、分类。

**不做**（➖）：深度学习（Deep Learning Toolbox）、`fitcsvm` 的核方法完整实现可后置、贝叶斯网络、时间序列模型（`arima` 等）、大数据 `tall` 数组。

## 二、函数清单

### 描述统计（内核大多已提供）

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `mean` / `median` / `mode` | `m = mean(A[,dim])` | 中心趋势 | ✅ | ✅ 内核 |
| `std` / `var` | `s = std(A[,w,dim])` | 离散度（默认 N-1 归一化） | ✅ | ✅ 内核 |
| `skewness` / `kurtosis` | `s = skewness(A[,flag])` | 形状 | ✅ | ✅ 内核 |
| `moment` | `m = moment(A,order)` | 中心矩 | ✅ | ✅ `toolbox/stats/moment.m` |
| `cov` / `corr` / `corrcoef` | `C = cov(A)` | 协方差/相关 | ✅ | ✅ 内核 |
| `prctile` / `quantile` | `q = prctile(A,p)` | 分位数 | ✅ | ✅ 内核 |
| `iqr` | `r = iqr(A)` | 四分位距 | ✅ | ✅ `toolbox/stats/iqr.m`（基于内核 `prctile`） |
| `mad` | `y = mad(A)` | 中位绝对偏差 | ✅ | ✅ 内核 |
| `range` / `zscore` | `r = range(A)` | 极差/标准化 | ✅ | ✅ `toolbox/stats/{range,zscore}.m` |
| `histcounts` / `hist` / `histogram` | `[n,edges] = histcounts(X[,nbins])` | 直方图 | ✅ | ✅ 内核 |

### 概率分布

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `normpdf`/`normcdf`/`norminv` | `y = normpdf(x,mu,sigma)` | 正态 | ✅ | ✅ 内核 |
| `normrnd` | `r = normrnd(mu,sigma[,sz])` | 正态随机数 | ✅ | ✅ 内核 |
| `normfit` | `[muhat,sigmahat] = normfit(x)` | 正态参数估计 | ✅ | ✅ `toolbox/stats/normfit.m`（+`chi2inv` 辅助） |
| `tpdf`/`tcdf` | `y = tpdf(x,nu)` | t 分布 | ✅ | ✅ 内核 |
| `chi2pdf` | `y = chi2pdf(x,nu)` | 卡方 | ✅ | ✅ `toolbox/stats_pdf/chi2pdf.m` |
| `exprnd`/`binornd`/`unifrnd` | 随机数 | ✅ | ✅ 内核 |
| `mvnrnd` | `R = mvnrnd(mu,Sigma,n)` | 多元正态随机数 | ✅ | ✅ 内核 |
| `fitdist` | `pd = fitdist(x,distname)` | 分布拟合 | ✅ | ✅ 内核 |
| `mle` | `phat = mle(x,...)` | 最大似然 | ✅ | ✅ `toolbox/stats_extra/mle/mle.m`（normal/exp/poisson） |

### 假设检验

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `ttest` | `h = ttest(x[,m,alpha])` | 单样本/配对 t 检验 | ✅ | ✅ `toolbox/stats/ttest.m`（无 stats 结构体） |
| `ttest2` | `h = ttest2(x,y)` | 双样本 t 检验 | ✅ | ✅ 内核 |
| `vartest` | `h = vartest(x,v)` | 方差检验 | ✅ | ✅ `toolbox/stats/vartest.m` |
| `anova1` / `anova2` | `p = anova1(y,group)` | 单/双因素方差分析 | ✅ | ✅ `toolbox/stats_extra/{anova1,anova2}`（返回 p/F，无 struct 表） |
| `kruskalwallis` | `p = kruskalwallis(x,group)` | 非参数检验 | ✅ | ✅ `toolbox/stats_extra/kruskalwallis`（含结秩校正） |

### 回归

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `polyfit` / `polyval` | `p = polyfit(x,y,n)` | 多项式拟合/求值 | ✅ | ✅ 内核 |
| `regress` | `[b,bint] = regress(y,X)` | 多元线性回归 | ✅ | ✅ 内核 |
| `fitlm` | `mdl = fitlm(X,y)` | 线性模型对象 | ✅ | ✅ 内核 |
| `glmfit` | `[b,dev] = glmfit(X,y,distr)` | 广义线性模型 | ✅ | ✅ `toolbox/stats_extra/glmfit`（normal/binomial/poisson，IRLS） |
| `nlinfit` | `beta = nlinfit(X,y,modelfun,beta0)` | 非线性回归 | ✅ | ✅ `toolbox/stats_extra/nlinfit`（数值 Jacobian Gauss–Newton） |

### 降维与聚类

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `pca` | `[coeff,score,latent] = pca(X)` | 主成分分析 | ✅ | ✅ `toolbox/stats/pca.m`（cov+eig，含符号约定/T²） |
| `kmeans` | `[idx,C] = kmeans(X,k)` | k-means 聚类 | ✅ | ✅ 内核 |
| `knnsearch` | `[idx,d] = knnsearch(X,Y)` | k 近邻搜索 | ✅ | ✅ 内核 |

### 分类与交叉验证

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `fitcsvm` | `mdl = fitcsvm(X,Y)` | 支持向量机 | ✅ | ⬜ v0.2 |
| `fitcknn` | `mdl = fitcknn(X,Y)` | k 近邻分类 | ✅ | ⬜ v0.2 |
| `fitctree` | `mdl = fitctree(X,Y)` | 决策树 | ✅ | ✅ 内核 |
| `fitcensemble` / `fitrensemble` | 集成 | ✅ | ➖ |
| `predict` | `y = predict(mdl,X)` | 预测 | ✅ | ✅ 内核 |
| `crossval` | `cvmdl = crossval(mdl)` | 交叉验证 | ✅ | ⬜ v0.2 |

### 随机数与随机流

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `rand`/`randn`/`rng` | `rng(seed)` | 随机流控制 | ✅ | ✅ 内核 |
| `randsample` / `datasample` | `y = randsample(n,k)` | 抽样 | ✅ | ✅ 内核 |

## 三、v0.2 实现清单（缺口）

v0.2 已交付：`pca`、`zscore`、`range`、`ttest`、`moment`、`iqr`、`normfit`、`vartest`、
`chi2pdf`、`mle`、`anova1`、`anova2`、`kruskalwallis`、`glmfit`、`nlinfit`（辅助：`fcdf`、`tl_betacf`）。

仍缺（诚实清单）：

1. **`fitcsvm` / `fitcknn` / `crossval`**：MATLAB 返回模型对象并依赖 `predict`；内核的 `predict`
   是 builtin，无法挂接自定义模型，v0.2 不交付。
2. `fitcensemble` / `fitrensemble`：明确不做。
3. **`nlinfit` 的发散模型**：P1 对抗性测试发现，指数模型（如 `b1*exp(b2*x)`）在初值不佳时会
   让数值 Jacobian 变成 NaN/Inf，触发内核 SVD 的 `Singular value was NaN` panic。当前 CI
   只测收敛模型与"模型返回长度错误"的干净反例；后续应在 Gauss–Newton 迭代内检查
   `isfinite(J)/isfinite(r)` 并提前返回诊断。

> `fitcsvm` / `fitcknn` 不在 v0.2 交付面（见第 1 条），因此 P1 的对抗性测试未包含它们。

> 辅助函数：`chi2inv`（`toolbox/stats/chi2inv.m`，基于内核 `chi2cdf` + 二分）也被加入内核缺口补齐。

## 四、实现笔记（clean-room 出处）

| 算法 | 参考出处 |
|------|----------|
| PCA | Jolliffe, *Principal Component Analysis*, 2nd ed., 2002（经 SVD） |
| k-means | Lloyd, 1957（1982 重印）；MacQueen, 1967 |
| SVM | Cortes & Vapnik, *Machine Learning* 20, 1995 |
| 决策树 | Breiman et al., *CART*, 1984 |
| 线性回归 | 最小二乘正规方程 / QR |
| t 检验 | 标准统计教材（如 Casella & Berger） |
| 卡方/正态 CDF | Abramowitz & Stegun, *Handbook of Mathematical Functions*, 1964 |

**数值陷阱**：`std`/`var` 默认 N-1（样本）归一化；`pca` 的中心化与符号约定（与 MATLAB 的 `coeff` 符号可能不同，需对齐 `Sign` 约定）；`kmeans` 的初始化（`kmeans++`）与 RNG 复现。

## 五、验收

- `demos/stats_demo.m`（v0.2 新增）：描述统计、回归、PCA、聚类各 1 例。
- 用已知答案的合成数据（如 `rng(0)` 后固定样本）断言。
