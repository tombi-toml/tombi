//! Platform-agnostic file system abstraction for Tombi
//!
//! This crate provides a unified file system interface that works across
//! native (macOS, Linux, Windows) and WASM environments.
//!
//! # Feature Flags
//!
//! - `native`: Use native file system implementation (tokio::fs)
//! - `wasm`: Use WASM file system implementation (WASI)
//!
//! These flags are mutually exclusive. Attempting to enable both will result
//! in a compile error.

// Ensure exactly one feature is enabled
#[cfg(all(feature = "native", feature = "wasm"))]
compile_error!("Features 'native' and 'wasm' cannot be enabled simultaneously");

#[cfg(not(any(feature = "native", feature = "wasm")))]
compile_error!("Either 'native' or 'wasm' feature must be enabled");

mod error;
mod fs;
mod memory;
mod path;

#[cfg(feature = "native")]
mod native;

#[cfg(feature = "wasm")]
mod wasm;

pub use error::Error;
pub use fs::{FileSystem, FileType, Metadata};
pub use memory::InMemoryFileSystem;
pub use path::{AbsPath, AbsPathBuf, VfsPath};

#[cfg(feature = "native")]
pub use native::NativeFileSystem;

#[cfg(feature = "wasm")]
pub use wasm::WasmFileSystem;

#[cfg(test)]
mod tests {
    #[test]
    fn test_feature_flags() {
        // This test ensures that exactly one feature flag is active
        #[cfg(feature = "native")]
        {
            assert!(cfg!(feature = "native"));
            assert!(!cfg!(feature = "wasm"));
        }

        #[cfg(feature = "wasm")]
        {
            assert!(!cfg!(feature = "native"));
            assert!(cfg!(feature = "wasm"));
        }
    }
}
