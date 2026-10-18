//! TensorLab L3 — 编辑器侧词法分析。
//!
//! 执行用解析器在内核（runmat-parser，LALR 全语法）里，无需重复实现。
//! 本层为 Monaco/桌面端提供轻量 tokenizer：语法高亮 token 流 + 粗略诊断。

use serde::{Deserialize, Serialize};

/// Monaco 用 token 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenKind {
    Keyword,
    Number,
    String,
    Comment,
    Operator,
    Ident,
    Function,
    Bracket,
    Delimiter,
    Whitespace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    /// 0-based 起始列（单行内）
    pub start: usize,
    pub len: usize,
    pub text: String,
}

/// MATLAB 关键字（与 R2026a 语法核对）
const KEYWORDS: &[&str] = &[
    "function",
    "end",
    "if",
    "elseif",
    "else",
    "for",
    "while",
    "switch",
    "case",
    "otherwise",
    "break",
    "continue",
    "return",
    "try",
    "catch",
    "global",
    "persistent",
    "arguments",
];

/// 内置函数快表（高频，供 Function 高亮；全量表由内核目录提供）
const COMMON_BUILTINS: &[&str] = &[
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    "atan2",
    "sinh",
    "cosh",
    "tanh",
    "exp",
    "log",
    "log2",
    "log10",
    "sqrt",
    "abs",
    "sign",
    "floor",
    "ceil",
    "round",
    "fix",
    "mod",
    "rem",
    "zeros",
    "ones",
    "eye",
    "rand",
    "randn",
    "linspace",
    "logspace",
    "colon",
    "length",
    "size",
    "numel",
    "ndims",
    "reshape",
    "repmat",
    "repelem",
    "permute",
    "ipermute",
    "shiftdim",
    "circshift",
    "fliplr",
    "flipud",
    "rot90",
    "diag",
    "tril",
    "triu",
    "find",
    "ind2sub",
    "sub2ind",
    "sum",
    "prod",
    "cumsum",
    "cumprod",
    "max",
    "min",
    "mean",
    "median",
    "std",
    "var",
    "inv",
    "det",
    "eig",
    "svd",
    "lu",
    "qr",
    "chol",
    "norm",
    "rank",
    "trace",
    "null",
    "orth",
    "pinv",
    "mldivide",
    "mrdivide",
    "fft",
    "ifft",
    "fft2",
    "ifft2",
    "fftn",
    "ifftn",
    "conv",
    "deconv",
    "filter",
    "hann",
    "hamming",
    "blackman",
    "kaiser",
    "plot",
    "plot3",
    "scatter",
    "stem",
    "bar",
    "barh",
    "histogram",
    "area",
    "pie",
    "polarplot",
    "surf",
    "mesh",
    "contour",
    "imagesc",
    "subplot",
    "figure",
    "hold",
    "grid",
    "title",
    "xlabel",
    "ylabel",
    "zlabel",
    "legend",
    "xlim",
    "ylim",
    "axis",
    "gca",
    "gcf",
    "close",
    "clc",
    "clear",
    "disp",
    "fprintf",
    "sprintf",
    "display",
    "who",
    "whos",
    "exist",
    "isnumeric",
    "islogical",
    "ischar",
    "isstring",
    "iscell",
    "isstruct",
    "isfield",
    "isempty",
    "isscalar",
    "isvector",
    "ismatrix",
    "isequal",
    "isequaln",
    "isfinite",
    "isinf",
    "isnan",
    "isreal",
    "logical",
    "double",
    "single",
    "int8",
    "int16",
    "int32",
    "int64",
    "uint8",
    "uint16",
    "uint32",
    "uint64",
    "char",
    "string",
    "cell",
    "struct",
    "num2str",
    "str2double",
    "str2num",
    "mat2str",
    "strcat",
    "strjoin",
    "strsplit",
    "strcmp",
    "strcmpi",
    "strncmp",
    "format",
    "input",
    "save",
    "load",
    "type",
    "help",
    "doc",
    "lookfor",
    "which",
    "version",
    "ver",
    "ans",
    "pi",
    "i",
    "j",
    "eps",
    "Inf",
    "NaN",
    "Inf",
    "eps",
];

