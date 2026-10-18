//! TensorLab L1 — 纯 Rust 线性代数（行主序矩阵 / LU 分解）。
//!
//! L1 不依赖内核：m 语义的数值计算（`inv`、`A\b`、`eig`、`fft` 等）由
//! RunMat 内核在 L2 之上执行；本层为宿主侧（统计、预览、轻量计算）
//! 提供无内核的小型数值工具。

/// 行主序稠密矩阵
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Matrix {
    pub fn from_rows(rows: &[&[f64]]) -> Self {
        let cols = rows.first().map(|r| r.len()).unwrap_or(0);
        Matrix {
            rows: rows.len(),
            cols,
            data: rows.iter().flat_map(|r| r.iter().copied()).collect(),
        }
    }

    pub fn zeros(rows: usize, cols: usize) -> Self {
        Matrix {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n, n);
        for i in 0..n {
            m.data[i * n + i] = 1.0;
        }
        m
    }

    pub fn get(&self, r: usize, c: usize) -> f64 {
        self.data[r * self.cols + c]
    }

    fn set(&mut self, r: usize, c: usize, v: f64) {
        self.data[r * self.cols + c] = v;
    }
}

/// LU 分解（部分主元）。返回 (L\U 合并矩阵, 主元排列, 奇偶符号)。
/// 若矩阵奇异则返回 None（仅在 U 对角出现 0 时）。
pub fn lu(a: &Matrix) -> Option<(Matrix, Vec<usize>, f64)> {
    let n = a.rows;
    debug_assert_eq!(n, a.cols, "lu requires a square matrix");
    let mut m = a.clone();
    let mut perm: Vec<usize> = (0..n).collect();
    let mut sign = 1.0;

    for k in 0..n {
        // 选主元
        let (p, _) = (k..n).fold((k, f64::NEG_INFINITY), |(bi, bv), r| {
            let v = m.get(r, k).abs();
            if v > bv {
                (r, v)
            } else {
                (bi, bv)
            }
        });
        if m.get(p, k).abs() < 1e-12 {
            return None; // 奇异
        }
        if p != k {
            for c in 0..n {
                let (a1, a2) = (m.get(k, c), m.get(p, c));
                m.set(k, c, a2);
                m.set(p, c, a1);
            }
            perm.swap(k, p);
            sign = -sign;
        }
        for i in (k + 1)..n {
            let factor = m.get(i, k) / m.get(k, k);
            m.set(i, k, factor);
            for j in (k + 1)..n {
                let v = m.get(i, j) - factor * m.get(k, j);
                m.set(i, j, v);
            }
        }
    }
    Some((m, perm, sign))
}

/// 行列式 = 置换符号 × U 对角乘积
pub fn det(a: &Matrix) -> Option<f64> {
    let (m, _, sign) = lu(a)?;
    let diag: f64 = (0..m.rows).map(|i| m.get(i, i)).product();
    Some(sign * diag)
}

/// 求解 A x = b（返回解向量）
pub fn solve(a: &Matrix, b: &[f64]) -> Option<Vec<f64>> {
    let n = a.rows;
    let (lu_m, perm, _) = lu(a)?;
    let mut pb: Vec<f64> = perm.iter().map(|&i| b[i]).collect();

    // 前代 L y = Pb（L 单位下三角）
    for i in 0..n {
        for j in 0..i {
            pb[i] -= lu_m.get(i, j) * pb[j];
        }
    }
    // 回代 U x = y
    for i in (0..n).rev() {
        for j in (i + 1)..n {
            pb[i] -= lu_m.get(i, j) * pb[j];
        }
        pb[i] /= lu_m.get(i, i);
    }
    Some(pb)
}

