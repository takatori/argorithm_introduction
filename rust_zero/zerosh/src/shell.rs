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

#[derive(Debug, PartialEq, Eq, Clone)]
enum ProcState {
    Run,  // 実行中
    Stop, // 停止中
}

#[derive(Debug, Clone)]
struct ProcInfo {
    state: ProcState, // 実行状態
    pgid: Pid,        // プロセスグループID
}

#[derive(Debug)]
struct Worker {
    exit_val: i32,                                     // 終了コード
    fg: Option<Pid>,                                   // フォアグラウンドのプロセスグループID
    jobs: BTreeMap<usize, (Pid, String)>, // ジョブIDから(プロセスグループID, 実行コマンド)へのマップ
    pgid_to_pids: HashMap<Pid, (usize, HashSet<Pid>)>, // プロセスグループIDから(ジョブID, 実行コマンド)へのマップ
    pid_to_info: HashMap<Pid, ProcInfo>,               // プロセスIDからプロセスグループIDへのマップ
    shell_pgid: Pid,                                   // シェルのプロセスグループID
}

impl Worker {
    fn new() -> Self {
        Worker {
            exit_val: 0,
            fg: None, // フォアグラウンドはシェル
            jobs: BTreeMap::new(),
            pgid_to_pids: HashMap::new(),
            pid_to_info: HashMap::new(),
            // シェルのプロセスグループIDを取得
            // tcgetpgrpという、同名のCライブラリ関数が存在し、
            // libc::STDIN_FILENOというファイルディスクリプタ
            // に関連付けられた、フォアグラウンドのプロセスグループIDを取得する。
            // ここでは、つまりシェルのプロセスグループIDを取得している
            // 自身のプロセスグループIDを取得するために、getpgidシステムコールも利用できるが、
            // tcgetpgrpを利用すると、シェルがフォアグラウンドであるかも検査できるため、こちらを利用している
            shell_pgid: tcgetpgrp(libc::STDIN_FILENO).unwrap(),
        }
    }

    /// workerスレッドを起動
    fn spawn(mut self, worker_rx: Receiver<WorkerMsg>, shell_tx: SyncSender<ShellMsg>) {
        thread::spawn(move || {
            for msg in worker_rx.iter() {
                match msg {
                    WorkerMsg::Cmd(line) => {
                        match parse_cmd(&line) {
                            Ok(cmd) => {
                                // 組み込みコマンドを実行
                                // 組み込みコマンドとは、シェル内部のコマンドのこと
                                if self.build_in_cmd(&cmd, &shell_tx) {
                                    // 組み込みコマンドならworker_rxから取得
                                    continue;
                                }

                                // 組み込みコマンドでない場合は、外部プログラムを実行
                                if !self.spawn_child(&line, &cmd) {
                                    // 子プロセス生成に失敗した場合、シェルからの入力を再開
                                    shell_tx.send(ShellMsg::Continue(self.exit_val)).unwrap();
                                }
                            }
                            Err(e) => {
                                eprintln!("ZeroSh: {e}");
                                // コマンドのパースに失敗した場合は入力を再開するためmainスレッドに通知
                                shell_tx.send(ShellMsg::Continue(self.exit_val)).unwrap();
                            }
                        }
                    }
                    WorkerMsg::Signal(SIGCHILD) => {
                        self.wait_child(&shell_tx); // 子プロセスの状態変化管理
                    }
                    _ => (), // 無視
                }
            }
        });
    }

    /// 組み込みコマンドの場合はtrueを返す
    fn build_in_cmd(&mut self, cmd: &[(&str, Vec<&str>)], shell_tx: &SyncSender<ShellMsg>) -> bool {
        if cmd.len() > 1 {
            return false; // 組み込みコマンドのパイプは非対応なのでエラー
        }

        match cmd[0].0 {
            "exit" => self.run_exit(&cmd[0].1, shell_tx),
            "jobs" => self.run_jobs(shell_tx),
            "fg" => self.run_fg(&cmd[0].1, shell_tx),
            "cd" => self.run_cd(&cmd[0].1, shell_tx),
            _ => false,
        }
    }

    /// eixtコマンドを実行
    fn run_exit(&mut self, args: &[&str], shell_tx: &SyncSender<ShellMsg>) -> bool {
        // バックエンドで実行中のジョブがある場合は終了しない
        if !self.jobs.is_empty() {
            eprintln!("ジョブが実行中なので終了できません");
            self.exit_val = 1; //　失敗
            shell_tx.send(ShellMsg::Continue(self.exit_val)).unwrap(); // シェルを再開
            return true;
        }

        // 終了コードを取得
        let exit_val = if let Some(s) = args.get(1) {
            if let Ok(n) = (*s).parse::<i32>() {
                n
            } else {
                // 終了コードが整数ではない
                eprintln!("{s}は不正な引数です");
                self.exit_val = 1; // 失敗
                shell_tx.send(ShellMsg::Continue(self.exit_val)).unwrap(); // シェルを再開
                return true;
            }
        } else {
            self.exit_val
        };

        shell_tx.send(ShellMsg::Quit(self.exit_val)).unwrap(); // シェルを終了
        true
    }