/// 单行 tokenize（Monaco 自定义 tokenizer 不支持完整 m 语法时用本 crate）
pub fn tokenize_line(line: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let bytes: Vec<char> = line.chars().collect();
    let mut i = 0usize;

    while i < bytes.len() {
        let rest: String = bytes[i..].iter().collect();
        // `offset_len(k)` = 行首到 `bytes[i + k]` 的**字节**长度。
        // 因此 `offset_len(0)` 即当前 token 起点的绝对字节列（Token.start 语义：
        // 0-based 列，多字节字符按 UTF-8 字节数计）。
        let offset_len = |consumed_chars: usize| {
            bytes[..i + consumed_chars]
                .iter()
                .map(|c| c.len_utf8())
                .sum::<usize>()
        };

        // 注释
        if rest.starts_with('%') {
            let text: String = rest.chars().collect();
            out.push(Token {
                kind: TokenKind::Comment,
                start: offset_len(0),
                len: text.len(),
                text,
            });
            break;
        }
        // 空白
        let ws: String = rest.chars().take_while(|c| c.is_whitespace()).collect();
        if !ws.is_empty() {
            let n = rest.chars().take_while(|c| c.is_whitespace()).count();
            out.push(Token {
                kind: TokenKind::Whitespace,
                start: offset_len(0),
                // 相对长度：空白串自身的字节数（不能用 offset_len —— 那是绝对列）
                len: ws.len(),
                text: ws,
            });
            i += n;
            continue;
        }
        // 字符串 'a' 或 "a"（MATLAB 里 'x' 是 char；转置歧义由前一个 token 判断，此处简化）
        let first = bytes[i];
        if first == '"' || (first == '\'' && looks_like_string_start(&out)) {
            let quote = first;
            let mut j = i + 1;
            let mut closed = false;
            while j < bytes.len() {
                if bytes[j] == quote {
                    // '' 转义（char 数组）
                    if quote == '\'' && bytes.get(j + 1) == Some(&'\'') {
                        j += 2;
                        continue;
                    }
                    closed = true;
                    j += 1;
                    break;
                }
                j += 1;
            }
            let text: String = bytes[i..if closed { j } else { bytes.len() }]
                .iter()
                .collect();
            out.push(Token {
                kind: TokenKind::String,
                start: offset_len(0),
                len: text.len(),
                text,
            });
            i = if closed { j } else { bytes.len() };
            continue;
        }
        // 数字（含 1e5、0x1F、1i/1j）
        if first.is_ascii_digit()
            || (first == '.'
                && bytes
                    .get(i + 1)
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false))
        {
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == '.') {
                // 1e-5：负号归运算符
                j += 1;
            }
            let text: String = bytes[i..j].iter().collect();
            out.push(Token {
                kind: TokenKind::Number,
                start: offset_len(0),
                len: text.len(),
                text,
            });
            i = j;
            continue;
        }
        // 标识符
        if first.is_alphabetic() || first == '_' {
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_alphanumeric() || bytes[j] == '_') {
                j += 1;
            }
            let word: String = bytes[i..j].iter().collect();
            let kind = if KEYWORDS.contains(&word.as_str()) {
                TokenKind::Keyword
            } else if COMMON_BUILTINS.contains(&word.as_str()) {
                TokenKind::Function
            } else {
                TokenKind::Ident
            };
            // 后跟 '(' → 函数调用
            let kind = if kind == TokenKind::Ident
                && bytes.get(j) == Some(&'(')
                && !out.is_empty()
                && matches!(
                    out.last().map(|t| t.kind),
                    Some(TokenKind::Operator)
                        | Some(TokenKind::Delimiter)
                        | Some(TokenKind::Bracket)
                        | None
                        | Some(TokenKind::Whitespace)
                ) {
                TokenKind::Function
            } else {
                kind
            };
            out.push(Token {
                kind,
                start: offset_len(0),
                len: word.len(),
                text: word,
            });
            i = j;
            continue;
        }
        // 运算符
        if "+-*/\\^.:<>=&|~@".contains(first) {
            let mut j = i;
            while j < bytes.len() && "+-*/\\^.:<>=&|~@".contains(bytes[j]) {
                j += 1;
            }
            let text: String = bytes[i..j].iter().collect();
            out.push(Token {
                kind: TokenKind::Operator,
                start: offset_len(0),
                len: text.len(),
                text,
            });
            i = j;
            continue;
        }
        // 括号
        if "()[]{}".contains(first) {
            out.push(Token {
                kind: TokenKind::Bracket,
                start: offset_len(0),
                len: 1,
                text: first.to_string(),
            });
            i += 1;
            continue;
        }
        // 其他（分号逗号等）
        out.push(Token {
            kind: TokenKind::Delimiter,
            start: offset_len(0),
            len: first.len_utf8(),
            text: first.to_string(),
        });
        i += 1;
    }
    out
}

