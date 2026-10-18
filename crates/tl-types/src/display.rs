//! MATLAB 风格显示格式化：把内核 `runmat_value::Value` 渲染为控制台文本。
//!
//! 回显语义（无分号语句）由 L2 产出 `display_events`，本层负责把值变成 MATLAB
//! 风格文本。数值格式化委托内核的 `runmat_value::format_number`（与 format
//! short/long 联动，宿主无需自行实现）。

use runmat_value::Value;

/// 类名（用于回显头与变量面板）
pub fn class_name(v: &Value) -> String {
    match v {
        Value::Int(i) => i.class_name().to_string(),
        Value::Num(_) => "double".into(),
        Value::Complex(_, _) => "double".into(),
        Value::Bool(_) | Value::LogicalArray(_) => "logical".into(),
        Value::String(_) | Value::StringArray(_) => "string".into(),
        Value::CharArray(_) => "char".into(),
        Value::Tensor(_) => "double".into(),
        Value::ComplexTensor(_) => "double (complex)".into(),
        Value::SparseTensor(t) => t.class_name().to_string(),
        Value::Cell(_) => "cell".into(),
        Value::Struct(_) => "struct".into(),
        Value::FunctionHandle(_) | Value::ExternalFunctionHandle(_) => "function_handle".into(),
        Value::Closure(_) => "function_handle".into(),
        Value::MException(_) => "MException".into(),
        Value::GpuTensor(_) => "gpuArray".into(),
        Value::Symbolic(_) | Value::SymbolicArray(_) => "sym".into(),
        other => format!("{}", runmat_class_fallback(other)),
    }
}

fn runmat_class_fallback(v: &Value) -> &'static str {
    // 与内核 class 语义对齐的兜底（对象/句柄等）
    match v {
        Value::Object(_) | Value::ObjectArray(_) => "object",
        Value::HandleObject(_) => "handle",
        _ => "value",
    }
}

/// 大小 [rows, cols]（N-D 取前两维）
pub fn size_of(v: &Value) -> (usize, usize) {
    match v {
        Value::Tensor(t) => (t.rows, t.cols),
        Value::ComplexTensor(t) => (t.rows, t.cols),
        Value::SparseTensor(t) => (t.rows, t.cols),
        Value::CharArray(c) => shape_2d(&c.shape),
        Value::LogicalArray(l) => shape_2d(&l.shape),
        Value::StringArray(s) => (s.rows, s.cols),
        Value::Cell(c) => (c.rows, c.cols),
        _ => (1, 1),
    }
}

fn shape_2d(shape: &[usize]) -> (usize, usize) {
    match shape {
        [] => (0, 0),
        [r] => (*r, 1),
        [r, c, ..] => (*r, *c),
    }
}

fn fmt_num(x: f64) -> String {
    runmat_value::format_number(x)
}

fn fmt_complex(re: f64, im: f64) -> String {
    if im == 0.0 {
        fmt_num(re)
    } else {
        let sign = if im < 0.0 { '-' } else { '+' };
        format!("{} {} {}i", fmt_num(re), sign, fmt_num(im.abs()))
    }
}

fn scalar_of(ns: runmat_value::NumericScalar) -> f64 {
    use runmat_value::NumericScalar;
    match ns {
        NumericScalar::F64(x) => x,
        NumericScalar::F32(x) => x as f64,
        _ => 0.0,
    }
}

/// 标量/短值文本（回显右值、元胞内容）
pub fn scalar_text(v: &Value) -> Option<String> {
    match v {
        Value::Num(x) => Some(fmt_num(*x)),
        Value::Int(i) => Some(i.decimal_string()),
        Value::Bool(b) => Some(if *b { "1" } else { "0" }.to_string()),
        Value::Complex(re, im) => Some(fmt_complex(*re, *im)),
        Value::String(s) => Some(format!("\"{s}\"")),
        Value::CharArray(c) => c.row_string().map(|s| format!("'{s}'")),
        Value::FunctionHandle(f) => Some(format!("@{f}")),
        Value::ExternalFunctionHandle(f) => Some(format!("@{f}")),
        _ => None,
    }
}

