use std::{fs, path::Path};

use rootproof_core::CandidateFix;

pub fn apply_candidate_fix(repo: &Path, fix: &CandidateFix) -> Result<(), String> {
    validate_target_path(&fix.target_file)?;

    if fix.original_code.trim().is_empty() {
        return Err("candidate fix original code is empty".to_owned());
    }

    if fix.replacement_code == fix.original_code {
        return Err("candidate fix does not change the source".to_owned());
    }

    validate_replacement_code(&fix.replacement_code)?;

    let target = repo.join(&fix.target_file);

    if !target.is_file() {
        return Err(format!(
            "candidate fix target file does not exist: {}",
            fix.target_file.display()
        ));
    }

    let source = fs::read_to_string(&target).map_err(|error| {
        format!(
            "failed to read candidate fix target {}: {error}",
            fix.target_file.display()
        )
    })?;

    let occurrence_count = source.match_indices(&fix.original_code).count();

    match occurrence_count {
        0 => {
            return Err("candidate fix original code was not found in target file".to_owned());
        }

        1 => {}

        count => {
            return Err(format!(
                "candidate fix original code occurs {count} times; exactly one match is required"
            ));
        }
    }

    let updated = source.replacen(&fix.original_code, &fix.replacement_code, 1);

    fs::write(&target, updated).map_err(|error| {
        format!(
            "failed to write candidate fix target {}: {error}",
            fix.target_file.display()
        )
    })?;

    Ok(())
}

fn validate_target_path(path: &Path) -> Result<(), String> {
    if path.is_absolute() {
        return Err("candidate fix target path must be repository-relative".to_owned());
    }

    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err("candidate fix target path must not contain `..`".to_owned());
    }

    Ok(())
}

fn validate_replacement_code(replacement: &str) -> Result<(), String> {
    if replacement.trim().is_empty() {
        return Err("candidate fix replacement code is empty".to_owned());
    }

    let forbidden_patterns = ["#[ignore]", "#[cfg(any())]", "todo!()", "unimplemented!()"];

    for pattern in forbidden_patterns {
        if replacement.contains(pattern) {
            return Err(format!(
                "candidate fix contains forbidden validation-bypass pattern `{pattern}`"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    fn candidate_fix() -> CandidateFix {
        CandidateFix {
            hypothesis_id: "H1".to_owned(),
            target_file: PathBuf::from("src/lib.rs"),
            original_code: "assert_eq!(divide(10, 2), 10);".to_owned(),
            replacement_code: "assert_eq!(divide(10, 2), 5);".to_owned(),
            rationale: "Correct expected value.".to_owned(),
        }
    }

    #[test]
    fn applies_exact_candidate_fix() {
        let temp = tempfile::tempdir().expect("create temp dir");

        fs::create_dir_all(temp.path().join("src")).expect("create src");
        fs::write(
            temp.path().join("src/lib.rs"),
            r#"
                pub fn divide(a: i32, b: i32) -> i32 {
                    a / b
                }

                #[test]
                fn example() {
                    assert_eq!(divide(10, 2), 10);
                }
                "#,
        )
        .expect("write lib");

        apply_candidate_fix(temp.path(), &candidate_fix()).expect("apply fix");

        let updated = fs::read_to_string(temp.path().join("src/lib.rs")).expect("read updated lib");

        assert!(updated.contains("assert_eq!(divide(10, 2), 5);"));

        assert!(!updated.contains("assert_eq!(divide(10, 2), 10);"));
    }

    #[test]
    fn rejects_missing_original_code() {
        let temp = tempfile::tempdir().expect("create temp dir");

        fs::create_dir_all(temp.path().join("src")).expect("create src");

        fs::write(
            temp.path().join("src/lib.rs"),
            "pub fn divide(a: i32, b: i32) -> i32 { a / b }",
        )
        .expect("write lib");

        let result = apply_candidate_fix(temp.path(), &candidate_fix());

        assert!(result.is_err());
    }

    #[test]
    fn rejects_parent_directory_escape() {
        let mut fix = candidate_fix();

        fix.target_file = PathBuf::from("../outside.rs");

        let result = validate_target_path(&fix.target_file);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_absolute_target_path() {
        let result = validate_target_path(Path::new("/tmp/outside.rs"));

        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_replacement_code() {
        let mut fix = candidate_fix();

        fix.replacement_code = "   ".to_owned();

        let temp = tempfile::tempdir().expect("create temp dir");

        fs::create_dir_all(temp.path().join("src")).expect("create src");

        fs::write(
            temp.path().join("src/lib.rs"),
            "assert_eq!(divide(10, 2), 10);",
        )
        .expect("write lib");

        let result = apply_candidate_fix(temp.path(), &fix);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_ignored_test_replacement() {
        let result = validate_replacement_code(
            r#"
                    #[ignore]
                    #[test]
                    fn example() {}
                    "#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn rejects_disabled_cfg_replacement() {
        let result = validate_replacement_code(
            r#"
                #[cfg(any())]
                #[test]
                fn example() {}
                "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn rejects_todo_replacement() {
        let result = validate_replacement_code("todo!()");
        assert!(result.is_err());
    }

    #[test]
    fn accepts_normal_replacement() {
        let result = validate_replacement_code("assert_eq!(divide(10, 2), 5);");
        assert!(result.is_ok());
    }
}
