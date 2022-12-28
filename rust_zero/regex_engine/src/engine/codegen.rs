use super::{parser::AST, Instruction};
use crate::helper::{safe_add, DynError};
use std::{
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug)]
pub enum CodeGenError {
    PCOverFlow,
    FailStar,
    FailOr,
    FailQuestion,
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for CodeGenError {}

/// コード生成器
#[derive(Default, Debug)]
struct Generator {
    pc: usize,               // 次に生成するアセンブリ命令のアドレス
    insts: Vec<Instruction>, // 生成された命令列
}

impl Generator {
    /// コード生成を行う関数の入り口
    fn gen_code(&mut self, ast: &AST) -> Result<(), DynError> {
        self.gen_expr(ast)?;
        self.inc_pc()?;
        self.insts.push(Instruction::Match);
        Ok(())
    }

    /// プログラムカウンタをインクリメント
    fn inc_pc(&mut self) -> Result<(), CodeGenError> {
        safe_add(&mut self.pc, &1, || CodeGenError::PCOverFlow)
    }

    /// ASTをパターン分けし、コード生成を行う関数
    fn gen_expr(&mut self, ast: &AST) -> Result<(), DynError> {
        match ast {
            AST::Char(c) => self.gen_char(*c)?,
            AST::Or(e1, e2) => self.gen_or(e1, e2)?,
            AST::Plus(e) => self.gen_plus(e)?,
            AST::Star(e) => self.gen_star(e)?,
            AST::Question(e) => self.gen_question(e)?,
            AST::Seq(v) => self.gen_seq(v)?,
        }
        Ok(())
    }

    /// 連続するASTのコード生成
    fn gen_seq(&mut self, exprs: &[AST]) -> Result<(), DynError> {
        for e in exprs {
            self.gen_expr(e)?;
        }
        Ok(())
    }

    /// char命令生成関数
    fn gen_char(&mut self, c: char) -> Result<(), DynError> {
        let inst = Instruction::Char(c);
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    /// OR演算子のコードを生成する
    ///
    /// 以下のようなコードを生成
    ///
    /// ```text
    ///     split L1, L2
    /// L1: e1のコード
    ///     jmp L3
    /// L2: e2のコード
    /// L3:
    /// ```
    fn gen_or(&mut self, e1: &AST, e2: &AST) -> Result<(), DynError> {
        // [split L1, L2] のコード生成
        let split_addr = self.pc;
        self.inc_pc()?;
        // L1 = self.pc, L2は仮に0と設定
        let split = Instruction::Split(self.pc, 0); // L2はe1とe2のコードを生成後でないと決定できないため、仮に0と設定している
        self.insts.push(split);

        // [L1: e1]のコード生成
        self.gen_expr(e1)?;

        // [jmp L3]のコード生成
        let jmp_addr = self.pc;
        self.insts.push(Instruction::Jump(0)); // L3はe2のコード生成後でないと決定できないため仮に0と設定
        self.inc_pc()?;

        // Split命令のL2の値を設定
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
        } else {
            return Err(Box::new(CodeGenError::FailOr));
        }

        // [L2: e2]のコードを生成
        self.gen_expr(e2)?;

        // Jum命令のL3の値を設定
        if let Some(Instruction::Jump(l3)) = self.insts.get_mut(jmp_addr) {
            *l3 = self.pc;
        } else {
            return Err(Box::new(CodeGenError::FailOr));
        }

        Ok(())
    }

    /// Plus演算子のコードを生成する
    ///
    /// 以下のようなコードを生成
    ///
    /// ```text
    /// L1: e1のコード
    ///     split L1, L2
    /// L2:
    /// ```
    fn gen_plus(&mut self, e1: &AST) -> Result<(), DynError> {
        let split_addr = self.pc;

        // [L1: e1]のコード生成
        self.gen_expr(e1)?;
        self.inc_pc()?;

        // [split L1, L2] のコード生成
        let split = Instruction::Split(split_addr, self.pc);
        self.insts.push(split);

        Ok(())
    }

    /// Star演算子のコードを生成する
    ///
    /// 以下のようなコードを生成
    ///
    /// ```text
    /// L1: split L2, L3
    /// L2: e1のコード
    ///     jump L1
    /// L3:
    /// ```
    fn gen_star(&mut self, e1: &AST) -> Result<(), DynError> {
        // [split L2, L3] のコード生成
        let split_addr = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0);
        self.insts.push(split);

        // [L2: e1]のコード生成
        self.gen_expr(e1)?;

        // [jmp L1]のコード生成
        self.insts.push(Instruction::Jump(split_addr));
        self.inc_pc()?;

        // Split命令のL3の値を設定
        if let Some(Instruction::Split(_, l3)) = self.insts.get_mut(split_addr) {
            *l3 = self.pc;
        } else {
            return Err(Box::new(CodeGenError::FailOr));
        }

        Ok(())
    }

    /// Question演算子のコードを生成する
    ///
    /// 以下のようなコードを生成
    ///
    /// ```text
    ///     split L1, L2
    /// L1: e1のコード
    /// L2:
    /// ```
    fn gen_question(&mut self, e1: &AST) -> Result<(), DynError> {
        // [split L1, L2] のコード生成
        let split_addr = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0);
        self.insts.push(split);

        // [L1: e1]のコード生成
        self.gen_expr(e1)?;

        // Split命令のL2の値を設定
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
        } else {
            return Err(Box::new(CodeGenError::FailOr));
        }

        Ok(())
    }
}

pub fn get_code(ast: &AST) -> Result<Vec<Instruction>, DynError> {
    let mut generator = Generator::default();
    generator.gen_code(ast)?;
    Ok(generator.insts)
}
