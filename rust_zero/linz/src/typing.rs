use crate::{
    helper::safe_add,
    parser::{self, PrimType, TypeExpr},
};
use std::{borrow::Cow, cmp::Ordering, collections::BTreeMap, mem};

/// 変数名から型へのマップ
/// Optionにしているのはlin型の変数を消費したことを表現するため
/// 値がNoneの場合は、その変数が一度使用されたことを意味する
type VarToType = BTreeMap<String, Option<parser::TypeExpr>>;

/// 型環境
/// 型環境とは、変数と型の対応付を保存するデータベースのこと
/// 型環境は、マップのスタックとして実装できる
/// 型環境をスタックとして実装することで、変数のスコープやシャドーイングを表現できる
#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct TypeEnvStack {
    // スタックを表す型
    // スタックはLinkedListやVecで実装するのが一般的だが、
    // lin型の変数キャプチャに関する問題に対処するため、BTreeMapを利用する
    vars: BTreeMap<usize, VarToType>,
}

impl TypeEnvStack {
    fn new() -> TypeEnvStack {
        TypeEnvStack {
            vars: BTreeMap::new(),
        }
    }

    /// 型環境をpush
    fn push(&mut self, depth: usize) {
        self.vars.insert(depth, BTreeMap::new());
    }

    /// 型環境をpop
    fn pop(&mut self, depth: usize) -> Option<VarToType> {
        self.vars.remove(&depth)
    }

    /// スタックの最も上にあるマップに変数と型の対応付を挿入
    fn insert(&mut self, key: String, value: parser::TypeExpr) {
        // BTreeMapやVec等のイテレータは両終端オペレータなので、前方と後方の両方から辿れる
        // スタックの最も上に位置する型環境に対して追加するので、next_backを使って後ろから辿っている
        if let Some(last) = self.vars.iter_mut().next_back() {
            last.1.insert(key, Some(value));
        }
    }

    /// スタックのトップからボトムに向かて順にマップをたどっていき、最初に発見したデータを取得する
    fn get_mut(&mut self, key: &str) -> Option<(usize, &mut Option<parser::TypeExpr>)> {
        for (depth, elm) in self.vars.iter_mut().rev() {
            if let Some(e) = elm.get_mut(key) {
                return Some((*depth, e));
            }
        }
        None
    }
}

/// 実際の型環境
/// lin用とun用で別々のTypeEnvStackを用意する
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeEnv {
    env_lin: TypeEnvStack, // lin用
    env_un: TypeEnvStack,  // un用
}

impl TypeEnv {
    pub fn new() -> TypeEnv {
        TypeEnv {
            env_lin: TypeEnvStack::new(),
            env_un: TypeEnvStack::new(),
        }
    }

    /// 型環境をpush
    /// 両方の型環境にpushする
    fn push(&mut self, depth: usize) {
        self.env_lin.push(depth);
        self.env_un.push(depth);
    }

    /// 型環境をpop
    /// 両方の型環境をpopする
    fn pop(&mut self, depth: usize) -> (Option<VarToType>, Option<VarToType>) {
        let t1 = self.env_lin.pop(depth);
        let t2 = self.env_un.pop(depth);
        (t1, t2)
    }

    /// 型環境へ変数と型を追加
    /// スタックの最も上にあるマップに対して追加するが、
    /// linかunかを判別して適切な型環境に追加する
    fn insert(&mut self, key: String, value: parser::TypeExpr) {
        if value.qual == parser::Qual::Lin {
            self.env_lin.insert(key, value);
        } else {
            self.env_un.insert(key, value);
        }
    }

    /// linとunの型環境からget_mutを呼び出し、depthが大きい方を返す
    fn get_mut(&mut self, key: &str) -> Option<&mut Option<parser::TypeExpr>> {
        match (self.env_lin.get_mut(key), self.env_un.get_mut(key)) {
            (Some((d1, t1)), Some((d2, t2))) => match d1.cmp(&d2) {
                Ordering::Less => Some(t2),
                Ordering::Greater => Some(t1),
                Ordering::Equal => panic!("invalid type environment"), // スタックの同じ高さに同じ変数名を複数指定できない
            },
            (Some((_, t1)), None) => Some(t1),
            (None, Some((_, t2))) => Some(t2),
            _ => None,
        }
    }
}

// 以下型検査器の実装

