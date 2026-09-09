use std::time::Instant;

use tokio::{
    process::Command,
    time::timeout,
};

use crate::{
    CommandResult,
    CommandSpec,
    ExecutorError,
};

pub async fn execute(
    spec: CommandSpec,
) -> Result<CommandResult, ExecutorError> {
    if !spec.working_directory.exists() {
        return Err(
            ExecutorError::WorkingDirectoryNotFound(
                spec.working_directory,
            ),
        );
    }

    if !spec.working_directory.is_dir() {
        return Err(
            ExecutorError::WorkingDirectoryNotDirectory(
                spec.working_directory,
            ),
        );
    }

    let started = Instant::now();

    let mut command = Command::new(&spec.program);

    command
        .args(&spec.args)
        .current_dir(&spec.working_directory)
        .kill_on_drop(true);

    let output_future = command.output();

    let output = timeout(
        spec.timeout,
        output_future,
    )
    .await
    .map_err(|_| ExecutorError::Timeout {
        program: spec.program.clone(),
        timeout: spec.timeout,
    })?
    .map_err(|source| ExecutorError::Spawn {
        program: spec.program.clone(),
        source,
    })?;

    let duration = started.elapsed();

    Ok(CommandResult {
        program: spec.program,
        args: spec.args,
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(
            &output.stdout,
        )
        .into_owned(),
        stderr: String::from_utf8_lossy(
            &output.stderr,
        )
        .into_owned(),
        duration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn captures_successful_command() {
        let temp =
            tempfile::tempdir()
                .expect("create temp directory");

        let spec =
            CommandSpec::new(
                "rustc",
                temp.path(),
            )
            .arg("--version");

        let result =
            execute(spec)
                .await
                .expect("execute command");

        assert!(result.success());
        assert_eq!(
            result.exit_code,
            Some(0)
        );

        assert!(
            result.stdout.contains(
                "rustc"
            )
        );
    }

    #[tokio::test]
    async fn rejects_missing_working_directory() {
        let spec =
            CommandSpec::new(
                "rustc",
                "/definitely/not/rootproof",
            );

        let result =
            execute(spec).await;

        assert!(matches!(
            result,
            Err(
                ExecutorError::WorkingDirectoryNotFound(_)
            )
        ));
    }

    #[tokio::test]
    async fn returns_spawn_error_for_missing_program() {
        let temp =
            tempfile::tempdir()
                .expect("create temp directory");

        let spec =
            CommandSpec::new(
                "definitely-not-a-real-command-rootproof",
                temp.path(),
            );

        let result =
            execute(spec).await;

        assert!(matches!(
            result,
            Err(
                ExecutorError::Spawn { .. }
            )
        ));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn preserves_non_zero_exit_code() {
        let temp =
            tempfile::tempdir()
                .expect("create temp directory");

        let spec =
            CommandSpec::new(
                "sh",
                temp.path(),
            )
            .args([
                "-c",
                "echo failure >&2; exit 7",
            ]);

        let result =
            execute(spec)
                .await
                .expect(
                    "command execution itself should succeed",
                );

        assert!(!result.success());

        assert_eq!(
            result.exit_code,
            Some(7)
        );

        assert!(
            result.stderr.contains(
                "failure"
            )
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn terminates_command_after_timeout() {
        let temp =
            tempfile::tempdir()
                .expect("create temp directory");

        let spec =
            CommandSpec::new(
                "sleep",
                temp.path(),
            )
            .arg("5")
            .timeout(
                Duration::from_millis(50),
            );

        let result =
            execute(spec).await;

        assert!(matches!(
            result,
            Err(
                ExecutorError::Timeout { .. }
            )
        ));
    }
}