use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, multispace0, multispace1},
    error::VerboseError,
    sequence::delimited,
    IResult,
};
use std::fmt;

/// 抽象構文木
pub enum Expr {
    Let(LetExpr),     // let式
    If(IfExpr),       // if式
    Split(SplitExpr), // split式
    Free(FreeExpr),   // free文
    App(AppExpr),     // 関数適用
    Var(String),      // 変数
    QVal(QValExpr),   // 値
}

/// 関数適用
#[derive(Debug)]
pub struct AppExpr {
    pub expr1: Box<Expr>,
    pub expr2: Box<Expr>,
}

/// if式
#[derive(Debug)]
pub struct IfExpr {
    pub cond_expr: Box<Expr>,
    pub then_expr: Box<Expr>,
    pub else_expr: Box<Expr>,
}

/// split式
#[derive(Debug)]
pub struct SplitExpr {
    pub expr: Box<Expr>,
    pub left: String,
    pub rigth: String,
    pub body: Box<Expr>,
}

/// let式
#[derive(Debug)]
pub struct LetExpr {
    pub var: String,
    pub ty: TypeExpr,
    pub expr1: Box<Expr>,
    pub expr2: Box<Expr>,
}

/// 値。真偽値、関数、ペア値などになる
#[derive(Debug)]
pub enum ValExpr {
    Bool(bool),                 // 真偽値リテラル
    Pair(Box<Expr>, Box<Expr>), // ペア
    Fun(FnExpr),                // 関数(λ抽象)
}

/// 修飾子
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Qual {
    Lin, // 線形型
    Un,  // 制約のない一般的な型
}

/// 修飾子付き値
#[derive(Debug)]
pub struct QValExpr {
    pub qual: Qual,
    pub val: ValExpr,
}

/// 関数
#[derive(Debug)]
pub struct FnExpr {
    pub var: String,
    pub ty: TypeExpr,
    pub expr: Box<Expr>,
}

/// free文
#[derive(Debug)]
pub struct FreeExpr {
    pub var: String,
    pub expr: Box<Expr>,
}

/// 修飾子付き型
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TypeExpr {
    pub qual: Qual,
    pub prim: PrimType,
}

/// プリミティブ型
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PrimType {
    Bool,                                // 真偽値型
    Pair(Box<TypeExpr>, Box<TypeExpr>),  // ペア型
    Arrow(Box<TypeExpr>, Box<TypeExpr>), // 関数型
}

impl fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.qual == Qual::Lin {
            write!(f, "lin {}", self.prim)
        } else {
            write!(f, "un {}", self.prime)
        }
    }
}

impl fmt::Display for PrimType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimType::Bool => write!(f, "bool"),
            PrimType::Pair(t1, t2) => write!(f, "({t1} * {t2})"),
            PrimType::Arrow(t1, t2) => write!(f, "({t1} -> {t2})"),
        }
    }
}
