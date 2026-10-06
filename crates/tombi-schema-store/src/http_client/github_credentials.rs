use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use reqwest::header::HeaderValue;
use tokio::process::Command;

use super::{FetchError, http_timeout_secs};

pub(super) async fn github_authorization() -> Result<Option<HeaderValue>, FetchError> {
    if let Some(token) = environment_token(|name| std::env::var(name).ok()) {
        return authorization_header(&token).map(Some);
    }

    let Some(executable) = github_cli_path() else {
        return Ok(None);
    };
    let mut command = Command::new(executable);
    command
        .args(["auth", "token", "--hostname", "github.com"])
        .env_remove("GH_TOKEN")
        .env_remove("GITHUB_TOKEN");
    authorization_from_command(command, Duration::from_secs(http_timeout_secs())).await
}

fn environment_token(mut lookup: impl FnMut(&str) -> Option<String>) -> Option<String> {
    ["GH_TOKEN", "GITHUB_TOKEN"]
        .into_iter()
        .find_map(|name| lookup(name).filter(|token| !token.trim().is_empty()))
}

fn authorization_header(token: &str) -> Result<HeaderValue, FetchError> {
    let mut header = HeaderValue::from_str(&format!("Bearer {}", token.trim())).map_err(|_| {
        FetchError::FetchFailed {
            reason: "invalid GitHub authentication token".to_string(),
        }
    })?;
    header.set_sensitive(true);
    Ok(header)
}

fn github_cli_path() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let working_dir = std::env::current_dir().ok()?.canonicalize().ok()?;
    let workspace_root = working_dir
        .ancestors()
        .find(|directory| directory.join(".git").exists());
    github_cli_path_from(&path, workspace_root)
}

fn github_cli_path_from(path: &OsStr, workspace_root: Option<&Path>) -> Option<PathBuf> {
    std::env::split_paths(path)
        .filter(|directory| directory.is_absolute())
        .filter_map(|directory| {
            directory
                .join(if cfg!(windows) { "gh.exe" } else { "gh" })
                .canonicalize()
                .ok()
        })
        .find(|executable| {
            executable.is_file()
                && workspace_root.is_none_or(|root| !executable.starts_with(root))
                && is_executable(executable)
        })
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

async fn authorization_from_command(
    mut command: Command,
    timeout: Duration,
) -> Result<Option<HeaderValue>, FetchError> {
    command
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let output = tokio::time::timeout(timeout, command.output())
        .await
        .map_err(|_| FetchError::FetchFailed {
            reason: "GitHub CLI authentication timed out".to_string(),
        })?
        .map_err(|_| FetchError::FetchFailed {
            reason: "failed to execute GitHub CLI authentication".to_string(),
        })?;
    if !output.status.success() {
        return Ok(None);
    }
    let token = std::str::from_utf8(&output.stdout).map_err(|_| FetchError::FetchFailed {
        reason: "invalid GitHub CLI authentication output".to_string(),
    })?;
    if token.trim().is_empty() {
        return Ok(None);
    }
    authorization_header(token).map(Some)
}

#[cfg(test)]
mod tests;
