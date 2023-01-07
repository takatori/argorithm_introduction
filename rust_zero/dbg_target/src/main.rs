use std::arch::asm;

use nix::{
    sys::signal::{kill, Signal},
    unistd::getpid,
};

fn main() {
    // int命令はx86_64でソフトウェア割り込みを発生させる命令
    // この命令の実行後に割り込みハンドラが起動され、その後にSIGTRAPが発行されてプロセスが停止する
    // これがブレークポイントの正体。ブレークポイントを設定するには停止したいアドレスを特定してint 3に書き換えれば良い
    println!("int 3");
    unsafe { asm!("int 3") };

    println!("kill -SIGTRAP");
    let pid = getpid();
    kill(pid, Signal::SIGTRAP).unwrap(); // SIGTRAPシグナルを自身に発行

    for i in 0..3 {
        unsafe { asm!("nop") }; // nopは何もしない命令
        println!("i = {i}");
    }
}
