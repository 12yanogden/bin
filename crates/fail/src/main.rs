use bin_lib::fmt;
use std::env;

fn main() {
    let msg = env::args().nth(1).unwrap_or_default();
    fmt::fail(&msg);
}
