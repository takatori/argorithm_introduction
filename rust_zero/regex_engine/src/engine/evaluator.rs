use super::Instruction;
use crate::helper::{safe_add, DynError};
use std::{
    // collections::VecDeque, 幅優先探索の際に使用する
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug)]
pub enum EvalError {
    PCOverFlow,
    SPOverFlow,
    InvalidPC,
    InvalidContext,
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EvaluatorError: {:?}", self)
    }
}

impl Error for EvalError {}

/// 深さ優先探索で再帰的にマッチングを行う関数
fn eval_depth(
    inst: &[Instruction],
    line: &[char],
    mut pc: usize,
    mut sp: usize,
) -> Result<bool, DynError> {
    loop {
        let next = if let Some(i) = inst.get(pc) {
            i
        } else {
            return Err(Box::new(EvalError::InvalidPC));
        };

        match next {
            Instruction::Char(c) => {
                if let Some(sp_c) = line.get(sp) {
                    if c == sp_c {
                        safe_add(&mut pc, &1, || Box::new(EvalError::PCOverFlow))?;
                        safe_add(&mut sp, &1, || Box::new(EvalError::SPOverFlow))?;
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Instruction::Match => {
                return Ok(true);
            }
            Instruction::Jump(addr) => {
                pc = *addr;
            }
            Instruction::Split(addr1, addr2) => {
                if eval_depth(inst, line, *addr1, sp)? || eval_depth(inst, line, *addr2, sp)? {
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
        }
    }
}

/// 命令列の評価を行う関数
///
/// instが命令列となり、その命令列を用いて入力文字列lineにマッチさせる
/// is_depthがtrueの場合に深さ優先探索を、falseの場合に幅優先探索を行う
///
/// 実行時エラーが起きた場合はErrを返す
/// マッチ成功時はOk(true)を、失敗時はOk(false)を返す
pub fn eval(inst: &[Instruction], line: &[char], is_depth: bool) -> Result<bool, DynError> {
    eval_depth(inst, line, 0, 0)
}