/// 矩阵逆（逐列求解 A x = e_c）
pub fn inv(a: &Matrix) -> Option<Matrix> {
    let n = a.rows;
    let mut out = Matrix::zeros(n, n);
    for c in 0..n {
        let mut e = vec![0.0; n];
        e[c] = 1.0;
        let x = solve(a, &e)?;
        for r in 0..n {
            out.set(r, c, x[r]);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn det_matches_known_values() {
        let a = Matrix::from_rows(&[&[2.0, 1.0], &[1.0, 3.0]]);
        assert!((det(&a).unwrap() - 5.0).abs() < 1e-10);
        // magic(4) 奇异：精确行列式为 0；浮点 LU 可能给出极小残差值
        let s = Matrix::from_rows(&[
            &[16.0, 2.0, 3.0, 13.0],
            &[5.0, 11.0, 10.0, 8.0],
            &[9.0, 7.0, 6.0, 12.0],
            &[4.0, 14.0, 15.0, 1.0],
        ]);
        let d = det(&s);
        assert!(
            d.is_none() || d.unwrap().abs() < 1e-9,
            "magic(4) is singular, got {d:?}"
        );
    }

    #[test]
    fn solve_and_inv_roundtrip() {
        let a = Matrix::from_rows(&[&[4.0, 7.0], &[2.0, 6.0]]);
        let x = solve(&a, &[1.0, 2.0]).unwrap();
        // A * x ≈ b
        let b1 = 4.0 * x[0] + 7.0 * x[1];
        let b2 = 2.0 * x[0] + 6.0 * x[1];
        assert!((b1 - 1.0).abs() < 1e-10 && (b2 - 2.0).abs() < 1e-10);

        let ai = inv(&a).unwrap();
        // A * A⁻¹ ≈ I
        for i in 0..2 {
            for j in 0..2 {
                let v: f64 = (0..2).map(|k| a.get(i, k) * ai.get(k, j)).sum();
                let expect = if i == j { 1.0 } else { 0.0 };
                assert!((v - expect).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn identity_and_zeros_constructors() {
        let z = Matrix::zeros(2, 3);
        assert_eq!((z.rows, z.cols, z.data.len()), (2, 3, 6));
        assert!(z.data.iter().all(|v| *v == 0.0));

        let id = Matrix::identity(3);
        for i in 0..3 {
            for j in 0..3 {
                assert_eq!(id.get(i, j), if i == j { 1.0 } else { 0.0 });
            }
        }
    }

    #[test]
    fn from_rows_infers_column_count() {
        let m = Matrix::from_rows(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
        assert_eq!((m.rows, m.cols), (2, 3));
        assert_eq!(m.get(1, 2), 6.0);
    }

    #[test]
    fn det_sign_flips_on_row_swap() {
        // 单位阵行列式 = 1；交换两行后应为 -1
        let swapped = Matrix::from_rows(&[&[0.0, 1.0], &[1.0, 0.0]]);
        assert!((det(&swapped).unwrap() + 1.0).abs() < 1e-12);
        assert!((det(&Matrix::identity(4)).unwrap() - 1.0).abs() < 1e-12);
        // 对角阵行列式 = 对角积
        let d = Matrix::from_rows(&[&[3.0, 0.0], &[0.0, 4.0]]);
        assert!((det(&d).unwrap() - 12.0).abs() < 1e-12);
    }

    #[test]
    fn solve_3x3_and_inv_roundtrip() {
        let a = Matrix::from_rows(&[&[2.0, 1.0, 1.0], &[1.0, 3.0, 2.0], &[1.0, 0.0, 0.0]]);
        let b = vec![4.0, 5.0, 6.0];
        let x = solve(&a, &b).expect("solve");
        for i in 0..3 {
            let lhs: f64 = (0..3).map(|j| a.get(i, j) * x[j]).sum();
            assert!((lhs - b[i]).abs() < 1e-9, "row {i}: {lhs} vs {}", b[i]);
        }
        let ai = inv(&a).expect("inv");
        for i in 0..3 {
            for j in 0..3 {
                let v: f64 = (0..3).map(|k| a.get(i, k) * ai.get(k, j)).sum();
                let expect = if i == j { 1.0 } else { 0.0 };
                assert!((v - expect).abs() < 1e-9, "({i},{j}) = {v}");
            }
        }
    }

    #[test]
    fn singular_systems_report_none() {
        // 秩 1 矩阵：第二行是第一行的 2 倍
        let s = Matrix::from_rows(&[&[1.0, 2.0], &[2.0, 4.0]]);
        assert!(lu(&s).is_none(), "singular lu must be None");
        assert!(det(&s).is_none(), "singular det must be None");
        assert!(inv(&s).is_none(), "singular inv must be None");
        assert!(
            solve(&s, &[1.0, 1.0]).is_none(),
            "singular solve must be None"
        );
    }

    #[test]
    fn inv_of_identity_is_identity() {
        let ai = inv(&Matrix::identity(5)).expect("inv");
        for i in 0..5 {
            for j in 0..5 {
                let expect = if i == j { 1.0 } else { 0.0 };
                assert!((ai.get(i, j) - expect).abs() < 1e-12);
            }
        }
    }
}
