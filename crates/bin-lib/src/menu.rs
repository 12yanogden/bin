use std::io::{self, BufRead, Write};

/// Display a numbered menu of options and prompt the user to select one.
/// Returns the selected item from the list.
///
/// Reads from `input` and writes to `output` to support testing.
/// For production use, pass `io::stdin().lock()` and `io::stderr()`.
pub fn select_from_menu<R: BufRead, W: Write>(
    options: &[String],
    input: &mut R,
    output: &mut W,
) -> Result<String, String> {
    if options.is_empty() {
        return Err("no options to select from".to_string());
    }
    if options.len() == 1 {
        return Ok(options[0].clone());
    }

    for (i, option) in options.iter().enumerate() {
        writeln!(output, "{}) {}", i, option).map_err(|e| e.to_string())?;
    }

    loop {
        write!(output, "Selection: ").map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;

        let mut line = String::new();
        input.read_line(&mut line).map_err(|e| e.to_string())?;

        // Handle EOF - read_line returns Ok(0) and line stays empty
        if line.is_empty() {
            return Err("unexpected end of input".to_string());
        }

        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        match trimmed.parse::<usize>() {
            Ok(n) if n < options.len() => return Ok(options[n].clone()),
            _ => {
                writeln!(output, "invalid response: {}", trimmed).map_err(|e| e.to_string())?;
            }
        }
    }
}

/// Convenience wrapper for stdin/stderr usage.
pub fn prompt_menu(options: &[String]) -> Result<String, String> {
    let mut stdin = io::stdin().lock();
    let mut stderr = io::stderr();
    select_from_menu(options, &mut stdin, &mut stderr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_option_returns_immediately() {
        let options = vec!["main".to_string()];
        let mut input = io::Cursor::new(b"");
        let mut output = Vec::new();
        let result = select_from_menu(&options, &mut input, &mut output);
        assert_eq!(result.unwrap(), "main");
    }

    #[test]
    fn valid_selection_returns_option() {
        let options = vec!["main".to_string(), "develop".to_string()];
        let mut input = io::Cursor::new(b"1\n");
        let mut output = Vec::new();
        let result = select_from_menu(&options, &mut input, &mut output);
        assert_eq!(result.unwrap(), "develop");
    }

    #[test]
    fn invalid_then_valid_selection() {
        let options = vec!["main".to_string(), "develop".to_string()];
        let mut input = io::Cursor::new(b"abc\n5\n0\n");
        let mut output = Vec::new();
        let result = select_from_menu(&options, &mut input, &mut output);
        assert_eq!(result.unwrap(), "main");
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("invalid response: abc"));
        assert!(output_str.contains("invalid response: 5"));
    }

    #[test]
    fn empty_options_returns_error() {
        let options: Vec<String> = vec![];
        let mut input = io::Cursor::new(b"");
        let mut output = Vec::new();
        let result = select_from_menu(&options, &mut input, &mut output);
        assert!(result.is_err());
    }

    #[test]
    fn eof_returns_error() {
        let options = vec!["main".to_string(), "develop".to_string()];
        let mut input = io::Cursor::new(b""); // EOF immediately
        let mut output = Vec::new();
        let result = select_from_menu(&options, &mut input, &mut output);
        assert!(result.is_err());
    }
}