/// 主体文本：回显/`disp` 的值部分（不含 `x =` 头）
pub fn body_text(v: &Value) -> String {
    if let Some(s) = scalar_text(v) {
        return s;
    }
    match v {
        Value::Tensor(t) => {
            let cells: Vec<String> = (0..t.len())
                .map(|i| {
                    fmt_num(scalar_of(
                        t.numeric_value_at(i)
                            .unwrap_or(runmat_value::NumericScalar::F64(f64::NAN)),
                    ))
                })
                .collect();
            grid_body(&cells, t.rows, t.cols)
        }
        Value::ComplexTensor(t) => {
            let cells: Vec<String> = (0..t.len())
                .map(|i| {
                    t.numeric_value_at(i)
                        .map(|(re, im)| fmt_complex(scalar_of(re), scalar_of(im)))
                        .unwrap_or_else(|| fmt_num(f64::NAN))
                })
                .collect();
            grid_body(&cells, t.rows, t.cols)
        }
        Value::LogicalArray(l) => {
            let cells: Vec<String> = l
                .data
                .iter()
                .map(|&b| if b != 0 { "1".into() } else { "0".into() })
                .collect();
            let (r, c) = shape_2d(&l.shape);
            grid_body(&cells, r, c)
        }
        Value::StringArray(s) => {
            let mut out = format!("{}x{} string array\n\n", s.rows, s.cols);
            for r in 0..s.rows {
                for c in 0..s.cols {
                    let text = s.data.get(r + c * s.rows).cloned().unwrap_or_default();
                    out.push_str(&format!("\"{text}\"  "));
                }
                out.push('\n');
            }
            out.trim_end().to_string()
        }
        Value::Cell(c) => {
            if c.rows * c.cols == 0 {
                return "{}".to_string();
            }
            let cells: Vec<String> = c
                .to_column_major()
                .iter()
                .map(|inner| format!("{{{}}}", compact(inner)))
                .collect();
            grid_body(&cells, c.rows, c.cols)
        }
        Value::Struct(s) => {
            if s.fields.is_empty() {
                return "struct with no fields.".to_string();
            }
            let mut out = String::from("struct with fields:\n\n");
            for (k, val) in &s.fields {
                out.push_str(&format!("    {k}: {}\n", compact(val)));
            }
            out.trim_end().to_string()
        }
        Value::MException(e) => format!("MException {}: {}", e.identifier, e.message),
        Value::Closure(c) => format!("@{} <closure>", c.function_name),
        other => format!("<{}>", class_name(other)),
    }
}

fn grid_body(cells: &[String], rows: usize, cols: usize) -> String {
    if rows * cols == 0 {
        return "[]".to_string();
    }
    if rows == 1 && cols == 1 {
        return cells.first().cloned().unwrap_or_default();
    }
    let width = cells.iter().map(|s| s.len()).max().unwrap_or(0);
    let mut out = format!("\n{rows}x{cols} array\n\n");
    for r in 0..rows {
        let line: Vec<String> = (0..cols)
            .map(|c| cells.get(r + c * rows).cloned().unwrap_or_default())
            .map(|s| format!("{s:>width$}"))
            .collect();
        out.push_str(&line.join("  "));
        out.push('\n');
    }
    out.trim_end().to_string()
}

/// 变量面板单行预览
pub fn compact(v: &Value) -> String {
    let full = body_text(v);
    let one_line = full.replace('\n', " ");
    let mut s: String = one_line.chars().take(60).collect();
    if s.chars().count() < one_line.chars().count() {
        s.push('…');
    }
    s
}

