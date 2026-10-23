use tombi_schema_store::{ResolvedFormatOrder, XTombiArrayValuesOrder, XTombiTableKeysOrder};
use tombi_x_keyword::{ArrayValuesOrderBy, ArrayValuesOrderGroup, StringFormat};

use super::display_value::DisplayValue;

/// Build enum values from const_value and enum fields
///
/// This function is used to create the enum field for ValueConstraints
/// by combining const_value and enum from various schema types.
pub fn build_enum_values<T, F>(
    const_value: &Option<T>,
    r#enum: &Option<Vec<T>>,
    convert_fn: F,
) -> Option<Vec<DisplayValue>>
where
    F: Fn(&T) -> Option<DisplayValue>,
{
    let const_len = if const_value.is_some() { 1 } else { 0 };
    let enum_len = r#enum.as_ref().map(|value| value.len()).unwrap_or_default();
    let mut enum_values = Vec::with_capacity(const_len + enum_len);

    if let Some(const_value) = const_value
        && let Some(display_value) = convert_fn(const_value)
    {
        enum_values.push(display_value);
    }

    if let Some(r#enum) = r#enum {
        for value in r#enum.iter().filter_map(convert_fn) {
            if !enum_values.contains(&value) {
                enum_values.push(value);
            }
        }
    }

    if enum_values.is_empty() {
        None
    } else {
        Some(enum_values)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ValueConstraints {
    // Common
    pub r#enum: Option<Vec<DisplayValue>>,
    pub default: Option<DisplayValue>,
    pub examples: Option<Vec<DisplayValue>>,

    // Integer OR Float
    pub minimum: Option<DisplayValue>,
    pub maximum: Option<DisplayValue>,
    pub exclusive_minimum: Option<DisplayValue>,
    pub exclusive_maximum: Option<DisplayValue>,
    pub multiple_of: Option<DisplayValue>,
    // String
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub format: Option<StringFormat>,
    pub pattern: Option<String>,

    // Array
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
    pub unique_items: Option<bool>,
    pub values_order: Option<ResolvedFormatOrder<XTombiArrayValuesOrder>>,

    // Table
    pub required_keys: Option<Vec<String>>,
    pub min_keys: Option<usize>,
    pub max_keys: Option<usize>,
    pub key_patterns: Option<Vec<String>>,
    pub additional_keys: Option<bool>,
    pub pattern_keys: bool,
    pub keys_order: Option<ResolvedFormatOrder<XTombiTableKeysOrder>>,
    pub array_values_order_by: Option<ArrayValuesOrderBy>,
}

impl std::fmt::Display for ValueConstraints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(r#enum) = &self.r#enum {
            write_list(
                f,
                "Enum Values:",
                r#enum.iter().map(|value| format!("`{value}`")),
            )?;
        }

        if let Some(default) = &self.default {
            write!(f, "Default: `{default}`\n\n")?;
        }

        if let Some(examples) = &self.examples {
            write_list(
                f,
                "Examples:",
                examples.iter().map(|example| format!("`{example}`")),
            )?;
        }

        if let Some(minimum) = &self.minimum {
            write!(f, "Minimum: `{minimum}`\n\n")?;
        }

        if let Some(exclusive_minimum) = &self.exclusive_minimum {
            write!(f, "Exclusive Minimum: `{exclusive_minimum}`\n\n")?;
        }

        if let Some(maximum) = &self.maximum {
            write!(f, "Maximum: `{maximum}`\n\n")?;
        }

        if let Some(exclusive_maximum) = &self.exclusive_maximum {
            write!(f, "Exclusive Maximum: `{exclusive_maximum}`\n\n")?;
        }

        if let Some(multiple_of) = &self.multiple_of {
            write!(f, "Multiple of: `{multiple_of}`\n\n")?;
        }

        if let Some(min_length) = self.min_length {
            write!(f, "Min Length: `{min_length}`\n\n")?;
        }

        if let Some(max_length) = self.max_length {
            write!(f, "Max Length: `{max_length}`\n\n")?;
        }

        if let Some(format) = &self.format {
            write!(f, "Format: `{format}`\n\n")?;
        }

        if let Some(pattern) = &self.pattern {
            write!(f, "Pattern: `{pattern}`\n\n")?;
        }

        if let Some(min_items) = self.min_items {
            write!(f, "Min Values: `{min_items}`\n\n")?;
        }

        if let Some(max_items) = self.max_items {
            write!(f, "Max Values: `{max_items}`\n\n")?;
        }

        if self.unique_items.unwrap_or_default() {
            write!(f, "Unique Values: `true`\n\n")?;
        }

        if let Some(values_order) = &self.values_order {
            write_array_values_order(f, &values_order.order, values_order.disabled)?;
        }

        if let Some(required_keys) = &self.required_keys {
            write_list(
                f,
                "Required Keys:",
                required_keys.iter().map(|key| format!("`{key}`")),
            )?;
        }

        if let Some(min_keys) = self.min_keys {
            write!(f, "Min Keys: `{min_keys}`\n\n")?;
        }

        if let Some(max_keys) = self.max_keys {
            write!(f, "Max Keys: `{max_keys}`\n\n")?;
        }

        if let Some(key_patterns) = &self.key_patterns {
            write_list(
                f,
                "Key Patterns:",
                key_patterns.iter().map(|pattern| format!("`{pattern}`")),
            )?;
        }

        if self.additional_keys.unwrap_or_default() {
            write!(f, "Additional Keys: `true`\n\n")?;
        }

        if self.pattern_keys {
            write!(f, "Pattern Keys: `true`\n\n")?;
        }

        if let Some(keys_order) = &self.keys_order {
            write_table_keys_order(f, &keys_order.order, keys_order.disabled)?;
        }

        if let Some(array_values_order_by) = &self.array_values_order_by {
            write!(f, "Array Values Order By: `{array_values_order_by}`\n\n")?;
        }

        Ok(())
    }
}