    /// fgコマンドを実行
    fn run_fg(&mut self, args: &[&str], shell_tx: &SyncSender<ShellMsg>) -> bool {
        self.exit_val = 1; // とりあえず失敗に設定

        // 引数をチェック
        if args.len() < 2 {
            eprintln!("usage: fg 数字");
            shell_tx.send(ShellMsg::Continue(self.exit_val)).unwrap();
            return true;
        }

        // ジョブIDを取得
        if let Ok(n) = args[1].parse::<usize>() {
            if let Some((pgid, cmd)) = self.jobs.get(&n) {
                eprintln!("{n} 再開\t{cmd}");

                // フォアグラウンドプロセスに設定
                self.fg = Some(*pgid);
                // tcsetpgrpはファイルディスクリプタとプロセスグループIDを受け取り、
                // そのファイルディスクリプタに関連付けられたセッションの
                // フォアグラウンドプロセスグループを指定されたプロセスグループとする
                tcsetpgrp(libc::STDIN_FILENO, *pgid).unwrap();

                // ジョブの実行を再開
                // 引数で指定したプロセスグループに対してSIGCONTシグナルを送信する
                // 停止中のプロセスがSIGCONTを受信すると、実行が再開される
                // フォアグラウンドプロセスを変更した場合は、シェルの読み込みは再開しない
                killpg(*pgid, Signal::SIGCONT).unwrap();
                return true;
            }
        }

        // 失敗
        eprintln!("{}というジョブは見つかりませんでした。", args[1]);
        shell_tx.send(ShellMsg::Continue(self.exit_val)).unwrap(); // シェルを再開
        true
    }

    /// jobsコマンドを実行
    ///
    /// 現在シェルが管理して実行しているジョブ一覧を表示する
    fn run_jobs(&mut self, shell_tx: &SyncSender<ShellMsg>) -> bool {
        true // TODO
    }

    /// cdコマンドを実行
    fn run_cd(&mut self, args: &[&str], shell_tx: &SyncSender<ShellMsg>) -> bool {
        true // TODO
    }

    /// 子プロセスを生成。失敗した場合はシェルからの入力を再開させる必要あり。
    fn spawn_child(&mut self, line: &str, cmd: &[(&str, Vec<&str>)]) -> bool {
        assert_ne!(cmd.len(), 0); // コマンドが空でないか検査

        // ジョブIDを取得
        let job_id = if let Some(id) = self.get_new_job_id() {
            id
        } else {
            eprintln!("ZeroSh: 管理可能なジョブの最大値に到達");
            return false;
        };

        if cmd.len() > 2 {
            eprintln!("ZeroSh: 3つ以上のコマンドによるパイプはサポートしていません");
            return false;
        }

        let mut input = None; // 2つ目のプロセスの標準入力
        let mut output = None; // １つ目のプロセスの標準出力
        if cmd.len() == 2 {
            // パイプを作成
            let p = pipe().unwrap();
            input = Some(p.0);
            output = Some(p.1);
        }

        // パイプを閉じる関数を定義
        let cleanup_pipe = CleanuUp {
            f: || {
                if let Some(fd) = input {
                    syscall(|| unistd::close(fd)).unwrap();
                }
                if let Some(fd) = output {
                    syscall(|| unistd::close(fd)).unwrap();
                }
            },
        };

        let pgid;

        // １つ目のプロセスを生成
        //
        match fork_exec(Pid::from_raw(0), cmd[0].0, &cmd[0].1, None, output) {
            Ok(child) => {
                pgid = child;
            }
            Err(e) => {
                eprintln!("ZeroSh: プロセス生成エラー: {e}");
                return false;
            }
        }

        // プロセス、ジョブの情報を追加
        let info = ProcInfo {
            state: ProcState::Run,
            pgid,
        };
        let mut pids = HashMap::new();
        pids.insert(pgid, info.clone()); // 1つ目のプロセスの情報

        // 2つ目のプロセスを生成
        if cmd.len() == 2 {
            match fork_exec(pgid, cmd[1].0, &cmd[1].1, input, None) {
                Ok(child) => {
                    // 2つ目のプロセスの情報
                    pids.insert(child, info);
                }
                Err(e) => {
                    eprintln!("ZeroSh: プロセス生成エラー: {e}");
                    return false;
                }
            }
        }

        std::mem::drop(cleanup_pipe); // パイプをクローズ。ここでクローズしても、子プロセスでは残っている

        // ジョブ情報を追加して子プロセスをフォアグラウンドプロセスグループにする
        self.fg = Some(pgid);
        self.insert_job(job_id, pgid, pids, line);
        tcsetpgrp(libc::STDIN_FILENO, pgid).unwrap();

        true
    }

