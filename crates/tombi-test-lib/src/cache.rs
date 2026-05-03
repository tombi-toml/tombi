use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, OnceLock},
};

use tempfile::TempDir;

fn cache_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub struct TestCacheHome {
    _guard: MutexGuard<'static, ()>,
    previous_tombi: Option<OsString>,
    previous_xdg: Option<OsString>,
    temp_dir: TempDir,
}

impl TestCacheHome {
    pub fn new() -> Self {
        Self::with_tombi_cache_home(None)
    }

    pub fn with_tombi_cache_home(tombi_cache_home: Option<&Path>) -> Self {
        let guard = cache_env_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let previous_tombi = std::env::var_os("TOMBI_CACHE_HOME");
        let previous_xdg = std::env::var_os("XDG_CACHE_HOME");
        // SAFETY: Tests serialize access with a process-wide mutex so env mutation
        // remains scoped to one test at a time.
        unsafe {
            if let Some(tombi_cache_home) = tombi_cache_home {
                std::env::set_var("TOMBI_CACHE_HOME", tombi_cache_home);
            } else {
                std::env::remove_var("TOMBI_CACHE_HOME");
            }
            std::env::set_var("XDG_CACHE_HOME", temp_dir.path());
        }
        Self {
            _guard: guard,
            previous_tombi,
            previous_xdg,
            temp_dir,
        }
    }

    pub fn xdg_cache_home_path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn tombi_cache_dir_path(&self) -> PathBuf {
        self.temp_dir.path().join("tombi")
    }
}

impl Default for TestCacheHome {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestCacheHome {
    fn drop(&mut self) {
        // SAFETY: Tests serialize access with a process-wide mutex so env mutation
        // remains scoped to one test at a time.
        unsafe {
            if let Some(previous) = &self.previous_tombi {
                std::env::set_var("TOMBI_CACHE_HOME", previous);
            } else {
                std::env::remove_var("TOMBI_CACHE_HOME");
            }

            if let Some(previous) = &self.previous_xdg {
                std::env::set_var("XDG_CACHE_HOME", previous);
            } else {
                std::env::remove_var("XDG_CACHE_HOME");
            }
        }
    }
}