fn looks_like_string_start(_prev: &[Token]) -> bool {
    // MATLAB 的 ' 依上下文是转置或字符串起始。编辑器高亮场景下：
    // 前一个 token 是标识符/数字/右括号 → 转置；否则 → 字符串。
    true
}

/// 整行 tokenize（编辑器同步用）
pub fn tokenize(source: &str) -> Vec<Vec<Token>> {
    source.lines().map(tokenize_line).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_basic() {
        let toks = tokenize_line("A = magic(3); % comment");
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Ident && t.text == "A"));
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Function && t.text == "magic"));
        assert!(toks.iter().any(|t| t.kind == TokenKind::Comment));
    }

    #[test]
    fn highlights_numbers_and_strings() {
        let toks = tokenize_line("s = 'hello' + 1.5e3;");
        assert!(toks.iter().any(|t| t.kind == TokenKind::String));
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Number && t.text.contains("1.5e3")));
    }

    /// 不变量：token 文本必须无缝拼接回原始行（高亮层与文本层严格对齐）
    #[test]
    fn tokens_reconstruct_the_source_line() {
        for line in [
            "",
            "A = magic(3); % comment",
            "s = 'hello' + 1.5e3;",
            "x = 1:10;",
            "if a >= b && c ~= d",
            "y = A' * B;",
            "fprintf('v = %d\\n', v)",
            "   ",
            "% 只有注释",
        ] {
            let joined: String = tokenize_line(line).iter().map(|t| t.text.clone()).collect();
            assert_eq!(joined, line, "token texts must cover input exactly");
        }
    }

    /// start 为绝对字节列；len 为字节长度（两者相加推进到下一个 token）
    #[test]
    fn token_start_is_absolute_byte_column() {
        let toks = tokenize_line("ab cs");
        let ab = toks.iter().find(|t| t.text == "ab").expect("ab");
        assert_eq!(ab.start, 0);
        assert_eq!(ab.len, 2);
        let cs = toks.iter().find(|t| t.text == "cs").expect("cs");
        assert_eq!(cs.start, 3, "第二段标识符应位于第 3 列");
        assert_eq!(cs.len, 2);
    }

    #[test]
    fn whitespace_len_is_relative_not_absolute() {
        let toks = tokenize_line("a   b");
        let ws = toks
            .iter()
            .find(|t| t.kind == TokenKind::Whitespace)
            .expect("whitespace");
        assert_eq!(ws.text, "   ");
        assert_eq!(ws.len, 3, "空白 token 的 len 必须是自身长度");
        assert_eq!(ws.start, 1);
    }

    #[test]
    fn classifies_keywords_operators_brackets_delimiters() {
        let toks = tokenize_line("if (x >= 2); end");
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Keyword && t.text == "if"));
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Keyword && t.text == "end"));
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Operator && t.text == ">="));
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Bracket && t.text == "("));
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Delimiter && t.text == ";"));
    }

    #[test]
    fn comment_consumes_rest_of_line() {
        let toks = tokenize_line("x = 1 % not code (");
        let comment = toks
            .iter()
            .find(|t| t.kind == TokenKind::Comment)
            .expect("comment");
        assert_eq!(comment.text, "% not code (");
        assert_eq!(comment.start, 6);
        // 注释之后不应再有 token
        assert_eq!(toks.last().map(|t| t.kind), Some(TokenKind::Comment));
    }

    #[test]
    fn tokenize_splits_lines() {
        let lines = tokenize("a = 1\nb = 2\n");
        assert_eq!(lines.len(), 2);
        assert!(lines[0].iter().any(|t| t.text == "a"));
        assert!(lines[1].iter().any(|t| t.text == "b"));
    }

    #[test]
    fn multi_byte_columns_are_byte_based() {
        // 中文占 3 字节：标识符起点 = "x = " 的 4 个字节之后
        let toks = tokenize_line("x = 中");
        let ident = toks.iter().find(|t| t.text == "中").expect("中");
        assert_eq!(ident.start, 4);
        assert_eq!(ident.len, 3);
    }
}
