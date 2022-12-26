//! 正規表現の式をパースし、抽象構文気に変換
use std::{
    error::Error,
    fmt::{self, Display},
    mem::take,
};


/// 抽象構文木を表現するための型
#[derive(Debug)]
pub enum AST {
    Char(char),
    Plus(Box<AST>),
    Star(Box<AST>),
    Question(Box<AST>),
    Or(Box<AST>, Box<AST>),
    Seq(Vec<AST>),
}

/// パースエラーを表すための型
#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char), // 誤ったエスケープシーケンス
    InvalidRightParen(usize), // 開き括弧なし
    NoPrev(usize), // +, |, *, ?の前に式がない
    NoRightParen, // 閉じ括弧なし
    Empty, // 空のパターン
}

/// パースエラーを表示するために、Displayトレイトを実装
impl Display for ParseError {
}

/// 特殊文字のエスケープ
fn parse_escape(pos: usize, c: char) -> Result<AST, ParseError> {

    match c {
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(AST::Char(c)),
        _ => {
            let err = ParseError::InvalidEscape(pos, c);
            Err(err)
        }
    }

}