    /// 子プロセスの状態変化を管理
    fn wait_child(&mut self, shell_tx: &SyncSender<ShellMsg>) {
        // WUNTRACED: 子プロセスの停止
        // WNOHANG: ブロックしない
        // WCONTINUED: 実行再開
        let flag = Some(WaitPidFlag::WUNTRACED | )
    }
}

type CmdResult<'a> = Result<Vec<(&'a str, Vec<&'a str>)>, DynError>;

/// コマンドをパース
fn parse_cmd(line: &str) -> CmdResult {
    let parsed_cmds = vec![];

    for cmd in line.split('|') {
        let cmd = cmd.trim();
        if cmd.is_empty() {
            return Err();
        }
        let cmd_and_options: Vec<&str> = cmd.split_whitespace().collect();
        let cmd = cmd_and_options[0];
        let options = cmd_and_options[1..].to_vec();
        parsed_cmds.push((cmd, options))
    }
    Ok(parsed_cmds)
}

/// プロセスグループIDを指定してfork & exec
/// pgidが0の場合は子プロセスのプロセスIDが、プロセスグループIDとなる
///
/// - inputがSome(fd)の場合は、標準入力をfdと設定
/// - outputがSome(fd)の場合は、標準出力をfdと設定
fn fork_exec(
    pgid: Pid,
    filename: &str,
    args: &[&str],
    input: Option<i32>,
    output: Option<i32>,
) -> Result<Pid, DynError> {
    let filename = CString::new(filename).unwrap();
    let args: Vec<CString> = args.iter().map(|s| CString::new(*s).unwrap()).collect();

    match syscall(|| unsafe { fork() })? {
        // forkを呼び出し子プロセスを生成
        ForkResult::Parent { child, .. } => {
            // 子プロセスのプロセスグループIDをpgidに設定
            setpgid(child, pgid).unwrap();
            Ok(child)
        }
        ForkResult::Child => {
            // 子プロセスのプロセスグループIDをpgidに設定
            // setpgidの第一引数を0とすると、自プロセスのプロセスグループIDにpgid設定される
            // 親と子の両方でsetpgidを呼び出している理由は、どちらが先に実行されるか決定不能であり、
            // 確実にプロセスグループIDを設定するためである
            setpgid(Pid::from_raw(0), pgid).unwrap();

            // 標準入出力を引数で与えられたものに置き換える
            // nix::unistd::dup2はシステムコールのラッパで、
            // 第一引数に元となるファイルディスクリプタを、
            // 第二引数に置き換え先のファイルディスクリプタを指定する
            // 第二引数に示すファイルディスクリプタがすでに使われていた場合はクローズする
            if let Some(infd) = input {
                syscall(|| dup2(infd, libc::STDIN_FILENO)).unwrap();
            }
            if let Some(outfd) = output {
                syscall(|| dup2(outfd, libc::STDOUT_FILENO)).unwrap();
            }

            // 標準入出力と標準エラー出力以外のファイルディスクリプタは不要なので
            // signal_hookで利用されるUnixドメインソケットとpipeをクローズ
            for i in 3..=6 {
                let _ = syscall(|| unistd::close(i));
            }

            // 実行ファイルをメモリに読み込み
            // nix::unistd::execvp関数を呼び足、実行ファイルを実行
            // execvpも同名のシステムコールのラッパであり、
            // 第一引数に実行ファイルへのパスを、第２引数にコマンドライン引数を指定する
            match execvp(&filename, &args) {
                Err(_) => {
                    // 標準エラー出力への書き込みにprintln!ではなく、write!を利用しているのは、
                    // fork後に安全に利用可能なシステムコールは限定されており、
                    // 内部でメモリ確保を行うprintln!の利用は避けるべきだからである。
                    // 詳細はman signal-safety
                    // https://qiita.com/rarul/items/090920b850acc4b7e910
                    unistd::write(libc::STDERR_FILENO, "不明なコマンドを実行\n".as_bytes()).ok();
                    exit(1);
                }
                Ok(_) => unreachable!(),
            }
        }
    }
}

/// ドロップ時にクロージャfを呼び出す型
///
/// フィールドfに示されるクロージャをドロップ時に実行するのみ
/// ファイルディスクリプタのクローズ処理に用いる
/// Rustはメモリなどのリソース解放は、ライブラリやコンパイラが自動で行ってくれるが
/// 自前でシステムコールを用いてパイプを作成した場合には、プログラマ自らが行う必要がある
struct CleanuUp<F>
where
    F: Fn(),
{
    f: F,
}

impl<F> Drop for CleanuUp<F>
where
    F: Fn(),
{
    fn drop(&mut self) {
        (self.f)()
    }
}
