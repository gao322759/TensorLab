//! AST：m-script 抽象语法树。

use std::sync::Arc;
use tl_types::C64;

#[derive(Debug)]
pub struct Program {
    pub items: Vec<TopItem>,
}

#[derive(Debug)]
pub enum TopItem {
    Stmt(Stmt),
    Func(Arc<FunctionDef>),
}

#[derive(Debug)]
pub struct FunctionDef {
    pub name: String,
    /// None 表示 `~`（忽略的输出）
    pub outputs: Vec<Option<String>>,
    pub inputs: Vec<Option<String>>,
    pub body: Vec<Stmt>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketKind {
    Paren,
    Brace,
}

#[derive(Debug)]
pub enum Stmt {
    Expr { expr: Expr, suppressed: bool, line: usize },
    Assign { target: Target, value: Expr, suppressed: bool, line: usize },
    If { arms: Vec<Arm>, else_body: Vec<Stmt>, line: usize },
    While { cond: Expr, body: Vec<Stmt>, line: usize },
    For { var: String, iter: Expr, body: Vec<Stmt>, line: usize },
    Switch { expr: Expr, cases: Vec<SwitchCase>, otherwise: Vec<Stmt>, line: usize },
    Return { line: usize },
    Break { line: usize },
    Continue { line: usize },
}

#[derive(Debug)]
pub struct Arm {
    pub cond: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct SwitchCase {
    pub matches: Vec<Expr>,
    pub body: Vec<Stmt>,
}

/// 赋值目标（左值）
#[derive(Debug)]
pub enum Target {
    Var(String),
    Index { base: Arc<Target>, kind: BracketKind, indices: Vec<Expr> },
    Field { base: Arc<Target>, name: String },
    /// `[a, ~, b] = ...` 多目标
    Multi(Vec<Option<Target>>),
}

#[derive(Debug)]
pub enum Expr {
    Number(C64),
    Str(String),
    Bool(bool),
    Ident(String),
    /// [a b; c d] — 行内空格/逗号分隔，分号/换行分行
    MatrixLit { rows: Vec<Vec<Expr>>, line: usize },
    /// {a, b} 元胞字面量
    CellLit { rows: Vec<Vec<Expr>>, line: usize },
    /// a:b 或 a:step:b
    Colon { start: Box<Expr>, step: Option<Box<Expr>>, end: Box<Expr>, line: usize },
    Bin { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, line: usize },
    Un { op: UnOp, expr: Box<Expr>, line: usize },
    /// f(args) — 变量或函数在运行时分辨
    Call { name: String, args: Vec<Expr>, line: usize },
    Index { base: Box<Expr>, kind: BracketKind, indices: Vec<Expr>, line: usize },
    /// s.field
    Field { base: Box<Expr>, name: String, line: usize },
    /// 后缀转置：'（共轭）或 .'（非共轭）
    Transpose { base: Box<Expr>, conj: bool, line: usize },
    /// @f
    Handle(String),
    /// @(x,y) expr — 保存源码文本以便跨 Value 边界
    Lambda { params: Vec<String>, body_src: String, body: Arc<Expr>, line: usize },
    /// `end` 关键字（仅索引上下文）
    End,
    /// 多目标中的 `~` 占位
    TildeExpr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    LDiv,
    Pow,
    DotMul,
    DotDiv,
    DotLDiv,
    DotPow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    AndAnd,
    OrOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Pos,
    Not,
}
