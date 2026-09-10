use std::path::PathBuf;

use regex::Regex;
use rootproof_core::FailureSignature;

pub fn parse_rust_failure(stdout: &str, stderr: &str) -> Option<FailureSignature> {
    let combined = combine_output(stdout, stderr);

    parse_rust_panic(&combined)
}

fn combine_output(stdout: &str, stderr: &str) -> String {
    match (stdout.trim().is_empty(), stderr.trim().is_empty()) {
        (false, false) => {
            format!("{stdout}\n{stderr}")
        }

        (false, true) => stdout.to_owned(),

        (true, false) => stderr.to_owned(),

        (true, true) => String::new(),
    }
}

fn parse_rust_panic(output: &str) -> Option<FailureSignature> {
    let location_regex = Regex::new(r"panicked at (?P<file>.+?):(?P<line>\d+):(?P<column>\d+):")
        .expect("valid panic location regex");

    let captures = location_regex.captures(output)?;

    let file = captures.name("file")?.as_str();

    let line = captures.name("line")?.as_str().parse::<u32>().ok()?;

    let column = captures.name("column")?.as_str().parse::<u32>().ok()?;

    let whole_match = captures.get(0)?;

    let remaining = &output[whole_match.end()..];

    let message = extract_message(remaining);

    Some(FailureSignature {
        error_type: Some("panic".to_owned()),
        message,
        file: Some(PathBuf::from(file)),
        line: Some(line),
        column: Some(column),
        stack_frames: Vec::new(),
    })
}

fn extract_message(output_after_location: &str) -> Option<String> {
    output_after_location
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("note:"))
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rust_panic() {
        let stdout = r#"
            running 1 test
            test tests::intentionally_failing_test ... FAILED

            thread 'tests::intentionally_failing_test' (721725) panicked at fixtures/rust-failing/src/lib.rs:11:9:
            assertion `left == right` failed
            left: 5
            right: 10
            note: run with `RUST_BACKTRACE=1`
            "#;

        let failure = parse_rust_failure(stdout, "").expect("parse failure");

        assert_eq!(failure.error_type.as_deref(), Some("panic"));

        assert_eq!(
            failure.message.as_deref(),
            Some("assertion `left == right` failed")
        );

        assert_eq!(
            failure.file,
            Some(PathBuf::from("fixtures/rust-failing/src/lib.rs"))
        );

        assert_eq!(failure.line, Some(11));

        assert_eq!(failure.column, Some(9));
    }

    #[test]
    fn returns_none_when_no_panic_exists() {
        let output = r#"
            running 1 test
            test tests::works ... ok

            test result: ok.
            "#;

        let failure = parse_rust_failure(output, "");

        assert!(failure.is_none());
    }

    #[test]
    fn parses_failure_from_stderr() {
        let stderr = r#"
            thread 'main' panicked at src/main.rs:42:5:
            called `Option::unwrap()` on a `None` value
            "#;

        let failure = parse_rust_failure("", stderr).expect("parse stderr");

        assert_eq!(
            failure.message.as_deref(),
            Some("called `Option::unwrap()` on a `None` value")
        );

        assert_eq!(failure.line, Some(42));
    }
}
