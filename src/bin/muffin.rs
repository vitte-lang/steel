fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = muffin::commands::run_cli(&args);
    std::process::exit(code);
}
