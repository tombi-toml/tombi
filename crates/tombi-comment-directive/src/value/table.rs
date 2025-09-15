#[cfg(feature = "jsonschema")]
use std::borrow::Cow;
use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    ArrayLintRules, ErrorRuleOptions, SortOptions, TombiValueDirectiveContent, WarnRuleOptions,
    WithCommonExtensibleLintRules, WithCommonFormatRules, WithCommonLintRules, WithKeyFormatRules,
    WithKeyLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type TableCommonFormatRules = WithCommonFormatRules<TableFormatRules>;
pub type TableCommonLintRules = WithCommonLintRules<TableLintRules>;

pub type ArrayOfTableCommonFormatRules = WithCommonFormatRules<TableCommonFormatRules>;
pub type ArrayOfTableCommonLintRules = WithCommonLintRules<ArrayOfTableLintRules>;

pub type InlineTableCommonFormatRules = TableCommonFormatRules;
pub type InlineTableCommonLintRules = WithCommonLintRules<InlineTableLintRules>;

pub type ParentTableCommonFormatRules = TableCommonFormatRules;
pub type ParentTableCommonLintRules = WithCommonExtensibleLintRules<TableLintRules>;

pub type RootTableCommonFormatRules = TableCommonFormatRules;
pub type RootTableCommonLintRules = WithCommonLintRules<RootTableLintRules>;

pub type KeyTableCommonFormatRules = WithKeyFormatRules<TableCommonFormatRules>;
pub type KeyTableCommonLintRules = WithKeyLintRules<WithCommonLintRules<TableLintRules>>;

pub type KeyArrayOfTableCommonFormatRules = WithKeyFormatRules<ArrayOfTableCommonFormatRules>;
pub type KeyArrayOfTableCommonLintRules = WithKeyLintRules<ArrayOfTableCommonLintRules>;

pub type KeyInlineTableCommonFormatRules = KeyTableCommonFormatRules;
pub type KeyInlineTableCommonLintRules = WithKeyLintRules<InlineTableCommonLintRules>;

pub type KeyParentTableCommonFormatRules = KeyTableCommonFormatRules;
pub type KeyParentTableCommonLintRules = WithKeyLintRules<ParentTableCommonLintRules>;

pub type KeyRootTableCommonFormatRules = KeyTableCommonFormatRules;
pub type KeyRootTableCommonLintRules = WithKeyLintRules<RootTableCommonLintRules>;

pub type TombiTableDirectiveContent =
    TombiValueDirectiveContent<TableCommonFormatRules, TableCommonLintRules>;

pub type TombiArrayOfTableDirectiveContent =
    TombiValueDirectiveContent<ArrayOfTableCommonFormatRules, ArrayOfTableCommonLintRules>;

pub type TombiInlineTableDirectiveContent =
    TombiValueDirectiveContent<InlineTableCommonFormatRules, InlineTableCommonLintRules>;

pub type TombiParentTableDirectiveContent =
    TombiValueDirectiveContent<ParentTableCommonFormatRules, ParentTableCommonLintRules>;

pub type TombiRootTableDirectiveContent =
    TombiValueDirectiveContent<RootTableCommonFormatRules, RootTableCommonLintRules>;

pub type TombiKeyTableDirectiveContent =
    TombiValueDirectiveContent<KeyTableCommonFormatRules, KeyTableCommonLintRules>;

pub type TombiKeyArrayOfTableDirectiveContent =
    TombiValueDirectiveContent<KeyArrayOfTableCommonFormatRules, KeyArrayOfTableCommonLintRules>;

pub type TombiKeyInlineTableDirectiveContent =
    TombiValueDirectiveContent<KeyInlineTableCommonFormatRules, KeyInlineTableCommonLintRules>;

impl TombiCommentDirectiveImpl for TombiKeyTableDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiTableDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyArrayOfTableDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-array-of-table-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiArrayOfTableDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-array-of-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiKeyInlineTableDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-inline-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiInlineTableDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-inline-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiParentTableDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-parent-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiRootTableDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-root-table-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct TableFormatRules {
    /// # Table keys order
    ///
    /// Control the sorting method of the table by keys.
    ///
    pub table_keys_order: Option<SortOptions>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct TableLintRules {
    /// # Dotted keys out of order
    ///
    /// Check if dotted keys are defined out of order.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// apple.type = "fruit"
    /// orange.type = "fruit"
    /// apple.skin = "thin"
    /// orange.skin = "thick"
    ///
    /// # RECOMMENDED
    /// apple.type = "fruit"
    /// apple.skin = "thin"
    /// orange.type = "fruit"
    /// orange.skin = "thick"
    /// ```
    pub dotted_keys_out_of_order: Option<WarnRuleOptions>,

    /// # Max keys
    ///
    /// Check if the table has more than the maximum number of keys.
    ///
    /// ```rust
    /// length(table) <= maximum
    /// ```
    ///
    pub table_max_keys: Option<ErrorRuleOptions>,

    /// # Min keys
    ///
    /// Check if the table has less than the minimum number of keys.
    ///
    /// ```rust
    /// length(table) >= minimum
    /// ```
    ///
    pub table_min_keys: Option<ErrorRuleOptions>,

    /// # Key required
    ///
    /// Check if the key is required in this Table.
    ///
    pub table_key_required: Option<ErrorRuleOptions>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ArrayOfTableLintRules {
    #[serde(flatten)]
    pub array: ArrayLintRules,

    #[serde(flatten)]
    pub table: TableLintRules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InlineTableLintRules(pub TableLintRules);

#[cfg(feature = "jsonschema")]
impl schemars::JsonSchema for InlineTableLintRules {
    fn schema_name() -> Cow<'static, str> {
        "InlineTableLintRules".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        TableLintRules::json_schema(generator)
    }

    fn inline_schema() -> bool {
        true
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct RootTableLintRules {
    /// # Tables out of order
    ///
    /// Check if tables are defined out of order.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// [fruit.apple]
    /// [animal]
    /// [fruit.orange]
    ///
    /// # RECOMMENDED
    /// [fruit.apple]
    /// [fruit.orange]
    /// [animal]
    /// ```
    pub tables_out_of_order: Option<WarnRuleOptions>,

    #[serde(flatten)]
    pub table: TableLintRules,
}
