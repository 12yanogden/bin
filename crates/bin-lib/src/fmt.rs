pub fn pass(msg: &str) {
    println!("\x1b[32m[ PASS ]\x1b[0m {}", msg);
}

pub fn fail(msg: &str) {
    eprintln!("\x1b[31m[ FAIL ]\x1b[0m {}", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_does_not_panic() {
        pass("test message");
    }

    #[test]
    fn fail_does_not_panic() {
        fail("test message");
    }
}
