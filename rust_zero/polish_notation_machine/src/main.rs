use nom::{
    branch::alt,
    character::complete::{char, one_of},
    error::ErrorKind,
    multi::{many0, many1},
    IResult,
};

use rustyline::Editor;

#[derive(Debug)]
enum Expr {
    Num(u64),                  // 数値
    Add(Box<Expr>, Box<Expr>), // 加算
    Mul(Box<Expr>, Box<Expr>), // 乗算
}

// 数値のパーサ
fn parse_num(c: &str) -> IResult<&str, Expr> {
    // 0~9のいずれかが１個上
    // 正規表現で[0..9]+
    //
    // 例えば、 many1(one_of("0123456789"))("123abc")は
    // Ok("abc", vec!['1', '2', '3'])を返す
    let (c1, v) = many1(one_of("0123456789"))(c)?;
    let var: String = v.into_iter().collect(); // Vec<char>をStringに変換

    // Stringをu64に変換
    if let Ok(n) = var.parse::<u64>() {
        // 数値変換に成功した場合は、次の文字列とExpr::Numを返す
        Ok((c1, Expr::Num(n)))
    } else {
        let err = nom::error::Error::new(c, ErrorKind::Fail);
        Err(nom::Err::Failure(err))
    }
}

// 演算子をパースする
fn parse_op(c: &str) -> IResult<&str, Expr> {
    // +か*のどちらか
    let (c, op) = one_of("+*")(c)?;
    let (c, e1) = parse_expr(c)?; // 1つ目の式をパース
    let (c, e2) = parse_expr(c)?; // ２つ目の式をパース

    if op == '+' {
        Ok((c, Expr::Add(Box::new(e1), Box::new(e2)))) // Addを返す
    } else {
        Ok((c, Expr::Mul(Box::new(e1), Box::new(e2)))) // Addを返す
    }
}

// 式のトップをパースする
fn parse_expr(c: &str) -> IResult<&str, Expr> {
    // 0個以上のホワイトスペースをスキップ
    let (c, _) = many0(char(' '))(c)?;

    // parse_numかparse_cpをパース
    let result = alt((parse_num, parse_op))(c)?;
    Ok(result)
}

fn parse(c: &str) -> Option<Expr> {
    match parse_expr(c) {
        Ok((_, e)) => {
            println!("AST: {:?}", e);
            Some(e)
        }
        Err(e) => {
            println!("{e}");
            None
        }
    }
}

fn eval(e: &Expr) -> u64 {
    match e {
        Expr::Num(n) => *n,
        Expr::Add(e1, e2) => eval(e1) + eval(e2),
        Expr::Mul(e1, e2) => eval(e1) * eval(e2),
    }
}

fn main() {
    let mut rl = Editor::<()>::new().unwrap();
    loop {
        // 1行読み込んでパースし、成功すれば評価
        if let Ok(readline) = rl.readline(">> ") {
            if let Some(e) = parse(&readline) {
                println!("result: {}", eval(&e));
            }
        } else {
            break;
        }
    }
}
