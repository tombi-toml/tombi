use std::path::PathBuf;
use thiserror::Error;

/// File system operation errors
#[derive(Debug, Error)]
pub enum Error {
    /// File not found
    #[error("File not found: {path}")]
    NotFound { path: PathBuf },

    /// Permission denied
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    /// I/O error
    #[error("I/O error on {path}: {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// WASM-specific error
    #[cfg(feature = "wasm")]
    #[error("WASM error: {message}")]
    WasmError { message: String },

    /// Sandbox violation (WASM)
    #[cfg(feature = "wasm")]
    #[error("Sandbox violation: attempted to access {path}")]
    SandboxViolation { path: PathBuf },

    /// Unimplemented feature
    #[error("Unimplemented feature: {feature}")]
    Unimplemented { feature: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_not_found_error() {
        let path = PathBuf::from("/test/file.txt");
        let error = Error::NotFound { path: path.clone() };
        assert_eq!(
            error.to_string(),
            format!("File not found: {}", path.display())
        );
    }

    #[test]
    fn test_permission_denied_error() {
        let path = PathBuf::from("/test/file.txt");
        let error = Error::PermissionDenied { path: path.clone() };
        assert_eq!(
            error.to_string(),
            format!("Permission denied: {}", path.display())
        );
    }

    #[test]
    fn test_io_error_conversion() {
        let path = PathBuf::from("/test/file.txt");
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = Error::IoError {
            path: path.clone(),
            source: io_error,
        };

        // Check that error message contains both path and source
        let msg = error.to_string();
        assert!(msg.contains(&path.display().to_string()));
        assert!(msg.contains("I/O error"));
    }

    #[test]
    fn test_unimplemented_error() {
        let error = Error::Unimplemented {
            feature: "streaming",
        };
        assert_eq!(error.to_string(), "Unimplemented feature: streaming");
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_wasm_error() {
        let error = Error::WasmError {
            message: "Browser API restriction".to_string(),
        };
        assert_eq!(error.to_string(), "WASM error: Browser API restriction");
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_sandbox_violation_error() {
        let path = PathBuf::from("/etc/passwd");
        let error = Error::SandboxViolation { path: path.clone() };
        assert_eq!(
            error.to_string(),
            format!("Sandbox violation: attempted to access {}", path.display())
        );
    }
}
