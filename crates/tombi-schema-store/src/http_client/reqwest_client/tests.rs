use reqwest::{StatusCode, Url};
use rstest::rstest;

use super::should_retry_with_github_auth;

#[rstest]
#[case::private_raw(
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    404,
    true
)]
#[case::anonymous_rate_limit(
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    403,
    true
)]
#[case::other_host(
    "https://example.com/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    404,
    false
)]
#[case::redirected_elsewhere(
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://example.com/schema.json",
    404,
    false
)]
#[case::insecure_origin(
    "http://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    404,
    false
)]
#[case::server_error(
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    "https://raw.githubusercontent.com/owner/repo/main/schema.json",
    500,
    false
)]
fn only_retry_github_raw_authentication_failures(
    #[case] original: &str,
    #[case] response: &str,
    #[case] status: u16,
    #[case] expected: bool,
) {
    assert_eq!(
        should_retry_with_github_auth(
            &Url::parse(original).unwrap(),
            &Url::parse(response).unwrap(),
            StatusCode::from_u16(status).unwrap(),
        ),
        expected,
    );
}