/// 回显行：`x = ...`（矩阵主体换行时先头后体）
pub fn echo_line(label: &str, v: &Value) -> String {
    let body = body_text(v);
    if body.starts_with('\n') {
        format!("{label} = {body}")
    } else {
        format!("{label} =\n{body}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runmat_value::{CellArray, CharArray, IntValue, LogicalArray, StructValue, Tensor};

    fn tensor(data: Vec<f64>, shape: Vec<usize>) -> Value {
        Value::Tensor(Tensor::new(data, shape).expect("tensor"))
    }

    fn chars(s: &str) -> Value {
        let data: Vec<char> = s.chars().collect();
        let cols = data.len();
        Value::CharArray(CharArray::new(data, 1, cols).expect("char array"))
    }

    #[test]
    fn class_names_match_matlab() {
        assert_eq!(class_name(&Value::Num(1.0)), "double");
        assert_eq!(class_name(&Value::Complex(1.0, 2.0)), "double");
        assert_eq!(class_name(&tensor(vec![1.0], vec![1, 1])), "double");
        assert_eq!(class_name(&Value::Int(IntValue::I32(-7))), "int32");
        assert_eq!(class_name(&Value::Int(IntValue::U64(7))), "uint64");
        assert_eq!(class_name(&Value::Bool(true)), "logical");
        assert_eq!(
            class_name(&Value::LogicalArray(
                LogicalArray::new(vec![1, 0], vec![1, 2]).expect("logical")
            )),
            "logical"
        );
        assert_eq!(class_name(&Value::String("x".into())), "string");
        assert_eq!(class_name(&chars("ab")), "char");
        assert_eq!(class_name(&Value::Struct(StructValue::new())), "struct");
    }

    #[test]
    fn size_of_reports_rows_and_cols() {
        assert_eq!(size_of(&Value::Num(1.0)), (1, 1));
        assert_eq!(size_of(&tensor(vec![1.0; 6], vec![2, 3])), (2, 3));
        assert_eq!(size_of(&chars("abc")), (1, 3));
    }

    #[test]
    fn body_text_renders_scalars() {
        assert_eq!(body_text(&Value::Num(42.0)), "42");
        assert_eq!(body_text(&Value::Bool(true)), "1");
        assert_eq!(body_text(&Value::Bool(false)), "0");
        assert_eq!(body_text(&Value::Int(IntValue::I32(-7))), "-7");
        assert_eq!(body_text(&Value::String("hi".into())), "\"hi\"");
        assert_eq!(body_text(&Value::Complex(1.0, -2.0)), "1 - 2i");
        // 虚部为 0 时退化为实数
        assert_eq!(body_text(&Value::Complex(3.0, 0.0)), "3");
        // char 加单引号
        assert_eq!(body_text(&chars("hi")), "'hi'");
    }

    #[test]
    fn body_text_renders_matrix_grid_in_column_major_order() {
        // 列主序数据 [1 2 3 4] + 2x2 → 第 0 行 = (1, 3)，第 1 行 = (2, 4)
        let text = body_text(&tensor(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]));
        assert!(text.contains("2x2 array"), "{text}");
        let rows: Vec<&str> = text.lines().filter(|l| l.contains('1')).collect();
        assert!(
            rows.iter().any(|l| l.contains('3')),
            "expect row (1,3) present:\n{text}"
        );
    }

    #[test]
    fn body_text_renders_1x1_matrix_without_header() {
        let text = body_text(&tensor(vec![7.0], vec![1, 1]));
        assert_eq!(text, "7");
        assert!(!text.contains("array"));
    }

    #[test]
    fn body_text_renders_logical_array_and_cell() {
        let l = Value::LogicalArray(LogicalArray::new(vec![1, 0], vec![1, 2]).expect("logical"));
        let text = body_text(&l);
        assert!(text.contains('1') && text.contains('0'), "{text}");

        let c = Value::Cell(
            CellArray::new(vec![Value::Num(1.0), Value::String("a".into())], 1, 2).expect("cell"),
        );
        let ctext = body_text(&c);
        assert!(ctext.contains("{1}"), "{ctext}");
        assert!(ctext.contains("{\"a\"}"), "{ctext}");
    }

    #[test]
    fn body_text_renders_struct_with_fields() {
        let mut s = StructValue::new();
        s.fields.insert("a".into(), Value::Num(1.0));
        let text = body_text(&Value::Struct(s));
        assert!(text.contains("struct with fields"), "{text}");
        assert!(text.contains("a:"), "{text}");

        // 无字段
        assert_eq!(
            body_text(&Value::Struct(StructValue::new())),
            "struct with no fields."
        );
    }

    #[test]
    fn compact_truncates_long_output_with_ellipsis() {
        let big: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let s = compact(&tensor(big, vec![1, 200]));
        assert!(s.chars().count() <= 61, "len={} s={s}", s.chars().count());
        assert!(s.ends_with('…'), "{s}");
    }

    #[test]
    fn compact_keeps_short_output_intact() {
        assert_eq!(compact(&Value::Num(42.0)), "42");
    }

    #[test]
    fn echo_line_carries_label_and_value() {
        let s = echo_line("x", &Value::Num(42.0));
        assert!(s.contains("x ="), "{s}");
        assert!(s.contains("42"), "{s}");

        let m = echo_line("A", &tensor(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]));
        assert!(m.contains("A ="), "{m}");
        assert!(m.contains("2x2 array"), "{m}");
    }
}
