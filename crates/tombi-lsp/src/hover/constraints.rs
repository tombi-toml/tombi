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
        enum_values.extend(r#enum.iter().filter_map(convert_fn));
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
            write!(f, "Enum Values:\n\n")?;
            for value in r#enum {
                write!(f, "- `{value}`\n\n")?;
            }
            writeln!(f)?;
        }

        if let Some(default) = &self.default {
            write!(f, "Default: `{default}`\n\n")?;
        }

        if let Some(examples) = &self.examples {
            write!(f, "Examples:\n\n")?;
            for example in examples {
                write!(f, "  - `{example}`\n\n")?;
            }
            writeln!(f)?;
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
            write!(f, "Required Keys:\n\n")?;
            for key in required_keys.iter() {
                write!(f, "- `{key}`\n\n")?;
            }
        }

        if let Some(min_keys) = self.min_keys {
            write!(f, "Min Keys: `{min_keys}`\n\n")?;
        }

        if let Some(max_keys) = self.max_keys {
            write!(f, "Max Keys: `{max_keys}`\n\n")?;
        }

        if let Some(key_patterns) = &self.key_patterns {
            write!(f, "Key Patterns:\n\n")?;
            for pattern_property in key_patterns.iter() {
                write!(f, "- `{pattern_property}`\n\n")?;
            }
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
            ArrayValuesOrderGroup::OneOf(values_order) => {
                writeln!(f, "Values Order: {}\n", markdown_code("oneOf", strike))?;
                for value in values_order.iter() {
                    writeln!(f, "  - {}\n", markdown_code(value, strike))?;
                }
                Ok(())
            }
            ArrayValuesOrderGroup::AnyOf(values_order) => {
                writeln!(f, "Values Order: {}\n", markdown_code("anyOf", strike))?;
                for value in values_order.iter() {
                    writeln!(f, "  - {}\n", markdown_code(value, strike))?;
                }
                Ok(())
            }
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
        XTombiTableKeysOrder::Groups(keys_order) => {
            writeln!(f, "Keys Order:\n")?;
            for key in keys_order.iter() {
                writeln!(
                    f,
                    "  - {}: {}\n",
                    key.target,
                    markdown_code(&key.order, strike)
                )?;
            }
            Ok(())
        }
    }
}

fn markdown_code(value: impl std::fmt::Display, strike: bool) -> String {
    let code = format!("`{value}`");
    if strike { format!("~~{code}~~") } else { code }
}

#[cfg(test)]
mod tests {
    use tombi_schema_store::{ResolvedFormatOrder, XTombiArrayValuesOrder, XTombiTableKeysOrder};
    use tombi_x_keyword::{ArrayValuesOrder, TableKeysOrder};

    use super::ValueConstraints;

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

        let rendered = constraints.to_string();
        assert!(rendered.contains("Values Order: ~~`descending`~~"));
        assert!(rendered.contains("Keys Order: ~~`ascending`~~"));
    }
}
