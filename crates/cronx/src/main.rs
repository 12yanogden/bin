fn main() {
    if let Err(e) = cronx_upstream::run() {
        eprintln!("cronx: {e:#}");
        std::process::exit(1);
    }
}
