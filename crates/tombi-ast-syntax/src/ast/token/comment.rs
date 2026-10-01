use crate::AstToken;

macro_rules! impl_comment {
    (
        #[derive(Debug, Clone, PartialEq, Eq, AsRef, From, Into)]
        pub struct $name:ident(crate::Comment);
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name<'t>(crate::Comment<'t>);

        impl<'t> $name<'t> {
            pub fn syntax(&self) -> &tombi_ast_syntax::SyntaxToken<'t> {
                self.0.syntax()
            }
        }

        impl<'t> AsRef<crate::Comment<'t>> for $name<'t> {
            fn as_ref(&self) -> &crate::Comment<'t> {
                &self.0
            }
        }

        impl<'t> std::ops::Deref for $name<'t> {
            type Target = crate::Comment<'t>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<'t> From<crate::Comment<'t>> for $name<'t> {
            fn from(comment: crate::Comment<'t>) -> Self {
                $name(comment)
            }
        }

        impl<'t> From<$name<'t>> for crate::Comment<'t> {
            fn from(comment: $name<'t>) -> Self {
                comment.0
            }
        }

        impl<'t> AstToken<'t> for $name<'t> {
            #[inline]
            fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool {
                crate::Comment::can_cast(kind)
            }

            #[inline]
            fn cast(syntax: tombi_ast_syntax::SyntaxToken<'t>) -> Option<Self> {
                crate::Comment::cast(syntax).map($name::from)
            }

            #[inline]
            fn syntax(&self) -> &tombi_ast_syntax::SyntaxToken<'t> {
                self.0.syntax()
            }
        }
    };
}

impl_comment!(
    #[derive(Debug, Clone, PartialEq, Eq, AsRef, From, Into)]
    pub struct DanglingComment(crate::Comment);
);

impl_comment!(
    #[derive(Debug, Clone, PartialEq, Eq, AsRef, From, Into)]
    pub struct LeadingComment(crate::Comment);
);

impl_comment!(
    #[derive(Debug, Clone, PartialEq, Eq, AsRef, From, Into)]
    pub struct TrailingComment(crate::Comment);
);
