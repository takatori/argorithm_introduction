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
}
