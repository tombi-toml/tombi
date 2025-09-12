#[cfg(feature = "jsonschema")]
use std::borrow::Cow;
use std::str::FromStr;

use tombi_uri::SchemaUri;

use crate::value::{
    ArrayLintRules, ErrorRuleOptions, SortOptions, TombiValueDirectiveContent, WarnRuleOptions,
    WithCommonExtensibleLintRules, WithCommonLintRules, WithKeyLintRules,
};
use crate::TombiCommentDirectiveImpl;

pub type KeyTableCommonLintRules = WithKeyLintRules<WithCommonLintRules<TableLintRules>>;

pub type KeyArrayOfTableCommonLintRules =
    WithKeyLintRules<WithCommonLintRules<ArrayOfTableLintRules>>;

pub type KeyInlineTableCommonLintRules =
    WithKeyLintRules<WithCommonLintRules<InlineTableLintRules>>;

pub type TableCommonLintRules = WithCommonLintRules<TableLintRules>;

pub type ArrayOfTableCommonLintRules = WithCommonLintRules<ArrayOfTableLintRules>;

pub type InlineTableCommonLintRules = WithCommonLintRules<InlineTableLintRules>;

pub type ParentTableCommonLintRules = WithCommonExtensibleLintRules<TableLintRules>;

pub type RootTableCommonLintRules = WithCommonLintRules<RootTableLintRules>;

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<TableFormatRules, KeyTableCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<TableFormatRules, TableCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<TableFormatRules, KeyArrayOfTableCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-array-of-table-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<TableFormatRules, ArrayOfTableCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-array-of-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<TableFormatRules, KeyInlineTableCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-inline-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<TableFormatRules, InlineTableCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-inline-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<TableFormatRules, ParentTableCommonLintRules>
{
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-parent-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl
    for TombiValueDirectiveContent<TableFormatRules, RootTableCommonLintRules>
{
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