/// 型検査器で実装する関数の返り値の型
/// エラー時にはStringか&strを返すため、Cow(Copy on Write)を利用している
/// Cowは中身がStringなら書込み可能なのでそのまま利用し、中身が&strなら、一旦Stringに変換してから書き込みをする
type TResult<'a> = Result<parser::TypeExpr, Cow<'a, str>>;

/// 型付け関数
/// 式と型環境を受け取り、型を返す
///
/// ## Arguments
/// * `expr`  - 式
/// * `env`   - 型環境
/// * `depth` - 変数スコープのネストの深さ
pub fn typing<'a>(expr: &parser::Expr, env: &mut TypeEnv, depth: usize) -> TResult<'a> {
    match expr {
        parser::Expr::App(e) => typing_app(e, env, depth),
        parser::Expr::QVal(e) => typing_qval(e, env, depth),
        parser::Expr::Free(e) => typing_free(e, env, depth),
        parser::Expr::If(e) => typing_if(e, env, depth),
        parser::Expr::Split(e) => typing_split(e, env, depth),
        parser::Expr::Var(e) => typing_var(e, env),
        parser::Expr::Let(e) => typing_let(e, env, depth),
    }
}
fn typing_app<'a>(expr: &parser::AppExpr, env: &mut TypeEnv, depth: usize) -> TResult<'a> {
    let func_t = typing(&expr.expr1, env, depth)?;
    let param_t = typing(&expr.expr2, env, depth)?;

    match func_t.prim {
        PrimType::Arrow(e1, e2) => {
            if *e1 != param_t {
                return Err("関数の引数型と与えられた変数の型が異なる".into());
            }
            Ok(*e2)
        }
        _ => Err("appで関数型以外を使用している".into()),
    }
}

fn typing_free<'a>(expr: &parser::FreeExpr, env: &mut TypeEnv, depth: usize) -> TResult<'a> {
    let t = env.get_mut(&expr.var);
    if let Some(it) = t {
        if it.is_some() {
            *it = None;
        } else {
            return Err("すでに消費済みのリソースを解放している".into());
        }
    } else {
        return Err("存在しない変数".into());
    }

    let t = typing(&expr.expr, env, depth)?;

    Ok(parser::TypeExpr {
        qual: t.qual,
        prim: t.prim,
    })
}

fn typing_split<'a>(expr: &parser::SplitExpr, env: &mut TypeEnv, depth: usize) -> TResult<'a> {
    if &expr.left == &expr.right {
        return Err("同じ変数名は使用できません。".into());
    }

    let param_type = typing(&expr.expr, env, depth)?;
    let (q, p) = match param_type.prim.clone() {
        PrimType::Pair(t1, t2) => {
            let mut depth = depth;
            safe_add(&mut depth, &1, || "変数スコープのネストが深すぎる")?;
            env.push(depth);
            env.insert(expr.left.clone(), *t1);
            env.insert(expr.right.clone(), *t2);

            // 関数中の式を型付け
            let t = typing(&expr.body, env, depth)?;

            let (elin, _) = env.pop(depth);
            for (k, v) in elin.unwrap().iter() {
                if v.is_some() {
                    return Err(format!("関数定義内でlin型の変数\"{k}\"を消費していない").into());
                }
            }

            (
                t.qual,
                parser::PrimType::Arrow(Box::new(param_type), Box::new(t)),
            )
        }
        _ => {
            return Err("splitでペア型以外を使用している".into());
        }
    };

    Ok(parser::TypeExpr { qual: q, prim: p })
}

fn typing_let<'a>(expr: &parser::LetExpr, env: &mut TypeEnv, depth: usize) -> TResult<'a> {
    let t1 = typing(&expr.expr1, env, depth)?;
    if expr.ty != t1 {
        return Err("変数の型が一致しない".into());
    }

    let mut depth = depth;
    safe_add(&mut depth, &1, || "変数スコープのネストが深すぎる")?;
    env.push(depth);
    env.insert(expr.var.clone(), expr.ty.clone());

    let t2 = typing(&expr.expr2, env, depth)?;

    let (elin, _) = env.pop(depth);
    for (k, v) in elin.unwrap().iter() {
        if v.is_some() {
            return Err(format!("関数定義内でlin型の変数\"{k}\"を消費していない").into());
        }
    }

    Ok(parser::TypeExpr {
        qual: t2.qual,
        prim: t2.prim,
    })
}

