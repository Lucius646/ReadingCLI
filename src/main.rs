fn main() {
    if let Err(err) = reading_cli::app::run() {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }
}
