pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const RESET: &str = "\x1b[0m";

pub fn pass(msg: &str) {
    println!("[ {}PASS{} ] {}", GREEN, RESET, msg);
}

pub fn fail(msg: &str) {
    println!("[ {}FAIL{} ] {}", RED, RESET, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_output() {
        // Verify it doesn't panic; output format is tested in integration tests
        pass("test message");
    }

    #[test]
    fn fail_output() {
        fail("test message");
    }
}
