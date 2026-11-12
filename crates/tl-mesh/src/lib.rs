//! TensorLab L1 — 结构化柱坐标 3D FVM 网格（plan.md 4.1，v0.5-A）。
//!
//! 纯数据结构与拓扑工具，零外部依赖（L1 规则：只依赖 std）。
//! 为 +sicpvt 阶段 16.1 的真 3D 工况提供网格底座；FVM 算子仍由包侧组装
//! （复用底层、上层叠加的既定哲学），本 crate 只回答"网格长什么样"：
//!
//! - [`CylGrid3D`]：柱坐标 (r, θ, z) 贴体网格的几何量——cell 中心、
//!   三向面面积（A_r / A_θ / A_z）、cell 体积、轴向角向邻接的线性索引。
//!   语义与 +sicpvt 的 2D `make_grid` 同构（Ar/Az/V 推广到 θ 向）。
//! - [`CylGrid3D::uniform_refine`]：整体加密（每轴 ×2，cell 数 ×8），
//!   加密后几何量重新解析计算（不是插值），保证体积/面积守恒。
//! - [`refine_fraction`]：按角向扇区局部加密的均匀子网格构造——非贴体
//!   的 2:1 非一致对接（hanging node）FVM 算子属后续批次；当前扇区加密
//!   以**独立子网格**形式交付（θ 向分区各解各的，边界通量交换留给上层）。
//!
//! 自适应 2:1 各向异性加密（plan 4.1 完整形态）需要非一致网格的通量
//! 插值与粗细面聚合，属后续批次；本 crate 的数据结构已为它预留
//! `level` 字段（0 = 基准层）。

/// 柱坐标结构化 3D 网格（cell 中心 FVM 语义，与 +sicpvt 2D make_grid 同构）。
#[derive(Debug, Clone, PartialEq)]
pub struct CylGrid3D {
    /// 径向 cell 数
    pub nr: usize,
    /// 角向 cell 数（0 < ntheta，全周 2π 均分）
    pub ntheta: usize,
    /// 轴向 cell 数
    pub nz: usize,
    /// 腔体外半径 [m]
    pub radius: f64,
    /// 腔体轴向长度 [m]
    pub length: f64,
    /// 加密层级（0 = 基准层；uniform_refine 每次 +1）
    pub level: u32,
    /// 角向覆盖范围 [rad]（new() = 全周 2π；refine_sector 子网格 = 扇区宽）
    pub theta_span: f64,
    /// 径向 cell 中心 [m]，nr 个
    pub r: Vec<f64>,
    /// 角向 cell 中心 [rad]，ntheta 个（θ_i = (i-1/2)·dθ）
    pub theta: Vec<f64>,
    /// 轴向 cell 中心 [m]，nz 个
    pub z: Vec<f64>,
    /// 径向面面积 A_r [m²]（(nr+1) 个面：θ-z 圆环面）
    pub ar: Vec<f64>,
    /// 角向面面积 A_θ [m²]（(ntheta+1) 个面：r-z 纵切面，全周相同）
    pub at: f64,
    /// 轴向面面积 A_z [m²]（(nz+1) 个面：r-θ 圆盘）
    pub az: Vec<f64>,
    /// cell 体积 [m³]，nr × ntheta × nz（r 最快的线性索引）
    pub vol: Vec<f64>,
}

impl CylGrid3D {
    /// 均匀柱坐标网格：r、θ、z 三向等分。
    pub fn new(nr: usize, ntheta: usize, nz: usize, radius: f64, length: f64) -> Option<Self> {
        if nr == 0 || ntheta == 0 || nz == 0 || radius <= 0.0 || length <= 0.0 {
            return None;
        }
        let mut g = CylGrid3D {
            nr,
            ntheta,
            nz,
            radius,
            length,
            level: 0,
            theta_span: std::f64::consts::TAU,
            r: Vec::new(),
            theta: Vec::new(),
            z: Vec::new(),
            ar: Vec::new(),
            at: 0.0,
            az: Vec::new(),
            vol: Vec::new(),
        };
        g.recompute();
        Some(g)
    }

