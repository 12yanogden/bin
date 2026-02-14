use bin_lib::{fmt, sets};
use std::process::ExitCode;

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() != 2 {
        return Err("usage: arr_intersect <input1> <input2>".to_string());
    }

    let mut stdin_used = false;
    let a = sets::resolve_input(&args[0], &mut stdin_used)?;
    let b = sets::resolve_input(&args[1], &mut stdin_used)?;

    let result = sets::intersect(&a, &b);
    for line in &result {
        println!("{}", line);
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            fmt::fail(&e);
            ExitCode::from(1)
        }
    }
}
