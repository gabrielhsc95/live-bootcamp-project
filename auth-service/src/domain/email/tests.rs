use super::*;

// TODO: try
// fake = "=2.3.0"
// quickcheck = "0.9.2"
// quickcheck_macros = "0.9.1"

#[test]
fn valid_email() {
    let test_cases = [
        "iasjd@iasjd.asd",
        "gabrielhsc95@gmail.com",
        "okasdo.oaskdo.oaskdoaskd@oaksok.oakso",
    ];
    for tc in test_cases {
        let result = Email::parse(tc);
        assert!(result.is_ok(), "failed for test case: {}", tc);
    }
}

#[test]
fn invalid_email() {
    let test_cases = [
        "aiksujdioasud",
        "oaskdoas@opalspoal@aoksoas",
        "ioasjdoiasjkd@",
    ];
    for tc in test_cases {
        let result = Email::parse(tc);
        assert!(result.is_err(), "failed for test case: {}", tc);
    }
}
