fn main() {
    if let Err(error) = tty_pet::mcp::run_stdio() {
        eprintln!("error: {error:?}");
        std::process::exit(1);
    }
}
