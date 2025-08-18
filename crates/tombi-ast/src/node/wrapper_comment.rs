use crate::AstToken;

macro_rules! impl_comment {
    (
        #[derive(Debug, Clone, PartialEq, Eq, AsRef, From, Into)]
        pub struct $name:ident(crate::Comment);
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name(crate::Comment);

        impl $name {
            pub fn syntax(&self) -> &tombi_syntax::SyntaxToken {
                self.0.syntax()
            }
        }

        impl AsRef<crate::Comment> for $name {
            fn as_ref(&self) -> &crate::Comment {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = crate::Comment;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<crate::Comment> for $name {
            fn from(comment: crate::Comment) -> Self {
                $name(comment)
            }
        }

        impl From<$name> for crate::Comment {
            fn from(comment: $name) -> Self {
                comment.0
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
    pub struct BeginDanglingComment(crate::Comment);
);

impl_comment!(
    #[derive(Debug, Clone, PartialEq, Eq, AsRef, From, Into)]
    pub struct EndDanglingComment(crate::Comment);
);

impl_comment!(
    #[derive(Debug, Clone, PartialEq, Eq, AsRef, From, Into)]
    pub struct LeadingComment(crate::Comment);
);

impl_comment!(
    #[derive(Debug, Clone, PartialEq, Eq, AsRef, From, Into)]
    pub struct TrailingComment(crate::Comment);
);
