use convert_case::{Case, Casing};
use itertools::Itertools;
use quote::{format_ident, quote};

use super::ast_src::AstSrc;
use crate::utils::reformat;

fn is_excluded_auto_generated_method(node_name: &str, method_name: &str) -> bool {
    matches!(
        (node_name, method_name),
        ("Array", "values")
            | ("Root", "key_values")
            | ("Table", "key_values")
            | ("ArrayOfTable", "key_values")
            | ("InlineTable", "key_values")
    )
}

pub fn generate_ast_node(ast: &AstSrc) -> Result<String, anyhow::Error> {
    let (node_defs, node_boilerplate_impls): (Vec<_>, Vec<_>) = ast
        .nodes
        .iter()
        .map(|node| {
            let name = format_ident!("{}", node.name);
            let kind = format_ident!("{}", node.name.to_case(Case::UpperSnake));
            let traits = node.traits.iter().map(|trait_name| {
                let trait_name = format_ident!("{}", trait_name);
                quote!(impl tombi_ast::#trait_name for #name {})
            });

            let methods = node.fields.iter().filter_map(|field| {
                let method_name_str = field.method_name();
                if is_excluded_auto_generated_method(&node.name, &method_name_str) {
                    return None;
                }
                let method_name = format_ident!("{}", method_name_str);
                let ty = field.ty();

                if field.is_many() {
                    Some(quote! {
                        #[inline]
                        pub fn #method_name(&self) -> AstChildren<#ty> {
                            support::node::children(&self.syntax)
                        }
                    })
                } else if let Some(token_kind) = field.token_kind() {
                    Some(quote! {
                        #[inline]
                        pub fn #method_name(&self) -> Option<#ty> {
                            support::node::token(&self.syntax, #token_kind)
                        }
                    })
                } else {
                    Some(quote! {
                        #[inline]
                        pub fn #method_name(&self) -> Option<#ty> {
                            support::node::child(&self.syntax)
                        }
                    })
                }
            });
            (
                quote! {
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub struct #name {
                        pub(crate) syntax: SyntaxNode,
                    }

                    #(#traits)*

                    impl #name {
                        #(#methods)*

                        #[inline]
                        pub fn range(&self) -> tombi_text::Range {
                            self.syntax.range()
                        }
                    }
                },
                quote! {
                    impl AstNode for #name {
                        #[inline]
                        fn can_cast(kind: SyntaxKind) -> bool {
                            kind == SyntaxKind::#kind
                        }
                        #[inline]
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
                        }
                        #[inline]
                        fn syntax(&self) -> &SyntaxNode { &self.syntax }
                    }
                },
            )
        })
        .unzip();

    let (enum_defs, enum_boilerplate_impls): (Vec<_>, Vec<_>) = ast
        .enums
        .iter()
        .map(|en| {
            let variants: Vec<_> = en
                .variants
                .iter()
                .map(|var| format_ident!("{}", var))
                .sorted()
                .collect();
            let name = format_ident!("{}", en.name);
            let kinds: Vec<_> = variants
                .iter()
                .map(|name| format_ident!("{}", name.to_string().to_case(Case::UpperSnake)))
                .collect();
            let traits = en.traits.iter().sorted().map(|trait_name| {
                let trait_name = format_ident!("{}", trait_name);
                quote!(impl tombi_ast::#trait_name for #name {})
            });

            let ast_node = quote! {
                impl AstNode for #name {
                    #[inline]
                    fn can_cast(kind: SyntaxKind) -> bool {
                        matches!(kind, #(SyntaxKind::#kinds)|*)
                    }
                    #[inline]
                    fn cast(syntax: SyntaxNode) -> Option<Self> {
                        let res = match syntax.kind() {
                            #(
                            SyntaxKind::#kinds => #name::#variants(#variants { syntax }),
                            )*
                            _ => return None,
                        };
                        Some(res)
                    }
                    #[inline]
                    fn syntax(&self) -> &SyntaxNode {
                        match self {
                            #(
                            #name::#variants(it) => &it.syntax,
                            )*
                        }
                    }
                }
            };

            (
                quote! {
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub enum #name {
                        #(#variants(#variants),)*
                    }

                    #(#traits)*
                },
                quote! {
                    #(
                        impl From<#variants> for #name {
                            #[inline]
                            fn from(node: #variants) -> #name {
                                #name::#variants(node)
                            }
                        }
                    )*
                    #ast_node
                },
            )
        })
        .unzip();

    let (any_node_defs, any_node_boilerplate_impls): (Vec<_>, Vec<_>) = ast
        .nodes
        .iter()
        .flat_map(|node| node.traits.iter().map(move |t: &String| (t, node)))
        .into_group_map()
        .into_iter()
        .sorted_by_key(|(name, _)| *name)
        .map(|(trait_name, nodes)| {
            let name = format_ident!("Any{}", trait_name);
            let trait_name = format_ident!("{}", trait_name);
            let kinds: Vec<_> = nodes
                .iter()
                .map(|name| format_ident!("{}", &name.name.to_string().to_case(Case::UpperSnake)))
                .collect();
            let nodes = nodes.iter().map(|node| format_ident!("{}", node.name));
            (
                quote! {
                    #[pretty_doc_comment_placeholder_workaround]
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub struct #name {
                        pub(crate) syntax: SyntaxNode,
                    }
                    impl tombi_ast::#trait_name for #name {}
                },
                quote! {
                    impl #name {
                        #[inline]
                        pub fn new<T: tombi_ast::#trait_name>(node: T) -> #name {
                            #name {
                                syntax: node.syntax().clone()
                            }
                        }
                    }
                    impl AstNode for #name {
                        #[inline]
                        fn can_cast(kind: SyntaxKind) -> bool {
                            matches!(kind, #(#kinds)|*)
                        }
                        #[inline]
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            Self::can_cast(syntax.kind()).then_some(#name { syntax })
                        }
                        #[inline]
                        fn syntax(&self) -> &SyntaxNode {
                            &self.syntax
                        }
                    }

                    #(
                        impl From<#nodes> for #name {
                            #[inline]
                            fn from(node: #nodes) -> #name {
                                #name { syntax: node.syntax }
                            }
                        }
                    )*
                },
            )
        })
        .unzip();

    let enum_names = ast.enums.iter().map(|it| &it.name);
    let node_names = ast.nodes.iter().map(|it| &it.name);

    let display_impls = enum_names
        .chain(node_names.clone())
        .map(|it| format_ident!("{}", it))
        .map(|name| {
            quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        std::fmt::Display::fmt(self.syntax(), f)
                    }
                }
            }
        });

    reformat(
        quote! {
            use crate::AstNode;
            use tombi_syntax::{SyntaxKind, SyntaxKind::*, SyntaxNode, SyntaxToken, T};
            use crate::support;
            use crate::AstChildren;

            #(#node_defs)*
            #(#enum_defs)*
            #(#any_node_defs)*
            #(#node_boilerplate_impls)*
            #(#enum_boilerplate_impls)*
            #(#any_node_boilerplate_impls)*
            #(#display_impls)*
        }
        .to_string(),
    )
    .map(|content| content.replace("#[derive", "\n#[derive"))
}