    /// 由维度重新解析计算全部几何量（new 与 uniform_refine 共用）。
    /// θ 中心从 0 起算（子网格为扇区局部坐标，全局偏移由 refine_sector 返回）。
    fn recompute(&mut self) {
        let dr = self.radius / self.nr as f64;
        let dt = self.theta_span / self.ntheta as f64;
        let dz = self.length / self.nz as f64;

        self.r = (0..self.nr).map(|i| (i as f64 + 0.5) * dr).collect();
        self.theta = (0..self.ntheta).map(|i| (i as f64 + 0.5) * dt).collect();
        self.z = (0..self.nz).map(|k| (k as f64 + 0.5) * dz).collect();

        // A_r(k 面) = θ 张角 × r_face × dz（圆环侧面）；r_face = i·dr
        self.ar = (0..=self.nr).map(|i| dt * (i as f64 * dr) * dz).collect();
        // A_θ = dr × dz（纵切矩形；全周相同）
        self.at = dr * dz;
        // A_z(k 面) = π R²（轴向面是全半径圆盘，与 z 无关）
        self.az = vec![std::f64::consts::PI * self.radius * self.radius; self.nz + 1];

        // cell (i, j, k) 体积 = ∫ r dr dθ dz = 0.5 (r_out² - r_in²) dθ dz
        self.vol = Vec::with_capacity(self.nr * self.ntheta * self.nz);
        for i in 0..self.nr {
            let r_in = i as f64 * dr;
            let r_out = (i + 1) as f64 * dr;
            let ring = 0.5 * (r_out * r_out - r_in * r_in);
            for _j in 0..self.ntheta {
                for _k in 0..self.nz {
                    self.vol.push(ring * dt * dz);
                }
            }
        }
    }

    /// cell (i, j, k) 的线性索引（r 最快：i + nr·(j + ntheta·k)）。
    pub fn index(&self, i: usize, j: usize, k: usize) -> Option<usize> {
        if i >= self.nr || j >= self.ntheta || k >= self.nz {
            return None;
        }
        Some(i + self.nr * (j + self.ntheta * k))
    }

    /// cell 总数。
    pub fn cell_count(&self) -> usize {
        self.nr * self.ntheta * self.nz
    }

    /// 整体加密：每轴 ×2（cell 数 ×8），层级 +1，几何量解析重算。
    pub fn uniform_refine(&self) -> Self {
        let mut g = CylGrid3D {
            nr: self.nr * 2,
            ntheta: self.ntheta * 2,
            nz: self.nz * 2,
            radius: self.radius,
            length: self.length,
            level: self.level + 1,
            theta_span: self.theta_span,
            r: Vec::new(),
            theta: Vec::new(),
            z: Vec::new(),
            ar: Vec::new(),
            at: 0.0,
            az: Vec::new(),
            vol: Vec::new(),
        };
        g.recompute();
        g
    }

