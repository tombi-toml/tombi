#[cfg(feature = "jsonschema")]
use std::borrow::Cow;
use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    ArrayRules, ErrorRuleOptions, TombiValueDirectiveContent, WarnRuleOptions,
    WithCommonExtensibleRules, WithCommonRules, WithKeyRules,
};
use crate::TombiCommentDirectiveImpl;

pub type KeyTableCommonRules = WithKeyRules<WithCommonRules<TableRules>>;

pub type KeyArrayOfTableCommonRules = WithKeyRules<WithCommonRules<ArrayOfTableRules>>;

pub type KeyInlineTableCommonRules = WithKeyRules<WithCommonRules<InlineTableRules>>;

pub type TableCommonRules = WithCommonRules<TableRules>;

pub type ArrayOfTableCommonRules = WithCommonRules<ArrayOfTableRules>;

pub type InlineTableCommonRules = WithCommonRules<InlineTableRules>;

pub type ParentTableCommonRules = WithCommonExtensibleRules<TableRules>;

pub type RootTableCommonRules = WithCommonRules<RootTableRules>;

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<KeyTableCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<TableCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<KeyArrayOfTableCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-array-of-table-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<ArrayOfTableCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-array-of-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<KeyInlineTableCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-inline-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<InlineTableCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-inline-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<ParentTableCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-parent-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiValueDirectiveContent<RootTableCommonRules> {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-root-table-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct TableRules {
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

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ArrayOfTableRules {
    #[serde(flatten)]
    pub array: ArrayRules,

    #[serde(flatten)]
    pub table: TableRules,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InlineTableRules(pub TableRules);

#[cfg(feature = "jsonschema")]
impl schemars::JsonSchema for InlineTableRules {
    fn schema_name() -> Cow<'static, str> {
        "InlineTableRules".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        TableRules::json_schema(generator)
    }

    fn inline_schema() -> bool {
        true
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct RootTableRules {
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
    pub table: TableRules,
}
