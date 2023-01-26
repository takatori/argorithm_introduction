use crate::{helper::safe_add, parser};
use std::{borrow::Cow, cmp::Ordering, collections::BTreeMap, mem};

type VarToType = BTreeMap<String, Option<parser::TypeExpr>>;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct TypeEnvStack {
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

    /// スタックの最も上にある肩環境に変数と型を追加
    fn insert(&mut self, key: String, value: parser::TypeExpr) {
        if let Some(last) = self.vars.iter_mut().next_back() {
            last.1.insert(key, Some(value));
        }
    }
}
