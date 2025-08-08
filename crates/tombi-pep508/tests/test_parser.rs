use tombi_pep508::{Parser, PartialParseResult, VersionOperator};

#[test]
fn test_parse_simple_package() {
    let mut parser = Parser::new("requests");
    let req = parser.parse().unwrap();
    assert_eq!(req.name, "requests");
    assert!(req.extras.is_empty());
    assert!(req.version_spec.is_none());
    assert!(req.marker.is_none());
}

#[test]
fn test_parse_with_version() {
    let mut parser = Parser::new("requests>=2.28.0");
    let req = parser.parse().unwrap();
    assert_eq!(req.name, "requests");
    assert!(req.version_spec.is_some());
    let spec = req.version_spec.unwrap();
    assert_eq!(spec.clauses.len(), 1);
    assert_eq!(spec.clauses[0].operator, VersionOperator::GreaterThanEqual);
    assert_eq!(spec.clauses[0].version, "2.28.0");
}

#[test]
fn test_parse_with_extras() {
    let mut parser = Parser::new("requests[security,socks]");
    let req = parser.parse().unwrap();
    assert_eq!(req.name, "requests");
    assert_eq!(req.extras, vec!["security", "socks"]);
}

#[test]
fn test_parse_with_marker() {
    let mut parser = Parser::new("requests ; python_version >= '3.8'");
    let req = parser.parse().unwrap();
    assert_eq!(req.name, "requests");
    assert!(req.marker.is_some());
}

#[test]
fn test_parse_url_dependency() {
    let mut parser = Parser::new("mypackage @ https://github.com/user/repo/archive/main.zip");
    let req = parser.parse().unwrap();
    assert_eq!(req.name, "mypackage");
    assert_eq!(req.url, Some("https://github.com/user/repo/archive/main.zip".to_string()));
}

#[test]
fn test_partial_parse_incomplete_extras() {
    let mut parser = Parser::new("requests[security,");
    let result = parser.parse_partial();
    match result {
        PartialParseResult::ExtrasIncomplete { name, extras, after_comma } => {
            assert_eq!(name, "requests");
            assert_eq!(extras, vec!["security"]);
            assert!(after_comma);
        }
        _ => panic!("Expected ExtrasIncomplete"),
    }
}

#[test]
fn test_partial_parse_after_at() {
    let mut parser = Parser::new("mypackage @ ");
    let result = parser.parse_partial();
    match result {
        PartialParseResult::AfterAt { name } => {
            assert_eq!(name, "mypackage");
        }
        _ => panic!("Expected AfterAt"),
    }
}

#[test]
fn test_partial_parse_after_semicolon() {
    let mut parser = Parser::new("requests ; ");
    let result = parser.parse_partial();
    match result {
        PartialParseResult::AfterSemicolon { name, version_spec } => {
            assert_eq!(name, "requests");
            assert!(version_spec.is_none());
        }
        _ => panic!("Expected AfterSemicolon"),
    }
}

#[test]
fn test_parse_complex_version_spec() {
    let mut parser = Parser::new("django>=3.2,<4.0");
    let req = parser.parse().unwrap();
    assert_eq!(req.name, "django");
    assert!(req.version_spec.is_some());
    let spec = req.version_spec.unwrap();
    assert_eq!(spec.clauses.len(), 2);
    assert_eq!(spec.clauses[0].operator, VersionOperator::GreaterThanEqual);
    assert_eq!(spec.clauses[0].version, "3.2");
    assert_eq!(spec.clauses[1].operator, VersionOperator::LessThan);
    assert_eq!(spec.clauses[1].version, "4.0");
}

#[test]
fn test_parse_with_trailing_comma_in_extras() {
    let mut parser = Parser::new("requests[security,]");
    let req = parser.parse().unwrap();
    assert_eq!(req.name, "requests");
    assert_eq!(req.extras, vec!["security"]);
}