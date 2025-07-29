use tombi_x_keyword::{ArrayValuesOrder, TableKeysOrder};

use super::display_value::DisplayValue;

/// Build enumerate values from const_value and enumerate fields
///
/// This function is used to create the enumerate field for ValueConstraints
/// by combining const_value and enumerate from various schema types.
pub fn build_enumerate_values<T, F>(
    const_value: &Option<T>,
    enumerate: &Option<Vec<T>>,
    convert_fn: F,
) -> Option<Vec<DisplayValue>>
where
    F: Fn(&T) -> Option<DisplayValue>,
{
    let const_len = if const_value.is_some() { 1 } else { 0 };
    let enumerate_len = enumerate
        .as_ref()
        .map(|value| value.len())
        .unwrap_or_default();
    let mut enumerate_values = Vec::with_capacity(const_len + enumerate_len);

    if let Some(const_value) = const_value {
        if let Some(display_value) = convert_fn(const_value) {
            enumerate_values.push(display_value);
        }
    }

    if let Some(enumerate) = enumerate {
        enumerate_values.extend(enumerate.iter().filter_map(convert_fn));
    }

    if enumerate_values.is_empty() {
        None
    } else {
        Some(enumerate_values)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValueConstraints {
    // Common
    pub enumerate: Option<Vec<DisplayValue>>,
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
    pub pattern: Option<String>,

    // Array
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
    pub unique_items: Option<bool>,
    pub values_order: Option<ArrayValuesOrder>,

    // Table
    pub required_keys: Option<Vec<String>>,
    pub min_keys: Option<usize>,
    pub max_keys: Option<usize>,
    pub key_patterns: Option<Vec<String>>,
    pub additional_keys: Option<bool>,
    pub keys_order: Option<TableKeysOrder>,
}

impl std::fmt::Display for ValueConstraints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(enumerate) = &self.enumerate {
            write!(f, "Enumerated Values:\n\n")?;
            for value in enumerate {
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
            write!(f, "Minimum Length: `{min_length}`\n\n")?;
        }

        if let Some(max_length) = self.max_length {
            write!(f, "Maximum Length: `{max_length}`\n\n")?;
        }

        if let Some(pattern) = &self.pattern {
            write!(f, "Pattern: `{pattern}`\n\n")?;
        }

        if let Some(min_items) = self.min_items {
            write!(f, "Minimum Items: `{min_items}`\n\n")?;
        }

        if let Some(max_items) = self.max_items {
            write!(f, "Maximum Items: `{max_items}`\n\n")?;
        }

        if self.unique_items.unwrap_or(false) {
            write!(f, "Unique Items: `true`\n\n")?;
        }

        if let Some(values_order) = &self.values_order {
            write!(f, "Values Order: `{values_order}`\n\n")?;
        }

        if let Some(required_keys) = &self.required_keys {
            write!(f, "Required Keys:\n\n")?;
            for key in required_keys.iter() {
                write!(f, "- `{key}`\n\n")?;
            }
        }

        if let Some(min_keys) = self.min_keys {
            write!(f, "Minimum Keys: `{min_keys}`\n\n")?;
        }

        if let Some(max_keys) = self.max_keys {
            write!(f, "Maximum Keys: `{max_keys}`\n\n")?;
        }

        if let Some(key_patterns) = &self.key_patterns {
            write!(f, "Key Patterns:\n\n")?;
            for pattern_property in key_patterns.iter() {
                write!(f, "- `{pattern_property}`\n\n")?;
            }
        }

        if self.additional_keys.unwrap_or(false) {
            write!(f, "Additional Keys: `true`\n\n")?;
        }

        if let Some(keys_order) = &self.keys_order {
            write!(f, "Keys Order: `{keys_order}`\n\n")?;
        }

        Ok(())
    }
}
