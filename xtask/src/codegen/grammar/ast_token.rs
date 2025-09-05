use convert_case::{Case, Casing};
use quote::{format_ident, quote};

use super::syntax_kind_src::TOKENS;
use crate::utils::reformat;

pub fn generate_ast_token() -> Result<String, anyhow::Error> {
    let tokens = TOKENS.iter().map(|token| {
        let name = format_ident!("{}", token.to_case(Case::Pascal));
        let kind = format_ident!("{}", token.to_case(Case::UpperSnake));
        quote! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            #[allow(dead_code)]
            pub struct #name {
                pub(crate) syntax: SyntaxToken,
            }
            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Display::fmt(&self.syntax, f)
                }
            }
            impl AstToken for #name {
                fn can_cast(kind: SyntaxKind) -> bool { kind == SyntaxKind::#kind }
                fn cast(syntax: SyntaxToken) -> Option<Self> {
                    if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
                }
                fn syntax(&self) -> &SyntaxToken { &self.syntax }
            }
        }
    });

    reformat(
        quote! {
            use crate::AstToken;
            use tombi_syntax::{SyntaxKind, SyntaxToken};
            #(#tokens)*
        }
        .to_string(),
    )
    .map(|content| content.replace("#[derive", "\n#[derive"))
}
