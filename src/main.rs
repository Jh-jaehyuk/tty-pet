fn main() {
    if let Err(error) = tty_pet::cli::run() {
        eprintln!("error: {error:?}");
        std::process::exit(1);
    }
}
