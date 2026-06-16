// 程序入口：调用应用主流程，并把错误转换为终端输出和失败退出码。
fn main() {
    if let Err(err) = reading_cli::app::run() {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }
}
