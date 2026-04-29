use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigLevel {
    Project,
    User,
    System,
    Default,
}

pub fn config_base_dir(config_path: &Path) -> Option<PathBuf> {
    let config_dir = config_path.parent()?;

    if config_path.file_name().and_then(|name| name.to_str()) == Some("tombi.toml")
        && config_dir.file_name().and_then(|name| name.to_str()) == Some(".config")
    {
        config_dir.parent().map(Path::to_path_buf)
    } else {
        Some(config_dir.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::config_base_dir;

    #[test]
    fn config_base_dir_uses_parent_for_regular_config() {
        assert_eq!(
            config_base_dir(Path::new("/tmp/project/tombi.toml")),
            Some(Path::new("/tmp/project").to_path_buf())
        );
    }

    #[test]
    fn config_base_dir_uses_grandparent_for_dot_config_tombi_toml() {
        assert_eq!(
            config_base_dir(Path::new("/tmp/project/.config/tombi.toml")),
            Some(Path::new("/tmp/project").to_path_buf())
        );
    }

    #[test]
    fn config_base_dir_keeps_dot_config_for_non_tombi_toml() {
        assert_eq!(
            config_base_dir(Path::new("/tmp/project/.config/config.toml")),
            Some(Path::new("/tmp/project/.config").to_path_buf())
        );
    }
}
