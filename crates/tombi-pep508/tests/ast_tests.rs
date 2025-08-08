use tombi_pep508::{parse, SyntaxKind};

#[test]
fn test_parse_simple_package() {
    let (root, errors) = parse("requests");
    assert!(errors.is_empty());
    assert_eq!(root.kind(), SyntaxKind::ROOT);

    let requirement = root.first_child().unwrap();
    assert_eq!(requirement.kind(), SyntaxKind::REQUIREMENT);

    let package_name = requirement.first_child().unwrap();
    assert_eq!(package_name.kind(), SyntaxKind::PACKAGE_NAME);
}

#[test]
fn test_parse_with_version() {
    let (root, errors) = parse("requests>=2.28.0");
    assert!(errors.is_empty());
    assert_eq!(root.kind(), SyntaxKind::ROOT);

    let requirement = root.first_child().unwrap();
    assert_eq!(requirement.kind(), SyntaxKind::REQUIREMENT);

    // Should have package name and version spec
    let mut children = requirement.children();
    let package_name = children.next().unwrap();
    assert_eq!(package_name.kind(), SyntaxKind::PACKAGE_NAME);

    let version_spec = children.next().unwrap();
    assert_eq!(version_spec.kind(), SyntaxKind::VERSION_SPEC);

    // Version spec should have a version clause
    let version_clause = version_spec.first_child().unwrap();
    assert_eq!(version_clause.kind(), SyntaxKind::VERSION_CLAUSE);
}

#[test]
fn test_parse_with_extras() {
    let (root, errors) = parse("requests[security,socks]");
    assert!(errors.is_empty());
    assert_eq!(root.kind(), SyntaxKind::ROOT);

    let requirement = root.first_child().unwrap();
    assert_eq!(requirement.kind(), SyntaxKind::REQUIREMENT);

    // Should have package name and extras list
    let mut children = requirement.children();
    let package_name = children.next().unwrap();
    assert_eq!(package_name.kind(), SyntaxKind::PACKAGE_NAME);

    let extras_list = children.next().unwrap();
    assert_eq!(extras_list.kind(), SyntaxKind::EXTRAS_LIST);
}

#[test]
fn test_parse_with_marker() {
    let (root, errors) = parse("requests ; python_version >= '3.8'");
    assert!(errors.is_empty());
    assert_eq!(root.kind(), SyntaxKind::ROOT);

    let requirement = root.first_child().unwrap();
    assert_eq!(requirement.kind(), SyntaxKind::REQUIREMENT);

    // Should have package name and marker
    let mut found_marker = false;
    for child in requirement.children() {
        if child.kind() == SyntaxKind::MARKER_EXPR {
            found_marker = true;
            break;
        }
    }
    assert!(found_marker);
}

#[test]
fn test_parse_complex() {
    let (root, errors) = parse("requests[security,socks]>=2.28.0 ; python_version >= '3.8'");
    assert!(errors.is_empty());
    assert_eq!(root.kind(), SyntaxKind::ROOT);

    let requirement = root.first_child().unwrap();
    assert_eq!(requirement.kind(), SyntaxKind::REQUIREMENT);

    // Should have all components
    let mut has_package = false;
    let mut has_extras = false;
    let mut has_version = false;
    let mut has_marker = false;

    for child in requirement.children() {
        match child.kind() {
            SyntaxKind::PACKAGE_NAME => has_package = true,
            SyntaxKind::EXTRAS_LIST => has_extras = true,
            SyntaxKind::VERSION_SPEC => has_version = true,
            SyntaxKind::MARKER_EXPR => has_marker = true,
            _ => {}
        }
    }

    assert!(has_package);
    assert!(has_extras);
    assert!(has_version);
    assert!(has_marker);
}

#[test]
fn test_parse_url_dependency() {
    let (root, errors) = parse("mypackage @ https://github.com/user/repo/archive/main.zip");
    assert!(errors.is_empty());
    assert_eq!(root.kind(), SyntaxKind::ROOT);

    let requirement = root.first_child().unwrap();
    assert_eq!(requirement.kind(), SyntaxKind::REQUIREMENT);

    // Should have package name and URL
    let mut has_package = false;
    let mut has_url = false;

    for child in requirement.children() {
        match child.kind() {
            SyntaxKind::PACKAGE_NAME => has_package = true,
            SyntaxKind::URL_SPEC => has_url = true,
            _ => {}
        }
    }

    assert!(has_package);
    assert!(has_url);
}

#[test]
fn test_parse_incomplete_extras() {
    let (root, errors) = parse("requests[security,");
    // Should have an error for incomplete extras
    assert!(!errors.is_empty());
    assert_eq!(root.kind(), SyntaxKind::ROOT);
}

#[test]
fn test_parse_multiple_version_clauses() {
    let (root, errors) = parse("package>=1.0,<2.0");
    assert!(errors.is_empty());
    assert_eq!(root.kind(), SyntaxKind::ROOT);

    let requirement = root.first_child().unwrap();
    let version_spec = requirement
        .children()
        .find(|n| n.kind() == SyntaxKind::VERSION_SPEC)
        .unwrap();

    // Should have two version clauses
    let clauses: Vec<_> = version_spec
        .children()
        .filter(|n| n.kind() == SyntaxKind::VERSION_CLAUSE)
        .collect();
    assert_eq!(clauses.len(), 2);
}