# 信号处理工具箱（Signal Processing）

> 对标 **MATLAB R2026a — Signal Processing Toolbox**。v0.2 目标：核心 20–30 函数可用。
> `category` = `"signal"`（`help` 分组用）。

## 一、范围与目标

涵盖：变换（DFT/DCT/STFT）、数字滤波（FIR/IIR 设计与应用）、相关与卷积、统计信号分析（谱估计）、重采样、窗函数、信号生成。

**不做**（➖）：小波工具箱（Wavelet Toolbox 独立产品）、DSP System Toolbox 的流式对象、定点（fixed-point）算术、GPU 加速路径。

## 二、函数清单

### 变换与频谱

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `fft` | `X = fft(x[,n[,dim]])` | 一维 DFT | ✅ | ✅ 内核（fft_demo 验证） |
| `ifft` | `x = ifft(X[,n[,dim]])` | 逆 DFT，含 1/N | ✅ | ✅ 内核 |
| `fft2` | `Y = fft2(X[,m,n])` | 二维 DFT | ✅ | ✅ 内核 |
| `ifft2` | `y = ifft2(Y[,m,n])` | 二维逆 DFT | ✅ | ✅ 内核 |
| `fftn` | `Y = fftn(X[,sz])` | N 维 DFT | ✅ | ✅ 内核 |
| `ifftn` | `y = ifftn(Y[,sz])` | N 维逆 DFT | ✅ | ✅ 内核 |
| `fftshift` | `Y = fftshift(X[,dim])` | 零频移到中心 | ✅ | ✅ 内核 |
| `ifftshift` | `Y = ifftshift(X[,dim])` | `fftshift` 的逆 | ✅ | ✅ 内核 |
| `dct` | `y = dct(x[,n])` | DCT-II（正交归一） | ✅ | ✅ `toolbox/signal/dct.m` |
| `idct` | `x = idct(y[,n])` | 逆 DCT | ✅ | ✅ `toolbox/signal/idct.m` |
| `czt` | `y = czt(x[,m,w,a])` | 线性调频 Z 变换 | ✅ | ✅ `toolbox/signal/czt.m`（直接求和） |
| `goertzel` | `y = goertzel(x,ind)` | 单频点 DFT | ✅ | ✅ `toolbox/signal/goertzel.m` |

### 滤波

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `filter` | `y = filter(b,a,x[,zi])` | IIR/FIR 直接 II 转置型 | ✅ | ✅ 内核 |
| `conv` / `conv2` | `c = conv(u,v[,shape])` | 一维/二维卷积 | ✅ | ✅ 内核（矩阵 demo 验证） |
| `convn` | `Y = convn(A,B[,shape])` | N 维卷积 | ✅ | ✅ `toolbox/signal/convn.m`（1-D/2-D；full/same/valid） |
| `butter` | `[b,a] = butter(n,Wn[,ftype])` | Butterworth 设计 | ✅ | ✅ 内核（设计部分） |
| `cheby1` / `cheby2` | `[b,a] = cheby1(n,Rp,Wn)` | Chebyshev I/II 设计 | ✅ | ⬜ v0.2 |
| `ellip` | `[b,a] = ellip(n,Rp,Rs,Wn)` | 椭圆滤波器 | ✅ | ⬜ v0.2 |
| `fir1` | `b = fir1(n,Wn[,ftype[,win]])` | 窗函数法 FIR 设计 | ✅ | ✅ 内核 |
| `fir2` | `b = fir2(n,f,m)` | 频率采样法 FIR | ✅ | ✅ `toolbox/signal/fir2.m`（网格插值 + Hamming + 首带归一） |
| `freqz` | `[h,w] = freqz(b,a[,n,fs])` | 数字滤波器频率响应 | ✅ | ✅ 内核 |
| `freqs` | `h = freqs(b,a,w)` | 模拟滤波器频率响应 | ✅ | ✅ `toolbox/signal/freqs.m` |
| `upfirdn` | `y = upfirdn(x,h,p,q)` | 上采样→滤波→下采样 | ✅ | ✅ `toolbox/signal/upfirdn.m` |