fn write_array_values_order(
    f: &mut std::fmt::Formatter<'_>,
    values_order: &XTombiArrayValuesOrder,
    strike: bool,
) -> std::fmt::Result {
    match values_order {
        XTombiArrayValuesOrder::All(values_order) => {
            writeln!(f, "Values Order: {}\n", markdown_code(values_order, strike))
        }
        XTombiArrayValuesOrder::Groups(values_order) => match values_order {
            ArrayValuesOrderGroup::OneOf(values_order) => write_list(
                f,
                &format!("Values Order: {}", markdown_code("oneOf", strike)),
                values_order
                    .iter()
                    .map(|value| markdown_code(value, strike)),
            ),
            ArrayValuesOrderGroup::AnyOf(values_order) => write_list(
                f,
                &format!("Values Order: {}", markdown_code("anyOf", strike)),
                values_order
                    .iter()
                    .map(|value| markdown_code(value, strike)),
            ),
        },
    }
}

fn write_table_keys_order(
    f: &mut std::fmt::Formatter<'_>,
    keys_order: &XTombiTableKeysOrder,
    strike: bool,
) -> std::fmt::Result {
    match keys_order {
        XTombiTableKeysOrder::All(keys_order) => {
            writeln!(f, "Keys Order: {}\n", markdown_code(keys_order, strike))
        }
        XTombiTableKeysOrder::Groups(keys_order) => write_list(
            f,
            "Keys Order:",
            keys_order
                .iter()
                .map(|key| format!("{}: {}", key.target, markdown_code(key.order, strike))),
        ),
    }
}

/// Writes a header followed by a tight bullet list.
///
/// Blank lines only separate blocks, never the lines inside one.
fn write_list(
    f: &mut std::fmt::Formatter<'_>,
    header: &str,
    items: impl Iterator<Item = String>,
) -> std::fmt::Result {
    writeln!(f, "{header}")?;
    for item in items {
        writeln!(f, "- {item}")?;
    }
    writeln!(f)
}

fn markdown_code(value: impl std::fmt::Display, strike: bool) -> String {
    let code = format!("`{value}`");
    if strike { format!("~~{code}~~") } else { code }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tombi_schema_store::{ResolvedFormatOrder, XTombiArrayValuesOrder, XTombiTableKeysOrder};
    use tombi_x_keyword::{ArrayValuesOrder, TableKeysOrder};

    use super::{DisplayValue, ValueConstraints};

    #[test]
    fn renders_disabled_sort_orders_with_strikethrough() {
        let constraints = ValueConstraints {
            values_order: Some(ResolvedFormatOrder {
                order: XTombiArrayValuesOrder::All(ArrayValuesOrder::Descending),
                disabled: true,
            }),
            keys_order: Some(ResolvedFormatOrder {
                order: XTombiTableKeysOrder::All(TableKeysOrder::Ascending),
                disabled: true,
            }),
            ..Default::default()
        };

        assert_eq!(
            constraints.to_string().trim(),
            textwrap::dedent(
                r#"
                Values Order: ~~`descending`~~

                Keys Order: ~~`ascending`~~
                "#
            )
            .trim()
        );
    }

    #[test]
    fn renders_enum_values_as_a_tight_list() {
        let constraints = ValueConstraints {
            r#enum: Some(vec![
                DisplayValue::String("function".to_string()),
                DisplayValue::String("class".to_string()),
            ]),
            default: Some(DisplayValue::String("function".to_string())),
            ..Default::default()
        };

        assert_eq!(
            constraints.to_string().trim(),
            textwrap::dedent(
                r#"
                Enum Values:
                - `"function"`
                - `"class"`

                Default: `"function"`
                "#
            )
            .trim()
        );
    }
}
