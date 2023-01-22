use crate::helper::DynError;
use nix::{
    libc::{ptrace, user_regs_struct},
    sys::{
        personality::{self, Persona},
        ptrace,
        wait::{waitpid, WaitStatus},
    },
    unistd::{execvp, fork, ForkResult, Pid},
};
use std::{
    ffi::{c_void, CString},
    ops::Not,
    rc::Rc,
};

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
                println!("personality{:?}", p);
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
                    dbg.set_break()?;
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
                // レジスタ情報の取得
                // Cのptrace(PTRACE_GETREGS, pid, 0, &struct)に相当
                // &structはレジスタ情報おw保存する構造体へのポインタであり、結果がこれに格納される
                let regs = ptrace::getregs(self.info.pid)?;
                print_regs(&regs); // 取得した情報を表示する
            }
            "stepi" | "s" => return self.do_stepi(),
            "run" | "r" => eprintln!("<<すでに実行中です>>"),
            "exit" => {
                self.do_exit()?; // 子プロセスを終了させる
                return Ok(State::Exit);
            }
            _ => self.do_cmd_common(cmd),
        }

        Ok(State::Running(self))
    }

    /// exitを実行。実行中のプロセスはkill
    fn do_exit(self) -> Result<(), DynError> {
        loop {
            // SIGKILLシグナルを子プロセスに送信する
            ptrace::kill(self.info.pid)?;
            match waitpid(self.info.pid, None)? {
                WaitStatus::Exited(..) | WaitStatus::Signaled(..) => return Ok(()),
                _ => (),
            }
        }
    }

    /// breakを実行
    fn do_break(&mut self, cmd: &[&str]) -> Result<(), DynError> {
        if self.set_break_addr(cmd) {
            self.set_break()?;
        }
        Ok(())
    }

    /// ブレークポイントを実際に設定
    /// つまり、該当アドレスのメモリを"int 3" = 0xccに設定
    fn set_break(&mut self) -> Result<(), DynError> {
        let addr = if let Some(addr) = self.info.brk_addr {
            addr
        } else {
            return Ok(());
        };

        // ブレークするアドレスにあるメモリ上の値を取得
        // メモリの値はi64型で返される。つまり、8バイト単位で取得できる。
        let val = match ptrace::read(self.info.pid, addr) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("<<ptrace::readに失敗 : {e}, addr = {:p}>>", addr);
                return Ok(());
            }
        };

        // メモリ上の値を表示する補助関数
        fn print_val(addr: usize, val: i64) {
            print!("{:x}:", addr);
            for n in (0..8).map(|n| ((val >> (n * 8)) & 0xff) as u8) {
                print!(" {:x}", n);
            }
        }

        println!("<<以下のようにメモリを書き換えます>>");
        print!("<<before : "); // もとの値を表示
        print_val(addr as usize, val);
        println!(">>");

        // "int 3"に設定
        // "int 3"はソフトウェア割り込みを発生させるx86_64の命令
        // この命令はブレークポイントに用いられ、int 3を発行したプロセスへはOSからSIGTRAPシグナルが送信される
        // プログラム中にint 3命令があると、この命令の実行後に割り込みハンドラが起動され、その後にSIGTRAPが発行されてプロセスが停止する
        // これがブレークポイントの正体。ブレークポイントを設定するためには、停止したいアドレスを特定してint 3に書き換えれば良い
        //
        // "int 3"命令のバイナリ表現は0xcc
        // valの下位8ビットを0xccに設定。(val & !0xff)とすると、valの下位8ビットが0クリアされ、
        // その後、0xccとビット和を取ると、下位8ビットが0xccとなる
        // x86_64はリトルエンディアンを用いるため、下位ビットを書き換えている
        let val_int3 = (val & !0xff) | 0xcc;
        print!("<<after : "); // 変更後の値を表示
        print_val(addr as usize, val_int3);
        println!(">>");

        // "int 3"をメモリに書き込み
        // as *mut c_voidと型変換しているのは、ptrace::write、つまり、Cのptraceが引数にポインタを取るためである
        match unsafe { ptrace::write(self.info.pid, addr, val_int3 as *mut c_void) } {
            Ok(_) => {
                self.info.brk_addr = Some(addr);
                self.info.brk_val = val; // 元の値を保持
            }
            Err(e) => {
                eprintln!("<<ptrace::writeに失敗 : {e}, addr = {:p}>>", addr);
            }
        }
        Ok(())
    }

    /// 停止中の子プロセスを再開させるcontinueを実行
    ///
    /// step_and_breakやwait_childメソッドを実行すると子プロセスが終了する可能性があるため
    /// このメソッドはselfで値を取得して、遷移後の状態を返すようにしている
    ///
    /// int 3でプロセスが停止した場合、プログラムカウンタはint 3のアドレス+1を指している
    /// プロセスを正常に再開させるためには、int 3に書き換えた箇所の復元と、プログラムカウンタを-1にする必要がある
    ///
    /// 1. continueコマンドが実行される
    /// 2. 機械語レベルで1ステップ実行　<step_and_brek関数> (前回のwait_childでbrk_addrのメモリの値はもとの値に復元済みなので普通に実行できる)
    /// 3. ブレークポイントを再設定 <step_and_brek関数> (brk_addrの値をint 3に再設定)
    /// 4. ptrace::contを呼び出し、子プロセスを再開 <step_and_brek関数>
    /// 5. waitpidで子プロセス停止を待ち、ブレークポイントで停止する <wait_child関数>
    /// 6. ブレークポイントを設定した番地のメモリの値を、元の値に復元 <wait_child関数> (次のstep_and_brek実行時に元の命令が実行されるようにする)
    /// 7. プログラムカウンタを-1する。 <wait_child関数> (プログラムカウンタがbrk_addrの+1を指しているので)
    /// 8. 1に戻る
    fn do_continue(self) -> Result<State, DynError> {
        // ブレークポイントで停止していた場合は1ステップ実行後再設定
        match self.step_and_break()? {
            State::Running(r) => {
                // 実行再開
                // ptrace::contで子プロセスを再開させる
                // ptrace::contの第２引数には、再開時に送信するシグナルを指定可能
                // Noneを指定した場合はシグナルは送信されない
                ptrace::cont(r.info.pid, None)?;
                r.wait_child()
            }
            n => Ok(n),
        }
    }

    /// ブレークポイントで停止していた場合は、
    /// 1ステップ実行しブレークポイントを再設定
    /// これは、ブレークポインが揮発してしまうのを防ぐための操作
    /// ブレークポイントを再設定しないと、ループなどで再び同じコードが時刻された場合に停止しなくなってしまう
    fn step_and_break(mut self) -> Result<State, DynError> {
        let regs = ptrace::getregs(self.info.pid)?; // レジスタ取得
        if Some((regs.rip) as *mut c_void) == self.info.brk_addr {
            // プログラムカウンタを意味するripがブレークポイントのアドレスかチェック
            ptrace::step(self.info.pid, None)?; // 機械語レベルで1ステップ実行
            match waitpid(self.info.pid, None)? {
                WaitStatus::Exited(..) | WaitStatus::Signaled(..) => {
                    println!("<<子プロセスが終了しました>>");
                    return Ok(State::NotRunning(ZDbg::<NotRunning> {
                        info: self.info,
                        _state: NotRunning,
                    }));
                }
                _ => (),
            }
            self.set_break()?; // 再度ブレークポイントを設定
        }
        Ok(State::Running(self))
    }

    /// 子プロセスをwait. 子プロセスが終了した場合はNotRunning状態に遷移
    fn wait_child(self) -> Result<State, DynError> {
        match waitpid(self.info.pid, None)? {
            WaitStatus::Exited(..) | WaitStatus::Signaled(..) => {
                println!("<<子プロセスが終了しました>>");
                let not_run = ZDbg::<NotRunning> {
                    info: self.info,
                    _state: NotRunning,
                };
                Ok(State::NotRunning(not_run))
            }
            WaitStatus::Stopped(..) => {
                // 子プロセスが停止した場合
                let mut regs = ptrace::getregs(self.info.pid)?;
                if Some((regs.rip - 1) as *mut c_void) == self.info.brk_addr {
                    // ブレークポイントで停止した場合
                    // 書き換えたメモリをもとの値に戻す
                    unsafe {
                        ptrace::write(
                            self.info.pid,
                            self.info.brk_addr.unwrap(),
                            self.info.brk_val as *mut c_void,
                        )?
                    };

                    // ブレークポイントで停止したアドレスから１つ戻す
                    regs.rip -= 1;
                    ptrace::setregs(self.info.pid, regs)?;
                }
                println!("<<子プロセスが停止しました : PC = {:#x}>>", regs.rip);
                Ok(State::Running(self))
            }
            _ => Err("waitpidの返り値が不正です".into()),
        }
    }

    /// stepiコマンドを実行する
    /// 機械語レベルで1ステップ実行を行うメソッド
    fn do_stepi(mut self) -> Result<State, DynError> {
        let regs = ptrace::getregs(self.info.pid)?;
        if Some((regs.rip) as *mut c_void) == self.info.brk_addr {
            // ブレークポイントで停止した場合は、そのメモリの値が0xccとなっている
            // 可能性があるため、もとの値に復元する
            unsafe {
                ptrace::write(
                    self.info.pid,
                    self.info.brk_addr.unwrap(),
                    self.info.brk_val as *mut c_void,
                )?
            };
            // regs.rip -= 1;
            // ptrace::setregs(self.info.pid, regs)?;
            ptrace::step(self.info.pid, None)?; // 機械語レベルで1ステップ実行
            match waitpid(self.info.pid, None)? {
                WaitStatus::Exited(..) | WaitStatus::Signaled(..) => {
                    println!("<<子プロセスが終了しました>>");
                    return Ok(State::NotRunning(ZDbg::<NotRunning> {
                        info: self.info,
                        _state: NotRunning,
                    }));
                }
                _ => (),
            }
            self.set_break()?;
        } else {
            ptrace::step(self.info.pid, None)?; // 機械語レベルで1ステップ実行
            match waitpid(self.info.pid, None)? {
                WaitStatus::Exited(..) | WaitStatus::Signaled(..) => {
                    println!("<<子プロセスが終了しました>>");
                    return Ok(State::NotRunning(ZDbg::<NotRunning> {
                        info: self.info,
                        _state: NotRunning,
                    }));
                }
                _ => (),
            }
        }
        Ok(State::Running(self))
    }
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

