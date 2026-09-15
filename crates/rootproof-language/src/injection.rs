use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn inject_reproduction_test(
    repo: &Path,
    relative_source_file: &Path,
    reproduction_code: &str,
) -> Result<(), std::io::Error> {
    let source_file = repo.join(relative_source_file);

    let metadata = fs::metadata(&source_file)?;

    if !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "reproduction target is not a file",
        ));
    }

    let mut file = OpenOptions::new().append(true).open(&source_file)?;

    writeln!(file)?;

    writeln!(file, "#[cfg(test)]")?;

    writeln!(file, "mod rootproof_generated_reproduction {{")?;

    writeln!(file, "    use super::*;")?;

    for line in reproduction_code.lines() {
        writeln!(file, "    {line}")?;
    }

    writeln!(file, "}}")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_reproduction_module() {
        let temp = tempfile::tempdir().expect("create temp dir");

        fs::create_dir_all(temp.path().join("src")).expect("create src");

        fs::write(
            temp.path().join("src/lib.rs"),
            "pub fn divide(a: i32, b: i32) -> i32 { a / b }\n",
        )
        .expect("write lib");

        inject_reproduction_test(
            temp.path(),
            Path::new("src/lib.rs"),
            r#"
                #[test]
                fn rootproof_reproduction_h1() {
                    assert_eq!(divide(10, 2), 10);
                }
                "#,
        )
        .expect("inject test");

        let result = fs::read_to_string(temp.path().join("src/lib.rs")).expect("read lib");

        assert!(result.contains("mod rootproof_generated_reproduction"));

        assert!(result.contains("rootproof_reproduction_h1"));
    }
}
