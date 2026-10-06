use rstest::rstest;

use super::environment_token;

#[rstest]
#[case(Some("gh-token"), Some("github-token"), Some("gh-token"))]
#[case(None, Some("github-token"), Some("github-token"))]
#[case(None, None, None)]
fn selects_environment_token_in_priority_order(
    #[case] gh_token: Option<&str>,
    #[case] github_token: Option<&str>,
    #[case] expected: Option<&str>,
) {
    let selected = environment_token(|name| match name {
        "GH_TOKEN" => gh_token.map(str::to_owned),
        "GITHUB_TOKEN" => github_token.map(str::to_owned),
        _ => panic!("unexpected credential source"),
    });
    assert_eq!(selected.as_deref(), expected);
}

#[cfg(unix)]
#[rstest]
fn skips_workspace_gh_but_keeps_user_installation() {
    use std::os::unix::fs::PermissionsExt;

    let home = tempfile::tempdir().unwrap();
    let home = home.path().canonicalize().unwrap();
    let workspace = home.join("project");
    let project_bin = workspace.join("node_modules/.bin");
    let user_bin = home.join(".local/bin");
    for directory in [&project_bin, &user_bin] {
        std::fs::create_dir_all(directory).unwrap();
        let executable = directory.join("gh");
        std::fs::write(&executable, b"test executable").unwrap();
        std::fs::set_permissions(executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = std::env::join_paths([&project_bin, &user_bin]).unwrap();
    assert_eq!(
        super::github_cli_path_from(&path, Some(&workspace)),
        Some(user_bin.join("gh"))
    );
    let path = std::env::join_paths([&user_bin]).unwrap();
    assert_eq!(
        super::github_cli_path_from(&path, None),
        Some(user_bin.join("gh"))
    );
}