/// レジスタを表示
fn print_regs(regs: &user_regs_struct) {
    println!(
        r#"RIP: {:#016x}, RSP:{:#016x}, RBP: {:#016x}
RAX: {:#016x}, RBX:{:#016x}, RCX: {:#016x}
RDX: {:#016x}, RSI:{:#016x}, RDI: {:#016x}
 R8: {:#016x},  R9:{:#016x}, R10: {:#016x}
R11: {:#016x}, R12:{:#016x}, R13: {:#016x} 
R14: {:#016x}, R15:{:#016x}"#,
        regs.rip,
        regs.rsp,
        regs.rbp,
        regs.rax,
        regs.rbx,
        regs.rcx,
        regs.rdx,
        regs.rsi,
        regs.rdi,
        regs.r8,
        regs.r9,
        regs.r10,
        regs.r11,
        regs.r12,
        regs.r13,
        regs.r14,
        regs.r15,
    );
}

/// コマンドからブレークポイントを計算
fn get_break_addr(cmd: &[&str]) -> Option<*mut c_void> {
    if cmd.len() < 2 {
        eprintln!("<<アドレスを指定してください\n例: break 0x8000>>");
        return None;
    }

    let addr_str = cmd[1];
    if &addr_str[0..2] != "0x" {
        eprintln!("<<アドレスは16進数でのみ指定可能です\n例: break 0x8000>>");
        return None;
    }

    let addr = match usize::from_str_radix(&addr_str[2..], 16) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("<<アドレス変換エラー : {e}>>");
            return None;
        }
    } as *mut c_void;

    Some(addr)
}
