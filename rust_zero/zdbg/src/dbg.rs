use crate::helper::DynError;
use nix::s
use nix::{
    libc::user_regs_struct,
    sys::{
        personality::{self, Persona},
        ptrace,
        wait::{waitpid, WaitStatus},
    },
    unistd::{execvp, fork, ForkResult, Pid},
};
use std::ffi::{c_void, CString};

/// デバッガ内の情報
pub struct DbgInfo {
    pid: Pid,
    brk_addr: Option<*mut c_void>, // ブレークポイントのアドレス
    brk_val: i64,                  // ブレークポイントを設定したメモリの元の値
    filename: String,              // 実行ファイル
}

/// デバッガ
/// ZDbg<Running>は子プロセスを実行中
/// ZDbg<NotRunning>は子プロセスを実行していない
pub struct ZDbg<T> {
    info: Box<DbgInfo>,
    _state: T,
}

// デバッガの状態
pub struct Running; // 実行中
pub struct NotRunning; // 実行していない

/// デバッガの状態の列挙型表現。Exitの場合、終了
pub enum State {
    Running(ZDbg<Running>),
    NotRunning(ZDbg<NotRunning>),
    Exit,
}

/// RunningとNotRunningで共通の実装
impl<T> ZDbg<T> {
    /// ブレークポイントのアドレスを設定する関数。子プロセスのメモリ上には反映しない。
    /// アドレス設定に成功した場合はtrueを返す。
    fn set_break_addr(&mut self, cmd: &[&str]) -> bool {
        if self.info.brk_addr.is_some() {
            eprintln!(
                "<<ブレークポイントは設定済みです: Addr = {:p}>>",
                self.info.brk_addr.unwrap()
            );
            false
        } else if let Some(addr) = get_break_addr(cmd) {
            self.info.brk_addr = Some(addr); // ブレークポイントのアドレスを保存
            true
        } else {
            false
        }
    }

    /// 共通のコマンドを実行
    fn do_cmd_common(&self, cmd: &[&str]) {
        match cmd[0] {
            "help" | "h" => do_help(),
            _ => (),
        }
    }
}

/// NotRunning時に呼び出し可能なメソッド
impl ZDbg<NotRunning> {
    pub fn new(filename: String) -> Self {
        ZDbg {
            info: Box::new(DbgInfo {
                pid: Pid::from_raw(0),
                brk_addr: None,
                brk_val: 0,
                filename,
            }),
            _state: NotRunning,
        }
    }

    pub fn do_cmd(mut self, cmd: &[&str]) -> Result<State, DynError> {
        if cmd.is_empty() {
            return Ok(State::NotRunning(self));
        }

        match cmd[0] {
            "run" | "r" => return self.do_run(cmd),
            "break" | "b" => {
                self.do_break(cmd);
            }
            "exit" => return Ok(State::Exit),
            "continue" | "c" | "stepi" | "s" | "registers" | "regs" => {
                eprintln!("<<ターゲットを実行していません。runで実行してください>>")
            }
            _ => self.do_cmd_common(cmd),
        }

        Ok(State::NotRunning(self))
    }

    /// ブレークポイントを設定
    fn do_break(&mut self, cmd: &[&str]) -> bool {
        self.set_break_addr(cmd)
    }

    /// 子プロセスを生成し、成功した場合はRunning状態に遷移
    fn do_run(mut self, cmd: &[&str]) -> Result<State, DynError> {
        // 子プロセスに渡すコマンドライン引数
        // execvpへはCStringの文字列を渡す必要があるため、ここで変換している
        let args: Vec<CString> = cmd.iter().map(|s| CString::new(*s).unwrap()).collect();

        match unsafe { fork()? } {
            ForkResult::Child => {
                // ASLR(address space layout randomization)を無効に
                //
                // ASLRは、実行時の仮想メモリのアドレスをランダムに配置する技術である。
                // ASLRが適用されると、実行時のアドレスがランダムになりセキュリティは向上するが、
                // デバッグ時には不便なため、ここでオフにする。
                // Linuxではセキュリティ上の理由から、可能な場合はASLRを適用している
                // ASLRは、Return-to-libc攻撃といった攻撃手法による被害を軽減させる目的で導入された。
                let p = personality::get().unwrap();
                personality::set(p | Persona::ADDR_NO_RANDOMIZE).unwrap();
                // 自身がデバッガによるトレース対象であることを指定する
                // tracemeを指定したあとは、execすると即座にプロセスが停止するようになる
                // nix::sys::ptraceにはシステムコールのptrace関数のラッパが多く定義されている
                ptrace::traceme().unwrap(); 
                // execvpで子プロセスをデバッグ対象のプログラムに置き換え
                execvp(&CString::new(self.info.filename.as_str()).unwrap(), &args).unwrap();
                unreachable!();
            }
            // 親プロセスは、waitpidで子プロセスが停止するのを待つ。
            // 子プロセスでtracemeを呼び出しているため、子プロセスは停止、もしくは終了するはずである。
            ForkResult::Parent { child, .. } => match waitpid(child, None)? {
                WaitStatus::Stopped(..) => {
                    println!("<<子プロセスの実行に成功しました : PID = {child}>>");
                    self.info.pid = child;
                    // ZDbg<Running>の値を生成して状態遷移を実現
                    let mut dbg = ZDbg::<Running> {
                        info: self.info,
                        _state: Running,
                    };
                    // ブレークポイントを子プロセスのメモリ上に実際に設定
                    // ブレークポイントはプロセスの実行中にしか行えないため、
                    // この時点でブレークポイントを設定している
                    dbg.set_break()?: 
                    // 子プロセスの実行を再開
                    dbg.do_continue()
                }
                WaitStatus::Exited(..) | WaitStatus::Signaled(..) => {
                    Err("子プロセスの実行に失敗しました".into())
                }
                _ => Err("子プロセスが不正な状態です".into()),
            },
        }
    }
}

/// Running時に呼び出し可能なメソッド
impl ZDbg<Running> {

    pub fn do_cmd(mut self, cmd: &[&str]) -> Result<State, DynError> {
        if cmd.is_empty() {
            return Ok(State::Running(self));
        }

        match cmd[0] {
            "break" | "b" => self.do_break(cmd)?,
            "continue" | "c" => return self.do_continue(),
            "registers" | "regs" => {
                let regs = ptrace::getregs(self.info.pid)?;
                print_regs(&regs);
            },
            "stepi" | "s" => return self.do_stepi(),
            "run" | "r" => eprintln!("<<すでに実行中です>>"),
            "exit" => {
                self.do_exit()?;
                return Ok(State::Exit);
            }
            _ => self.do_cmd_common(cmd),
        }

        Ok(State::Running(self))
    }

    fn do_stepi(self) -> Result<State, DynError> {}
}

/// ヘルプを表示
fn do_help() {
    println!(
        r#"コマンド一覧(括弧内は省略記法)
        break 0x8000 : ブレークポイントを0x8000番地に設定 (b 0x8000)
        run          : プログラムを実行 (r)
        continue     : プログラムを再開 (c)
        stepi        : 機械語レベルで1ステップ実行 (s)
        registers    : レジスタを表示 (regs)
        exit         : 終了
        help         : このヘルプを表示 (h) "#
    );
}
