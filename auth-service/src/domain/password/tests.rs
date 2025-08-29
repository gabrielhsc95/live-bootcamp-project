use super::*;

// TODO: try
// fake = "=2.3.0"
// quickcheck = "0.9.2"
// quickcheck_macros = "0.9.1"

#[test]
fn valid_password() {
    let test_cases = [
        "Password1!",
        "LongPasswordWithNumb3rAndSpeci@lCharacter",
        "VeryStrongPassword!1",
    ];
    for tc in test_cases {
        let result = Password::parse(tc);
        assert!(result.is_ok(), "failed for test case: {}", tc);
    }
}

#[test]
fn invalid_password() {
    let test_cases = [
        "Small1!",
        "NoSpecialCharacters",
        "NoNumber!",
        "no_capital_letter",
        "123456",
    ];
    for tc in test_cases {
        let result = Password::parse(tc);
        assert!(result.is_err(), "failed for test case: {}", tc);
    }
}
