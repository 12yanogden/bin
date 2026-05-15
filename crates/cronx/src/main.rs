fn main() {
    if let Err(e) = cronx::run() {
        eprintln!("cronx: {e:#}");
        std::process::exit(1);
    }
}
