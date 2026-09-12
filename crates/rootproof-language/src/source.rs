use std::{fs, path::Path};

use rootproof_core::SourceContext;

pub fn read_source_context(
    repo: &Path,
    relative_file: &Path,
    target_line: u32,
    context_lines: u32,
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

    let start_index = target_index.saturating_sub(context_lines as usize);

    let end_index = std::cmp::min(target_index + context_lines as usize + 1, lines.len());

    let mut content = String::new();

    for (index, line) in lines[start_index..end_index].iter().enumerate() {
        let line_number = start_index + index + 1;

        content.push_str(&format!("{line_number:>4} | {line}\n"));
    }

    Ok(SourceContext {
        file: relative_file.to_path_buf(),
        target_line,
        start_line: (start_index + 1) as u32,
        end_line: end_index as u32,
        content,
    })
}

pub fn resolve_failure_file(repo: &Path, failure_file: &Path) -> Option<std::path::PathBuf> {
    let direct = repo.join(failure_file);

    if direct.is_file() {
        return Some(failure_file.to_path_buf());
    }

    for ancestor in repo.ancestors() {
        if let Ok(relative) = failure_file.strip_prefix(ancestor.file_name()?) {
            if repo.join(relative).is_file() {
                return Some(relative.to_path_buf());
            }
        }
    }

    if let Some(file_name) = failure_file.file_name() {
        let candidate = Path::new("src").join(file_name);

        if repo.join(&candidate).is_file() {
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
        let temp = tempfile::tempdir().expect("create temporary directory");

        let source = r#"
                            fn one() {}
                            fn two() {}
                            fn three() {}
                            fn four() {}
                            fn five() {}
                            "#;

        let file = temp.path().join("src.rs");

        fs::write(&file, source).expect("write source");

        let context =
            read_source_context(temp.path(), Path::new("src.rs"), 4, 1).expect("read context");

        assert!(context.content.contains("fn three()"));

        assert!(context.content.contains("fn four()"));

        assert!(context.content.contains("fn five()"));
    }
}