/// 修飾子付きの型付け
fn typing_qval<'a>(expr: &parser::QValExpr, env: &mut TypeEnv, depth: usize) -> TResult<'a> {
    // プリミティブ型を計算
    let p = match &expr.val {
        parser::ValExpr::Bool(_) => parser::PrimType::Bool,
        parser::ValExpr::Pair(e1, e2) => {
            // 式e1とe2をtypingにより型付け
            let t1 = typing(e1, env, depth)?;
            let t2 = typing(e2, env, depth)?;

            // expr.qualがunであり、
            // e1かe2の型にlinが含まれていた場合、型付けエラー
            if expr.qual == parser::Qual::Un
                && (t1.qual == parser::Qual::Lin || t2.qual == parser::Qual::Lin)
            {
                return Err("un型のペア内でlin型を使用している".into());
            }

            // ペア型を返す
            parser::PrimType::Pair(Box::new(t1), Box::new(t2))
        }
        parser::ValExpr::Fun(e) => {
            // un型の関数の場合、この関数の外側で定義されたlin型の変数は利用できない
            // そのため、ここでlin用の型環境を空にする。
            // ただし、後で復元する必要があるため保存する。
            // これが、linとunで別々の型環境を用意し、BTreeMapでスタックを実装した理由
            let env_prev = if expr.qual == parser::Qual::Un {
                Some(mem::take(&mut env.env_lin))
            } else {
                None
            };

            // 型環境のスタックをインクリメントする
            // スタックのプッシュにはdepthが必要なため、インクリメントを忘れずに行う
            let mut depth = depth;
            safe_add(&mut depth, &1, || "変数スコープのネストが深すぎる")?;
            env.push(depth);
            env.insert(e.var.clone(), e.ty.clone()); // 変数の型を挿入

            // 関数中の式を型付け
            let t = typing(&e.expr, env, depth)?;

            // 型環境をポップし、ポップした型環境の中にlin型の変数(つまり引数)が残っていた場合は、
            // 消費されなかったということなのでエラー
            // このように型環境をスタックとして表すことで、変数のスコープを表現できる
            // また、スタックの上から順にたどるようにget_mutを実装しているため、シャドーイングも表現できる
            let (elin, _) = env.pop(depth);
            for (k, v) in elin.unwrap().iter() {
                if v.is_some() {
                    return Err(format!("関数定義内でlin型の変数\"{k}\"を消費していない").into());
                }
            }

            // lin用の型環境を復元
            if let Some(ep) = env_prev {
                env.env_lin = ep;
            }

            // 関数の型を生成
            parser::PrimType::Arrow(Box::new(e.ty.clone()), Box::new(t))
        }
    };

    // 修飾子付き型を返す
    Ok(parser::TypeExpr {
        qual: expr.qual,
        prim: p,
    })
}

/// 変数の型付け
/// lin型の変数が参照された場合は、消費して型環境から削除する
fn typing_var<'a>(expr: &str, env: &mut TypeEnv) -> TResult<'a> {
    let ret = env.get_mut(expr);
    if let Some(it) = ret {
        // 定義されている
        if let Some(t) = it {
            // 消費されていない
            if t.qual == parser::Qual::Lin {
                // lin型
                let eret = t.clone();
                *it = None; // lin型の変数を消費
                return Ok(eret);
            } else {
                return Ok(t.clone());
            }
        }
    }
    Err(format!("\"{expr}\"という変数は定義されていないか、利用済みか、キャプチャできない").into())
}

/// if式の型付け
fn typing_if<'a>(expr: &parser::IfExpr, env: &mut TypeEnv, depth: usize) -> TResult<'a> {
    // 条件の式の型つけを行い、その型がboolであるかを検査
    let t1 = typing(&expr.cond_expr, env, depth)?;
    if t1.prim != parser::PrimType::Bool {
        return Err("ifの条件式がboolでない".into());
    }

    // thenとelseで別々の式を同じ型環境で検査するため、型環境をcloneしてから、それぞれの式の型付けを行う
    let mut e = env.clone();
    let t2 = typing(&expr.then_expr, &mut e, depth)?;
    let t3 = typing(&expr.else_expr, &mut e, depth)?;

    // thenとelse部の型は同じで、
    // thenとelse部の評価後の型環境は同じかチェック
    if t2 != t3 || e != *env {
        return Err("ifのthenとelseの式の型が異なる".into());
    }
    Ok(t2)
}
