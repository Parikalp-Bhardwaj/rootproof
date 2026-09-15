use std::{
    fs, io,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

pub struct IsolatedRepository {
    _temp_dir: TempDir,
    path: PathBuf,
}

impl IsolatedRepository {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn create_isolated_repository(source: &Path) -> Result<IsolatedRepository, io::Error> {
    let temp_dir = tempfile::tempdir()?;

    let destination = temp_dir.path().join("repository");
    fs::create_dir_all(&destination)?;

    copy_repository(source, &destination)?;

    Ok(IsolatedRepository {
        _temp_dir: temp_dir,
        path: destination,
    })
}

fn copy_repository(source: &Path, destination: &Path) -> Result<(), io::Error> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;

        let file_type = entry.file_type()?;

        let name = entry.file_name();

        if name == ".git" || name == "target" {
            continue;
        }

        let source_path = entry.path();

        let destination_path = destination.join(&name);
        if file_type.is_dir() {
            fs::create_dir_all(&destination_path)?;

            copy_repository(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path)?;
        }

        // Symlinks are intentionally skipped
        // in the initial MVP isolation layer.
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_repository_files() {
        let source = tempfile::tempdir().expect("create source");

        fs::create_dir_all(source.path().join("src")).expect("create src");

        fs::write(
            source.path().join("Cargo.toml"),
            "[package]\nname = \"example\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .expect("write Cargo.toml");

        fs::write(source.path().join("src/lib.rs"), "pub fn example() {}").expect("write lib.rs");

        let isolated =
            create_isolated_repository(source.path()).expect("create isolated repository");

        assert!(isolated.path().join("Cargo.toml").is_file());

        assert!(isolated.path().join("src/lib.rs").is_file());
    }

    #[test]
    fn does_not_copy_target_directory() {
        let source = tempfile::tempdir().expect("create source");

        fs::create_dir_all(source.path().join("target")).expect("create target");

        fs::write(source.path().join("target/temp"), "ignored").expect("write target file");

        let isolated = create_isolated_repository(source.path()).expect("create isolation");

        assert!(!isolated.path().join("target").exists());
    }
}