    /// 角向扇区局部加密（stage 16.1 遮挡/偏心工况）：把角向
    /// [θ_lo, θ_hi]（弧度）扇区独立加密 `factor` 倍，返回**独立子网格**
    /// （θ 覆盖范围 = 扇区宽，theta 为扇区局部坐标）与全局角向范围
    /// [lo, hi]。非一致对接（hanging node 通量交换）属后续批次——
    /// 上层以分区求解 + 边界通量交换使用。
    pub fn refine_sector(
        &self,
        theta_lo: f64,
        theta_hi: f64,
        factor: usize,
    ) -> Option<(CylGrid3D, f64, f64)> {
        if factor == 0 || factor > 8 || theta_hi <= theta_lo {
            return None;
        }
        let dt = self.theta_span / self.ntheta as f64;
        let j_lo = ((theta_lo / dt).floor() as usize).min(self.ntheta);
        let j_hi = ((theta_hi / dt).ceil() as usize).min(self.ntheta);
        if j_hi <= j_lo {
            return None;
        }
        let mut sub = CylGrid3D {
            nr: self.nr,
            ntheta: (j_hi - j_lo) * factor,
            nz: self.nz,
            radius: self.radius,
            length: self.length,
            level: self.level + 1,
            theta_span: (j_hi - j_lo) as f64 * dt,
            r: Vec::new(),
            theta: Vec::new(),
            z: Vec::new(),
            ar: Vec::new(),
            at: 0.0,
            az: Vec::new(),
            vol: Vec::new(),
        };
        sub.recompute();
        Some((sub, j_lo as f64 * dt, j_hi as f64 * dt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn grid_geometry_invariants() {
        let g = CylGrid3D::new(8, 12, 20, 0.0375, 0.032).expect("grid");
        assert_eq!(g.cell_count(), 8 * 12 * 20);

        // 体积守恒：Σ vol == π R² L
        let total: f64 = g.vol.iter().sum();
        let domain = std::f64::consts::PI * 0.0375f64.powi(2) * 0.032;
        assert!(close(total, domain, 1e-12), "vol sum {total} vs {domain}");

        // A_z 圆盘：每层都是全半径圆盘
        assert!(close(
            g.az[0],
            std::f64::consts::PI * 0.0375f64.powi(2),
            1e-12
        ));
        assert!(close(
            g.az[g.nz],
            std::f64::consts::PI * 0.0375f64.powi(2),
            1e-12
        ));

        // A_r 外缘面（每 cell 扇段）= θ 段 × R × dz
        let ar_outer = g.ar[g.nr];
        let dt = std::f64::consts::TAU / 12.0;
        assert!(close(ar_outer, dt * 0.0375 * (0.032 / 20.0), 1e-12));

        // cell 中心在第一象限正确
        assert!(close(g.r[0], 0.0375 / 16.0, 1e-15));
        assert!(close(g.z[g.nz - 1], 0.032 - 0.032 / 40.0, 1e-15));
    }

    #[test]
    fn indexing_and_bounds() {
        let g = CylGrid3D::new(4, 6, 3, 1.0, 1.0).expect("grid");
        assert_eq!(g.index(0, 0, 0), Some(0));
        assert_eq!(g.index(3, 0, 0), Some(3));
        assert_eq!(g.index(0, 1, 0), Some(4));
        assert_eq!(g.index(0, 0, 1), Some(4 * 6));
        assert_eq!(g.index(4, 0, 0), None);
        assert_eq!(g.index(0, 6, 0), None);
        assert_eq!(g.index(0, 0, 3), None);
    }

    #[test]
    fn uniform_refine_doubles_and_conserves() {
        let g = CylGrid3D::new(6, 8, 10, 0.03, 0.04).expect("grid");
        let v0: f64 = g.vol.iter().sum();
        let g2 = g.uniform_refine();
        assert_eq!(g2.nr, 12);
        assert_eq!(g2.ntheta, 16);
        assert_eq!(g2.nz, 20);
        assert_eq!(g2.level, 1);
        assert_eq!(g2.cell_count(), g.cell_count() * 8);

        // 体积守恒跨层级
        let v2: f64 = g2.vol.iter().sum();
        let domain = std::f64::consts::PI * 0.03f64.powi(2) * 0.04;
        assert!(close(v0, domain, 1e-14));
        assert!(close(v2, domain, 1e-14));

        // 面积守恒：单位立体角·轴向长度的外缘 A_r 跨层级不变（= R），
        // A_z 圆盘不变
        let dt1 = std::f64::consts::TAU / 12.0;
        let dt2 = std::f64::consts::TAU / 24.0;
        assert!(close(
            g2.ar[g2.nr] / (dt2 * (0.04 / 20.0)),
            g.ar[g.nr] / (dt1 * (0.04 / 10.0)),
            1e-12
        ));
        assert!(close(g2.az[g2.nz], g.az[g.nz], 1e-12));

        // 链式加密层级累积
        let g3 = g2.uniform_refine();
        assert_eq!(g3.level, 2);
    }

    #[test]
    fn sector_refinement_covers_requested_range() {
        let g = CylGrid3D::new(10, 24, 10, 0.03, 0.03).expect("grid");
        let dt = std::f64::consts::TAU / 24.0;
        // 扇区 [30°, 90°] = 角块 2..=6（0 基，dt=15°）→ 加密 4 倍
        let (sub, lo, hi) = g
            .refine_sector(30.0f64.to_radians(), 90.0f64.to_radians(), 4)
            .expect("sector");
        assert!(close(lo, 2.0 * dt, 1e-12));
        assert!(close(hi, 6.0 * dt, 1e-12));
        assert_eq!(sub.ntheta, 4 * 4);
        assert_eq!(sub.nr, 10);
        assert_eq!(sub.nz, 10);
        // 子网格体积守恒
        let v: f64 = sub.vol.iter().sum();
        let dr = 0.03 / 10.0;
        let expect = 0.5 * 0.03f64.powi(2) * (hi - lo) * 0.03;
        assert!(close(v, expect, 1e-12), "sector vol {v} vs {expect}");
        assert!(close(dr, 0.003, 1e-15));
    }

    #[test]
    fn invalid_constructions_rejected() {
        assert!(CylGrid3D::new(0, 4, 4, 1.0, 1.0).is_none());
        assert!(CylGrid3D::new(4, 0, 4, 1.0, 1.0).is_none());
        assert!(CylGrid3D::new(4, 4, 4, -1.0, 1.0).is_none());
        assert!(CylGrid3D::new(4, 4, 4, 1.0, 0.0).is_none());
        // 非法扇区参数
        let g = CylGrid3D::new(4, 8, 4, 1.0, 1.0).expect("grid");
        assert!(g.refine_sector(1.0, 1.0, 2).is_none());
        assert!(g.refine_sector(0.0, 1.0, 0).is_none());
        assert!(g.refine_sector(0.0, 1.0, 16).is_none());
    }
}
