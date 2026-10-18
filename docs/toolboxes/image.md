# 图像处理工具箱（Image Processing）

> 对标 **MATLAB R2026a — Image Processing Toolbox**。v0.2 目标：核心 20–30 函数可用。
> `category` = `"image"`。

## 一、范围与目标

涵盖：图像 I/O、类型转换与显示、几何变换、空间域滤波、形态学、颜色空间、直方图增强、区域分析。

**不做**（➖）：深度学习图像模型（Deep Learning Toolbox）、医学影像（Medical Imaging Toolbox）、点云、GPU 加速、`imshow` 的交互式工具栏（桌面端用自研查看器）。

## 二、函数清单

### I/O 与显示

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `imread` | `A = imread(fname[,fmt])` | 读 PNG/JPEG/BMP/TIFF | ✅ | ✅ 内核 |
| `imwrite` | `imwrite(A,fname[,fmt])` | 写图像 | ✅ | ✅ 内核 |
| `imfinfo` | `info = imfinfo(fname)` | 元信息 | ✅ | ✅ 内核 |
| `imshow` | `imshow(I[,map])` | 显示图像（图窗） | ✅ | ✅ 内核 |
| `imagesc` | `imagesc(C[,clims])` | 位置校正显示（图窗） | ✅ | ✅ 内核 |
| `montage` | `montage(files)` | 图像拼贴 | ✅ | ➖（可选） |

### 类型转换

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `im2double` | `I = im2double(X)` | 归一化到 [0,1] double | ✅ | ✅ 内核 |
| `im2uint8` | `I = im2uint8(X)` | 转 uint8 | ✅ | ✅ 内核 |
| `im2uint16` | `I = im2uint16(X)` | 转 uint16 | ✅ | ✅ 内核 |
| `mat2gray` | `I = mat2gray(A[,limits])` | 线性映射到 [0,1] | ✅ | ✅ `toolbox/image/mat2gray.m` |

### 几何变换

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `imresize` | `B = imresize(A,scale[,method])` | 缩放（nearest/bilinear；bicubic→bilinear） | ✅ | ✅ `toolbox/image/imresize.m` |
| `imrotate` | `B = imrotate(A,angle[,method])` | 旋转 | ✅ | ✅ `toolbox/image/imrotate.m` |
| `imcrop` | `B = imcrop(A[,rect])` | 裁剪 | ✅ | ✅ `toolbox/image/imcrop.m` |
| `imtranslate` | `B = imtranslate(A,translation)` | 平移 | ✅ | ✅ `toolbox/image/imtranslate.m` |
| `imwarp` | `B = imwarp(A,tform)` | 通用几何变换 | ✅ | ➖（依赖 affinetform2d 类） |

### 空间域滤波与去噪

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `imfilter` | `B = imfilter(A,h[,options])` | 相关/卷积滤波 | ✅ | ✅ 内核 |
| `fspecial` | `h = fspecial(type,params)` | 预定义模板（average/gaussian/sobel/laplacian） | ✅ | ✅ 内核 |
| `medfilt2` | `B = medfilt2(A[,m n])` | 中值滤波 | ✅ | ✅ `toolbox/image/medfilt2.m`（零填充边界） |
| `ordfilt2` | `B = ordfilt2(A,order,domain)` | 顺序统计滤波 | ✅ | ✅ `toolbox/image/ordfilt2.m` |
| `wiener2` | `B = wiener2(A[,m n])` | 自适应维纳去噪 | ✅ | ✅ `toolbox/image/wiener2.m` |
| `imgaussfilt` | `B = imgaussfilt(A,sigma)` | 高斯平滑 | ✅ | ✅ `toolbox/image/imgaussfilt.m` |
| `imnoise` | `J = imnoise(I,type,...)` | 加噪（gaussian/salt&pepper/speckle） | ✅ | ✅ `toolbox/image/imnoise.m` |
| `edge` | `BW = edge(I[,method])` | 边缘检测（Sobel/Prewitt/Roberts/Laplacian/LoG） | ✅ | ✅ `toolbox/image/edge.m`（无 Canny） |

### 形态学

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `strel` | `SE = strel(shape,params)` | 结构元素 | ✅ | ✅ `toolbox/image/strel.m`（square/rectangle/disk/diamond/line/octagon，返回 struct） |
| `imdilate` / `imerode` | `J = imdilate(I,SE)` | 膨胀/腐蚀 | ✅ | ✅ `toolbox/image/{imdilate,imerode}.m`（接受数值 SE 或 struct SE） |
| `imopen` / `imclose` | `J = imopen(I,SE)` | 开/闭运算 | ✅ | ✅ `toolbox/image/{imopen,imclose}.m` |
| `imfill` | `BW2 = imfill(BW[,locations])` | 孔洞填充 | ✅ | ✅ `toolbox/image/imfill.m`（'holes'，4 连通） |
| `bwareaopen` | `BW2 = bwareaopen(BW,P)` | 去除小连通域 | ✅ | ✅ `toolbox/image/bwareaopen.m` |
| `bwlabel` / `bwconncomp` | `[L,n] = bwlabel(BW)` | 连通域标记 | ✅ | ✅ `toolbox/image/bwlabel.m`（4/8 连通）、`toolbox/image/bwconncomp.m`（返回 struct：Connectivity/ImageSize/NumObjects/PixelIdxList） |
| `regionprops` | `stats = regionprops(BW,props)` | 区域属性 | ✅ | ✅ `toolbox/image/regionprops.m`（Area/Centroid/BoundingBox/PixelIdxList/MajorAxisLength/MinorAxisLength/Orientation/Perimeter；非 'basic' 返回固定超集，文档化偏差） |
| `bwdist` | `D = bwdist(BW)` | 距离变换 | ✅ | ✅ `toolbox/image/bwdist.m`（精确欧氏，暴力搜索） |

