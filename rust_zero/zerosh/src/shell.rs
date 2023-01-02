use crate::helper::DynError;
use nix::{
    libc,
    sys::{
        signal::{killpg, signal, SigHandler, Signal},
        wait::{waitpid, WaitPidFlag, WaitStatus},
    },
    unistd::{self, dup2, execvp, fork, pipe, setpgid, tcgetpgrp, tcsetpgrp, ForkResult, Pid},
};
use rustyline::{error::ReadlineError, Editor};
use signal_hook::{consts::*, iterator::Signals};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ffi::CString,
    mem::replace,
    path::PathBuf,
    process::exit,
    sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender},
    thread,
};

/// システムコール呼び出しのラッパ。EINTRならリトライ
///
/// EINTRはシステムコール中に割り込みが発生したことを示しており、
/// 再度システムコールを呼び出す必要があるが、それを自動で行う
///
/// 引数fにシステムコールを呼び出す関数を受け取り、その結果がEINTRなら再度実行する。
/// システムコールがどのようなエラーを返すかは、manのERRORSエントリからわかり、
/// エラーに応じて適切な処理を行う必要がある。
fn syscall<F, T>(f: F) -> Result<T, nix::Error>
where
    F: Fn() -> Result<T, nix::Error>,
{
    loop {
        match f() {
            Err(nix::Error::EINTR) => (), // リトライ
            result => return result,
        }
    }
}

/// workerスレッドが受信するメッセージ
enum WorkerMsg {
    Signal(i32), // シグナルを受信
    Cmd(String), // コマンド入力
}

/// mainスレッドが受信するメッセージ
enum ShellMsg {
    Continue(i32), // シェルの読み込みを再開。i32は最後の終了コード
    Quit(i32),     // シェルを終了。i32はシェルの終了コード
}

#[derive(Debug)]
pub struct Shell {
    logfile: String, // ログファイル
}

impl Shell {
    pub fn new(logfile: &str) -> Self {
        Shell {
            logfile: logfile.to_string(),
        }
    }

    /// mainスレッド
    pub fn run(&self) -> Result<(), DynError> {
        // SIGTTOUを無視に設定しないと、SIGTSTPが配送される
        // デフォルトの挙動だと、標準出力への書き込み時にSIGTSTPが配送されて、シェルが停止してしまう
        // そこで、SIGTTOUシグナルを無視するために、SigIgnと設定する
        unsafe { signal(Signal::SIGTTOU, SigHandler::SigIgn).unwrap() };

        // rustylineのEditorを利用すると、標準入力からの読み込みが容易に行え、
        // 矢印キーを使った操作などをサポートできる。
        let mut rl = Editor::<()>::new()?;
        if let Err(e) = rl.load_history(&self.logfile) {
            eprintln!("Zerosh: ヒストリファイルの読み込みに失敗: {e}");
        };

        // チャネルを生成し、signal_handlerとworkerスレッドを生成
        let (worker_tx, worker_rx) = channel();
        let (shell_tx, shell_rx) = sync_channel(0);
        spawn_sig_handler(worker_tx.clone())?;
        Worker::new().spawn(worker_rx, shell_tx);

        let exit_val; // 終了コード
        let mut prev = 0; // 直前の終了コード

        loop {
            // 1行読み込んで、その行をworkerスレッドに送信
            let face = if prev == 0 { '\u{1F642}' } else { '\u{1F480}' };
            match rl.readline(&format!("ZeroSh {face} &> ")) {
                Ok(line) => {
                    let line_trimed = line.trim();
                    if line_trimed.is_empty() {
                        continue; // 空のコマンドの場合は再読み込み
                    } else {
                        rl.add_history_entry(line_trimed); // ヒストリファイルに追加
                    }

                    // workerスレッドに送信
                    worker_tx.send(WorkerMsg::Cmd(line)).unwrap();

                    //workerスレッドの処理が完了するまで待機
                    match shell_rx.recv().unwrap() {
                        ShellMsg::Continue(n) => prev = n, // 読み込み再開
                        ShellMsg::Quit(n) => {
                            // シェルを終了
                            exit_val = n;
                            break;
                        }
                    }
                }
                // コマンド読み込み時に割り込みが発生した場合は、再実行する
                // これは、主にCtrl+cが入力された場合に発生し、
                // 誤ってシェルを終了させてしまうことを防ぐために、このようにしている
                Err(ReadlineError::Interrupted) => eprintln!("ZeroSh: 終了はCtrl+d"),
                // Ctrl+dを入力すると、End of File(EOF)と呼ばれる入力終了を意味する特殊な文字を入力できる
                // EOFが入力されるとexitコマンドをworkerスレッドに送信し、workerスレッドからの返答を受信後終了する
                // exitコマンド実行後は必ず、Quitを受信するはずなので、それ以外を受信した場合にはパニックさせてプログラムを終了させる
                Err(ReadlineError::Eof) => {
                    worker_tx.send(WorkerMsg::Cmd("exit".to_string())).unwrap();
                    match shell_rx.recv().unwrap() {
                        ShellMsg::Quit(n) => {
                            // シェルを終了
                            exit_val = n;
                            break;
                        }
                        _ => {
                            panic!("exitに失敗");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("ZeroSh: 読み込みエラー\n{e}");
                    exit_val = 1;
                    break;
                }
            }
        }

        if let Err(e) = rl.save_history(&self.logfile) {
            eprintln!("ZeroSh: ヒストリファイルへの書き込みに失敗: {e}");
        }
        exit(exit_val);
    }
}

fn spawn_sig_handler(tx: Sender<WorkerMsg>) -> Result<(), DynError> {
    // SIGCHLD: 子プロセスの状態変化時に通知される    
    let mut signals = Signals::new(&[SIGINT, SIGTSTP, SIGCHLD])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            // シグナルを受信しworkerスレッドに転送
            tx.send(WorkerMsg::Signal(sig)).unwrap();            
        }
    });
    Ok(())
}