### 相关与统计信号

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `xcorr` | `r = xcorr(x,y[,maxlag[,scaleopt]])` | 互/自相关 | ✅ | ✅ `toolbox/signal/xcorr.m`（自/互相关，无 maxlag/scaleopt） |
| `xcov` | `c = xcov(x,y[,maxlag])` | 协方差序列 | ✅ | ✅ `toolbox/signal/xcov.m`（去均值后调 `xcorr`） |
| `findpeaks` | `[pks,locs] = findpeaks(x)` | 局部极大值 | ✅ | ✅ `toolbox/signal/findpeaks.m`（内部局部极大，无 MinPeakHeight 等） |
| `envelope` | `[up,lo] = envelope(x[,fl[,method]])` | 包络提取 | ✅ | ✅ 内核 |
| `hilbert` | `x = hilbert(xr)` | 解析信号 | ✅ | ✅ 内核 |
| `spectrogram` | `[s,f,t] = spectrogram(x,...)` | 短时傅里叶谱 | ✅ | ✅ 内核 |
| `pwelch` | `[pxx,f] = pwelch(x,...)` | Welch 功率谱估计 | ✅ | ✅ 内核 |
| `periodogram` | `[pxx,f] = periodogram(x,...)` | 周期图 | ✅ | ✅ 内核 |
| `pspectrum` | `pspectrum(x,fs)` | 现代谱/谱图 | ✅ | ✅ `toolbox/signal/pspectrum.m`（单边功率谱，矩形窗） |
| `stft` / `istft` | `[s,f,t] = stft(x,fs[,win,noverlap,nfft])` | STFT / 逆 STFT | ✅ | ✅ `toolbox/signal/{stft,istft}.m`（Hann、单边谱、重叠相加重建） |

### 重采样与窗

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `interp1` | `yi = interp1(x,y,xi[,method])` | 一维插值（linear/nearest/spline/pchip） | ✅ | ✅ 内核 |
| `resample` | `y = resample(x,p,q)` | 有理数重采样 | ✅ | ✅ 内核 |
| `decimate` | `y = decimate(x,r)` | 抗混叠抽取 | ✅ | ✅ `toolbox/signal/decimate.m`（fir1+`filtfilt`，短信号回退 `filter`） |
| `downsample` / `upsample` | `y = downsample(x,n)` | 无滤波抽取/插零 | ✅ | ✅ 内核 |
| `hamming` / `hann` / `blackman` | `w = hamming(n)` | 窗函数 | ✅ | ✅ 内核 |
| `rectwin` / `tukeywin` | `w = rectwin(n)` | 矩形/Tukey 窗 | ✅ | ✅ `toolbox/signal/{rectwin,tukeywin}.m` |
| `kaiser` | `w = kaiser(n,beta)` | Kaiser 窗 | ✅ | ✅ `toolbox/signal/kaiser.m`（I0 幂级数） |
| `kaiserord` | `[n,Wn,beta,ftype] = kaiserord(f,a,dev,fs)` | Kaiser 阶数估算 | ✅ | ✅ `toolbox/signal/kaiserord.m` |

### 信号生成（内核已提供）

`square` ✅ · `sawtooth` ✅ · `sinc` ✅ · `pulstran` ✅ · `gauspuls` ✅ · `rectpuls` ✅ · `tripuls` ✅ · `chirp` ⬜

## 三、v0.2 实现清单（缺口）

v0.2 已交付：`xcorr`、`findpeaks`、`xcov`、`dct`/`idct`、`kaiser`、`decimate`、`stft`/`istft`、
`convn`、`fir2`、`freqs`、`upfirdn`、`pspectrum`、`kaiserord`、`czt`、`goertzel`。

仍缺（诚实清单）：

1. **IIR 设计族** `cheby1`/`cheby2`/`ellip`：需要双线性变换 + 切比雪夫/椭圆原型；`ellip` 还涉及
   Landen 变换与椭圆积分，v0.2 未交付。
2. **`chirp`**：内核未提供，v0.2 未补。
3. **`designfilt`**：返回 filter object，内核无对应 class，标记 ➖/待内核。

## 四、实现笔记（clean-room 出处）

| 算法 | 参考出处 |
|------|----------|
| FFT | Cooley & Tukey, *Math. Comp.* 19(90), 1965 |
| FIR 窗函数设计 | Oppenheim & Schafer, *Discrete-Time Signal Processing*, 3rd ed. |
| IIR 双线性变换设计 | Bilinear transform, 见 Oppenheim & Schafer ch.7 |
| 直接型 II 转置滤波 | Oppenheim & Schafer ch.6 |
| 相关（直接/FFT） | Oppenheim & Schafer ch.2, 10 |
| Welch 谱估计 | Welch, *IEEE Trans. AU-15*, 1967 |
| STFT / 谱图 | Allen & Rabiner, *Proc. IEEE* 65(11), 1977 |
| 多相重采样 | Crochiere & Rabiner, *Multirate Digital Signal Processing*, 1983 |
| Kaiser 窗 | Kaiser, 1974 |

**数值陷阱**：`fft` 长度非 2 的幂时性能下降（RunMat 使用混合基）；`conv` 与 `filter` 的边界（`shape`/`zi`）需与 MATLAB 逐项对齐。

## 五、验收

- `demos/fft_demo.m`（v0.1 已有，零错误）
- v0.2 新增 `demos/signal_toolbox.m`：对每类函数各取 1–2 例，断言数值与 MATLAB 参考一致。
- `tl-toolbox/tests/` 或 `tl-commands/tests/engine_contract.rs` 加端到端断言。