### 颜色与增强

| 函数 | 签名 | 语义要点 | R2026a | v0.1 状态 |
|------|------|----------|--------|-----------|
| `rgb2gray` | `I = rgb2gray(RGB)` | 灰度（Rec.601 luma） | ✅ | ✅ 内核 |
| `rgb2hsv` / `hsv2rgb` | `HSV = rgb2hsv(RGB)` | RGB↔HSV | ✅ | ✅ 内核 |
| `rgb2ycbcr` / `ycbcr2rgb` | `YCBCR = rgb2ycbcr(RGB)` | RGB↔YCbCr | ✅ | ✅ `toolbox/image/{rgb2ycbcr,ycbcr2rgb}.m` |
| `ind2rgb` | `RGB = ind2rgb(X,map)` | 索引→真彩 | ✅ | ✅ 内核 |
| `label2rgb` | `RGB = label2rgb(L)` | 标签→伪彩 | ✅ | ✅ `toolbox/image/label2rgb.m`（HSV 轮盘调色） |
| `imadjust` | `J = imadjust(I,[low_in;high_in],[low_out;high_out])` | 对比度拉伸 | ✅ | ✅ `toolbox/image/imadjust.m` |
| `imhist` | `[counts,x] = imhist(I)` | 直方图 | ✅ | ✅ 内核 |
| `histeq` | `J = histeq(I[,n])` | 直方图均衡 | ✅ | ✅ `toolbox/image/histeq.m` |
| `adapthisteq` | `J = adapthisteq(I)` | CLAHE | ✅ | ⬜ v0.2 |
| `imquantize` | `J = imquantize(I,levels)` | 多级量化 | ✅ | ✅ `toolbox/image/imquantize.m` |

## 三、v0.2 实现清单（缺口）

v0.2 已交付：`imresize`、`edge`、`medfilt2`、`imrotate`、`imcrop`、`imtranslate`、`ordfilt2`、
`histeq`、`imadjust`、`imgaussfilt`、`imnoise`、`wiener2`、`bwdist`、`bwlabel`、`bwareaopen`、
`imfill`、`imdilate`、`imerode`、`imopen`、`imclose`、`mat2gray`、`imquantize`、`label2rgb`、
`rgb2ycbcr`、`ycbcr2rgb`（25 个脚本函数 + 内核函数，满足 20–30 的目标）。

仍缺（诚实清单，均受内核能力阻塞）：

1. **`adapthisteq`（CLAHE）**：v0.2 未实现。
2. **`imwarp`**：依赖 `affinetform2d` class，标记 ➖（`classdef` 仅 Path 请求可用，见 [gaps.md](../gaps.md) K8）。
3. **`montage`**：标记 ➖。

> v0.3-dev 更正：内核 `struct` 可用（[gaps.md](../gaps.md) K5 已关闭），`strel`/`regionprops`/`bwconncomp` 已按 MATLAB 返回 struct 交付，并由 `tl-commands` 的 `toolbox_struct_based_image_functions_are_usable` 回归测试覆盖。

## 四、实现笔记（clean-room 出处）

| 算法 | 参考出处 |
|------|----------|
| 双三次插值 | Keys, *IEEE Trans. ASSP* 29(6), 1981 |
| Canny 边缘 | Canny, *IEEE TPAMI* 8(6), 1986 |
| Sobel/Prewitt | Gonzalez & Woods, *Digital Image Processing*, 4th ed. |
| 中值滤波 | Huang et al., 1979 |
| 数学形态学 | Serra, *Image Analysis and Mathematical Morphology*, 1982 |
| 直方图均衡 | Gonzalez & Woods ch.3 |
| CLAHE | Zuiderveld, 1994 |
| 灰度权重 | ITU-R BT.601 luma（与 MATLAB `rgb2gray` 一致） |

**数值陷阱**：`imresize` 的 antialiasing 与 `Method` 默认值（C 实现细节）需对齐；`edge` 的阈值自动算法（`graythresh`/Otsu）需单独实现；形态学的边界处理（pad 方式）需与 MATLAB 一致。

## 五、验收

- `demos/image_toolbox.m`（v0.2 新增）：读图→灰度→缩放→边缘→形态学→保存，逐值断言。
- 需要 `tests/data/` 放置小尺寸测试图（自研生成，不用 MathWorks 测试数据）。
