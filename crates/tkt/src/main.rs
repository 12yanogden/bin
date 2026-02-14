use bin_lib::git;
use bin_lib::ticket::extract_ticket;
use std::process::ExitCode;

fn main() -> ExitCode {
    let branch = match git::get_current_branch() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("tkt: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match extract_ticket(&branch) {
        Some(ticket) => {
            println!("{}", ticket);
            ExitCode::SUCCESS
        }
        None => ExitCode::SUCCESS,
    }
}

#[cfg(test)]
mod tests {
    use bin_lib::ticket::extract_ticket;

    #[test]
    fn feature_branch() {
        assert_eq!(extract_ticket("feature/PROJ-123-add-login"), Some("PROJ-123"));
    }

    #[test]
    fn bare_ticket() {
        assert_eq!(extract_ticket("BIN-5"), Some("BIN-5"));
    }

    #[test]
    fn ticket_at_start() {
        assert_eq!(extract_ticket("JIRA-42/some-description"), Some("JIRA-42"));
    }

    #[test]
    fn no_ticket() {
        assert_eq!(extract_ticket("main"), None);
    }

    #[test]
    fn lowercase_not_matched() {
        assert_eq!(extract_ticket("fix/proj-99-typo"), None);
    }

    #[test]
    fn multiple_tickets_returns_first() {
        assert_eq!(extract_ticket("PROJ-1-PROJ-2"), Some("PROJ-1"));
    }

    #[test]
    fn single_letter_project_key() {
        assert_eq!(extract_ticket("X-1"), Some("X-1"));
    }

    #[test]
    fn empty_string_returns_none() {
        assert_eq!(extract_ticket(""), None);
    }

    #[test]
    fn prefix_without_digits_returns_none() {
        assert_eq!(extract_ticket("PROJ-"), None);
    }

    #[test]
    fn mixed_case_prefix_returns_none() {
        assert_eq!(extract_ticket("Proj-123"), None);
    }

    #[test]
    fn digits_only_returns_none() {
        assert_eq!(extract_ticket("123-456"), None);
    }

    #[test]
    fn ticket_embedded_in_word_still_matches() {
        assert_eq!(extract_ticket("myPROJ-99fix"), Some("PROJ-99"));
    }

    #[test]
    fn bugfix_branch_with_ticket() {
        assert_eq!(extract_ticket("bugfix/TEAM-9-fix-crash"), Some("TEAM-9"));
    }

    #[test]
    fn exact_ticket_branch() {
        assert_eq!(extract_ticket("ABC-123"), Some("ABC-123"));
    }

    #[test]
    fn multiple_tickets_returns_first_only() {
        assert_eq!(extract_ticket("feat/ABC-123-DEF-456"), Some("ABC-123"));
    }
}
