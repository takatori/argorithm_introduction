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
