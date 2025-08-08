use tombi_pep508::{AstNode, Root};

#[test]
fn test_ast_node_navigation() {
    let input = "requests[security,socks]>=2.28.0 ; python_version >= '3.8'";
    let (syntax_node, errors) = tombi_pep508::parse(input);

    assert_eq!(errors.len(), 0);

    // Cast to Root node
    let root = Root::cast(syntax_node).expect("Should be a Root node");

    // Get requirement
    let requirement = root.requirement().expect("Should have a requirement");

    // Get package name
    let package_name = requirement
        .package_name()
        .expect("Should have a package name");
    assert_eq!(package_name.name(), Some("requests".to_string()));

    // Get extras
    let extras_list = requirement.extras_list().expect("Should have extras");
    let extras: Vec<String> = extras_list.extras().collect();
    assert_eq!(extras, vec!["security", "socks"]);

    // Get version spec
    let version_spec = requirement
        .version_spec()
        .expect("Should have version spec");
    let clauses: Vec<_> = version_spec.clauses().collect();
    assert_eq!(clauses.len(), 1);

    let clause = &clauses[0];
    assert_eq!(
        clause.operator(),
        Some(tombi_pep508::VersionOperator::GreaterThanEqual)
    );
    assert_eq!(clause.version(), Some("2.28.0".to_string()));

    // Get marker
    let marker = requirement.marker().expect("Should have marker");
    assert_eq!(marker.expression(), "python_version >= '3.8'");
}

#[test]
fn test_ast_node_simple() {
    let input = "numpy";
    let (syntax_node, errors) = tombi_pep508::parse(input);

    assert_eq!(errors.len(), 0);

    let root = Root::cast(syntax_node).expect("Should be a Root node");
    let requirement = root.requirement().expect("Should have a requirement");
    let package_name = requirement
        .package_name()
        .expect("Should have a package name");

    assert_eq!(package_name.name(), Some("numpy".to_string()));
    assert!(requirement.extras_list().is_none());
    assert!(requirement.version_spec().is_none());
    assert!(requirement.marker().is_none());
}

#[test]
fn test_ast_node_with_url() {
    let input = "package @ https://github.com/user/repo/archive/main.zip";
    let (syntax_node, errors) = tombi_pep508::parse(input);

    assert_eq!(errors.len(), 0);

    let root = Root::cast(syntax_node).expect("Should be a Root node");
    let requirement = root.requirement().expect("Should have a requirement");

    assert_eq!(
        requirement.package_name().unwrap().name(),
        Some("package".to_string())
    );
    assert!(requirement.url_spec().is_some());
    assert!(requirement.version_spec().is_none());
}

#[test]
fn test_clone_for_update() {
    let input = "requests>=2.0.0";
    let (syntax_node, errors) = tombi_pep508::parse(input);

    assert_eq!(errors.len(), 0);

    let root = Root::cast(syntax_node).expect("Should be a Root node");
    let cloned = root.clone_for_update();

    // Verify cloned node has same structure
    let requirement = cloned.requirement().expect("Should have a requirement");
    assert_eq!(
        requirement.package_name().unwrap().name(),
        Some("requests".to_string())
    );
}
