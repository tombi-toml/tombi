use crate::ast::{
    AstNode, AstToken, ExtrasList, MarkerExpr, PackageName, Root, UrlSpec, VersionSpecNode,
};
use crate::SyntaxKind;
use tombi_rg_tree::TokenAtOffset;
use tombi_text::{Position, Range};

/// Represents the context for completion at a specific position in PEP 508 requirement
#[derive(Debug, Clone)]
pub enum CompletionContext {
    /// Empty input - suggest package names
    Empty,

    /// After package name - suggest operators, brackets, semicolon
    AfterPackageName {
        name: PackageName,
        name_range: Range,
    },

    /// Inside extras list
    InExtras {
        name: PackageName,
        extras_list: Option<ExtrasList>,
        after_comma: bool,
        bracket_range: Range,
    },

    /// After @ symbol - expecting URL
    AfterAt { name: PackageName, at_range: Range },

    /// In URL specification
    InUrl {
        name: PackageName,
        url_spec: Option<UrlSpec>,
        url_range: Range,
    },

    /// After version operator or in version specification
    InVersionSpec {
        name: PackageName,
        version_spec: Option<VersionSpecNode>,
        cursor_range: Range,
    },

    /// After semicolon - expecting marker expression
    AfterSemicolon {
        name: PackageName,
        semicolon_range: Range,
    },

    /// In marker expression
    InMarkerExpression {
        name: PackageName,
        marker: Option<MarkerExpr>,
        cursor_range: Range,
    },
}

impl CompletionContext {
    /// Extract completion context from AST at given position
    pub fn from_ast(root: &Root, position: Position) -> Option<Self> {
        // Find the token at the cursor position
        let token_at_cursor = root.syntax().token_at_position(position);

        // Find the requirement node if it exists
        let requirement = root.requirement();

        // If no requirement node exists, we're at the beginning
        if requirement.is_none() {
            return Some(CompletionContext::Empty);
        }

        let requirement = requirement.unwrap();

        // Check if we have a package name
        let package_name = requirement.package_name();

        if package_name.is_none() {
            return Some(CompletionContext::Empty);
        }

        let package_name = package_name.unwrap();
        let name_range = package_name.syntax().range();

        // Check position relative to various syntax elements

        // Check if we're after the package name but before any other element
        if position > name_range.end {
            return Some(CompletionContext::AfterPackageName {
                name: package_name.clone(),
                name_range,
            });
        }

        // Check if we're in extras
        if let Some(extras_list) = requirement.extras_list() {
            let extras_range = extras_list.syntax().range();
            if extras_range.contains(position) {
                // Check if we're after a comma
                let after_comma = match token_at_cursor {
                    TokenAtOffset::Single(token) | TokenAtOffset::Between(token, _) => {
                        // Look for comma before cursor
                        let mut current = Some(token);
                        let mut found_comma = false;
                        while let Some(t) = current {
                            if t.kind() == SyntaxKind::COMMA {
                                found_comma = true;
                                break;
                            }
                            if t.kind() == SyntaxKind::BRACKET_START {
                                break;
                            }
                            current = t.prev_token();
                        }
                        found_comma
                    }
                    TokenAtOffset::None => false,
                };

                return Some(CompletionContext::InExtras {
                    name: package_name,
                    extras_list: Some(extras_list),
                    after_comma,
                    bracket_range: extras_range,
                });
            }
        }

        // Check if we're after @ symbol
        if let Some(url_spec) = requirement.url_spec() {
            let url_range = url_spec.syntax().range();

            // Check if cursor is right after @
            // Find @ token in url_spec
            let at_token = url_spec
                .syntax()
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|t| t.kind() == SyntaxKind::AT);

            if let Some(at_token) = at_token {
                let at_range = at_token.range();
                if position >= at_range.end {
                    return Some(CompletionContext::AfterAt {
                        name: package_name,
                        at_range,
                    });
                }
            }

            if url_range.contains(position) {
                return Some(CompletionContext::InUrl {
                    name: package_name,
                    url_spec: Some(url_spec),
                    url_range,
                });
            }
        }

        // Check if we're in version specification
        if let Some(version_spec) = requirement.version_spec() {
            let version_range = version_spec.syntax().range();
            if version_range.contains(position)
                || (position >= name_range.end && position <= version_range.start)
            {
                return Some(CompletionContext::InVersionSpec {
                    name: package_name,
                    version_spec: Some(version_spec),
                    cursor_range: version_range,
                });
            }
        }

        // Check if we're after semicolon
        if let Some(marker) = requirement.marker() {
            let marker_range = marker.syntax().range();

            // Find semicolon token
            let semicolon = requirement
                .syntax()
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|t| t.kind() == SyntaxKind::SEMICOLON);

            if let Some(semicolon) = semicolon {
                let semicolon_range = semicolon.range();
                if position >= semicolon_range.end {
                    return Some(CompletionContext::AfterSemicolon {
                        name: package_name,
                        semicolon_range,
                    });
                }
            }

            if marker_range.contains(position) {
                return Some(CompletionContext::InMarkerExpression {
                    name: package_name,
                    marker: Some(marker),
                    cursor_range: marker_range,
                });
            }
        }

        // Default case - after package name
        Some(CompletionContext::AfterPackageName {
            name: package_name,
            name_range,
        })
    }

    /// Get the package name from any context
    pub fn package_name(&self) -> Option<String> {
        match self {
            CompletionContext::Empty => None,
            CompletionContext::AfterPackageName { name, .. }
            | CompletionContext::InExtras { name, .. }
            | CompletionContext::AfterAt { name, .. }
            | CompletionContext::InUrl { name, .. }
            | CompletionContext::InVersionSpec { name, .. }
            | CompletionContext::AfterSemicolon { name, .. }
            | CompletionContext::InMarkerExpression { name, .. } => Some(name.text()),
        }
    }

    /// Get existing extras if in extras context
    pub fn existing_extras(&self) -> Vec<String> {
        if let CompletionContext::InExtras {
            extras_list: Some(extras),
            ..
        } = self
        {
            extras.extras().collect()
        } else {
            Vec::new()
        }
    }
}

/// Trait for getting text from AST nodes
trait AstText {
    fn text(&self) -> String;
}

impl AstText for PackageName {
    fn text(&self) -> String {
        self.identifier()
            .map(|id| id.syntax().text().to_string())
            .unwrap_or_default()
    }
}
