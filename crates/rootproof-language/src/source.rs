use std::{fs, path::{Path, PathBuf}};

use rootproof_core::SourceContext;

pub fn read_source_context(repo: &Path, relative_file: &Path, target_line: u32,context_lines: u32,
    ) -> Result<SourceContext, std::io::Error> {
    let full_path = repo.join(relative_file);

    let source = fs::read_to_string(&full_path)?;

    let lines: Vec<&str> = source.lines().collect();

    if lines.is_empty() {
        return Ok(SourceContext {
            file: relative_file.to_path_buf(),
            target_line,
            start_line: 0,
            end_line: 0,
            content: String::new(),
        });
    }

    let target_index = target_line.saturating_sub(1) as usize;

    let start_index =
        target_index.saturating_sub(context_lines as usize);

    let end_index = std::cmp::min(
        target_index + context_lines as usize + 1,
        lines.len(),
    );

    let mut content = String::new();

    for (index, line) in
        lines[start_index..end_index]
            .iter()
            .enumerate()
    {
        let line_number =
            start_index + index + 1;

        content.push_str(
            &format!(
                "{line_number:>4} | {line}\n"
            ),
        );
    }

    Ok(SourceContext {
        file: relative_file.to_path_buf(),
        target_line,
        start_line: (start_index + 1) as u32,
        end_line: end_index as u32,
        content,
    })
}

pub fn resolve_failure_file(
    repo: &Path,
    failure_file: &Path,
) -> Option<PathBuf> {
    let direct =
        repo.join(failure_file);

    if direct.is_file() {
        return Some(
            failure_file.to_path_buf()
        );
    }

    if let Some(repo_name) =
        repo.file_name(){
        let components: Vec<_> =
            failure_file
                .components()
                .collect();

        for (index, component) in
            components
                .iter()
                .enumerate(){
            if component.as_os_str()
                == repo_name
            {
                let mut relative = PathBuf::new();

                for remaining in components.iter().skip(index + 1){
                    relative.push(
                        remaining.as_os_str()
                    );
                }

                if !relative
                    .as_os_str()
                    .is_empty()
                    && repo
                        .join(&relative)
                        .is_file()
                {
                    return Some(relative);
                }
            }
        }
    }

    let components: Vec<_> =
        failure_file
            .components()
            .collect();

    for start in 1..components.len() {
        let mut candidate =
            PathBuf::new();

        for component in
            components
                .iter()
                .skip(start)
        {
            candidate.push(
                component.as_os_str()
            );
        }

        if repo
            .join(&candidate)
            .is_file()
        {
            return Some(candidate);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn reads_lines_around_target() {
        let temp =
            tempfile::tempdir().expect("create temporary directory");

        let source = "\
            fn one() {}
            fn two() {}
            fn three() {}
            fn four() {}
            fn five() {}
            ";

        let file =
            temp.path().join("src.rs");

        fs::write(&file, source).expect("write source");

        let context =
            read_source_context(
                temp.path(),
                Path::new("src.rs"),
                3,
                1,
            )
            .expect(
                "read context"
            );

        assert!(
            context
                .content
                .contains(
                    "fn two()"
                )
        );

        assert!(
            context
                .content
                .contains(
                    "fn three()"
                )
        );

        assert!(
            context
                .content
                .contains(
                    "fn four()"
                )
        );
    }

    #[test]
    fn resolves_repository_relative_path() {
        let temp =
            tempfile::tempdir()
                .expect(
                    "create temporary directory"
                );

        fs::create_dir_all(
            temp
                .path()
                .join("src"),
        )
        .expect(
            "create src directory"
        );

        fs::write(
            temp
                .path()
                .join("src/lib.rs"),
            "fn example() {}",
        )
        .expect(
            "write source file"
        );

        let resolved =
            resolve_failure_file(
                temp.path(),
                Path::new(
                    "src/lib.rs"
                ),
            );

        assert_eq!(
            resolved,
            Some(
                PathBuf::from(
                    "src/lib.rs"
                )
            )
        );
    }

    #[test]
    fn resolves_failure_path_with_repo_prefix() {
        let temp =
            tempfile::tempdir()
                .expect(
                    "create temporary directory"
                );

        let repo =
            temp
                .path()
                .join("fixtures")
                .join(
                    "rust-failing"
                );

        fs::create_dir_all(
            repo.join("src"),
        )
        .expect(
            "create src directory"
        );

        fs::write(
            repo.join(
                "src/lib.rs"
            ),
            "fn example() {}",
        )
        .expect(
            "write source file"
        );

        let failure_file =
            Path::new(
                "fixtures/rust-failing/src/lib.rs"
            );

        let resolved =
            resolve_failure_file(
                &repo,
                failure_file,
            );

        assert_eq!(
            resolved,
            Some(
                PathBuf::from(
                    "src/lib.rs"
                )
            )
        );
    }

    #[test]
    fn returns_none_for_missing_source_file() {
        let temp =
            tempfile::tempdir()
                .expect(
                    "create temporary directory"
                );

        let resolved =
            resolve_failure_file(
                temp.path(),
                Path::new(
                    "src/missing.rs"
                ),
            );

        assert_eq!(
            resolved,
            None
        );
    }
}
