use indexmap::IndexSet;
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    Null,
    Boolean,
    Integer,
    Float,
    String,
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
    Array,
    Table,
    OneOf(Vec<ValueType>),
    AnyOf(Vec<ValueType>),
    AllOf(Vec<ValueType>),
}

impl ValueType {
    pub fn set_nullable(&mut self) {
        let value_type = self.clone();
        *self = match value_type {
            ValueType::Null => ValueType::Null,
            ValueType::Boolean
            | ValueType::Integer
            | ValueType::Float
            | ValueType::String
            | ValueType::OffsetDateTime
            | ValueType::LocalDateTime
            | ValueType::LocalDate
            | ValueType::LocalTime
            | ValueType::Array
            | ValueType::Table => ValueType::AnyOf(vec![value_type, ValueType::Null]),
            ValueType::OneOf(mut types) => {
                if !types.iter().any(|t| t.is_nullable()) {
                    types.push(ValueType::Null);
                }
                ValueType::OneOf(types)
            }
            ValueType::AnyOf(mut types) => {
                if !types.iter().all(|t| t.is_nullable()) {
                    types.push(ValueType::Null);
                }
                ValueType::AnyOf(types)
            }
            ValueType::AllOf(types) => {
                if types.iter().all(|t| !t.is_nullable()) {
                    ValueType::AnyOf(vec![ValueType::AllOf(types), ValueType::Null])
                } else {
                    ValueType::AllOf(types)
                }
            }
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            ValueType::Null => true,
            ValueType::Boolean
            | ValueType::Integer
            | ValueType::Float
            | ValueType::String
            | ValueType::OffsetDateTime
            | ValueType::LocalDateTime
            | ValueType::LocalDate
            | ValueType::LocalTime
            | ValueType::Array
            | ValueType::Table => false,
            ValueType::OneOf(types) | ValueType::AnyOf(types) => {
                types.iter().any(|t| t.is_nullable())
            }
            ValueType::AllOf(types) => types.iter().all(|t| t.is_nullable()),
        }
    }

    fn to_display(&self, is_root: bool) -> String {
        match self {
            ValueType::Null => {
                // NOTE: If this representation appears in the Hover of the Language Server, it is a bug.
                "Null".to_string()
            }
            ValueType::Boolean => "Boolean".to_string(),
            ValueType::Integer => "Integer".to_string(),
            ValueType::Float => "Float".to_string(),
            ValueType::String => "String".to_string(),
            ValueType::OffsetDateTime => "OffsetDateTime".to_string(),
            ValueType::LocalDateTime => "LocalDateTime".to_string(),
            ValueType::LocalDate => "LocalDate".to_string(),
            ValueType::LocalTime => "LocalTime".to_string(),
            ValueType::Array => "Array".to_string(),
            ValueType::Table => "Table".to_string(),
            ValueType::OneOf(types) => fmt_composit_types(types, '^', is_root),
            ValueType::AnyOf(types) => fmt_composit_types(types, '|', is_root),
            ValueType::AllOf(types) => fmt_composit_types(types, '&', is_root),
        }
    }

    /// Simplify the type by removing unnecessary nesting.
    ///
    /// For example, `OneOf([OneOf([A, B]), C])` will be simplified to `OneOf([A, B, C])`.
    /// Also, if `Null` is included, it is taken out at the end of the outermost. This always displays `? at the end of type display.
    pub fn simplify(&self) -> Self {
        // Macro to handle the common pattern of simplifying composite types (OneOf, AnyOf, AllOf)
        macro_rules! simplify_composite {
            ($value_types:expr, $current_variant:ident, $($other_variant:ident)|+) => {{
                let mut flattened = IndexSet::new();
                let mut has_null = false;

                for value_type in $value_types {
                    match value_type.simplify() {
                        ValueType::Null => has_null = true,
                        // Flatten nested types of the same variant
                        ValueType::$current_variant(nested_value_types) => {
                            for nested_value_type in nested_value_types {
                                if matches!(nested_value_type, ValueType::Null) {
                                    has_null = true;
                                } else {
                                    flattened.insert(nested_value_type);
                                }
                            }
                        }
                        // Handle nested types of other composite variants (one match arm per variant)
                        $(
                            ValueType::$other_variant(nested_value_types) => {
                                let non_nulls = nested_value_types
                                    .into_iter()
                                    .filter_map(|nested_value_type| {
                                        if matches!(nested_value_type, ValueType::Null) {
                                            has_null = true;
                                            None
                                        } else {
                                            Some(nested_value_type)
                                        }
                                    })
                                    .collect_vec();

                                if non_nulls.len() == 1 {
                                    flattened.insert(non_nulls.into_iter().next().unwrap());
                                } else if !non_nulls.is_empty() {
                                    flattened.insert(ValueType::$other_variant(non_nulls));
                                }
                            }
                        )+
                        other => {
                            flattened.insert(other);
                        }
                    }
                }

                if has_null {
                    flattened.insert(ValueType::Null);
                }

                ValueType::$current_variant(flattened.into_iter().collect())
            }};
        }

        let simplified = match self {
            ValueType::OneOf(value_types) => {
                simplify_composite!(value_types, OneOf, AnyOf | AllOf)
            }
            ValueType::AnyOf(value_types) => {
                simplify_composite!(value_types, AnyOf, AllOf | OneOf)
            }
            ValueType::AllOf(value_types) => {
                simplify_composite!(value_types, AllOf, OneOf | AnyOf)
            }
            other => other.to_owned(),
        };

        // Further simplify single-element composite types
        match simplified {
            ValueType::OneOf(value_types)
            | ValueType::AnyOf(value_types)
            | ValueType::AllOf(value_types)
                if value_types.len() == 1 =>
            {
                value_types.into_iter().next().unwrap()
            }
            _ => simplified,
        }
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.simplify().to_display(true))
    }
}

#[cfg(feature = "document-tree")]
impl From<tombi_document_tree::ValueType> for ValueType {
    fn from(value_type: tombi_document_tree::ValueType) -> Self {
        match value_type {
            tombi_document_tree::ValueType::Boolean => ValueType::Boolean,
            tombi_document_tree::ValueType::Integer => ValueType::Integer,
            tombi_document_tree::ValueType::Float => ValueType::Float,
            tombi_document_tree::ValueType::String => ValueType::String,
            tombi_document_tree::ValueType::OffsetDateTime => ValueType::OffsetDateTime,
            tombi_document_tree::ValueType::LocalDateTime => ValueType::LocalDateTime,
            tombi_document_tree::ValueType::LocalDate => ValueType::LocalDate,
            tombi_document_tree::ValueType::LocalTime => ValueType::LocalTime,
            tombi_document_tree::ValueType::Array => ValueType::Array,
            tombi_document_tree::ValueType::Table => ValueType::Table,
            tombi_document_tree::ValueType::Incomplete => unreachable!("incomplete value"),
        }
    }
}

fn fmt_composit_types(types: &[ValueType], separator: char, is_root: bool) -> String {
    let mut nullable = false;
    let non_null_types = types
        .iter()
        .filter(|t| {
            if let ValueType::Null = t {
                nullable = true;
                false
            } else {
                true
            }
        })
        .collect_vec();

    if nullable {
        if non_null_types.len() == 1 {
            format!("{}?", non_null_types[0].to_display(false))
        } else {
            format!(
                "({})?",
                non_null_types
                    .iter()
                    .map(|t| t.to_display(false))
                    .join(&format!(" {separator} ")),
            )
        }
    } else if is_root {
        non_null_types
            .iter()
            .map(|t| t.to_display(false))
            .join(&format!(" {separator} "))
            .to_string()
    } else if non_null_types.len() == 1 {
        non_null_types[0].to_display(false).to_string()
    } else {
        format!(
            "({})",
            non_null_types
                .iter()
                .map(|t| t.to_display(false))
                .join(&format!(" {separator} ")),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn any_of_array_null() {
        let value_type = ValueType::AnyOf(
            vec![ValueType::Array, ValueType::Null]
                .into_iter()
                .collect(),
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "Array?");
    }

    #[test]
    fn one_of_array_null() {
        let value_type = ValueType::OneOf(
            vec![ValueType::Array, ValueType::Null]
                .into_iter()
                .collect(),
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "Array?");
    }

    #[test]
    fn all_of_array_null() {
        let value_type = ValueType::AllOf(
            vec![ValueType::Array, ValueType::Null]
                .into_iter()
                .collect(),
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "Array?");
    }

    #[test]
    fn nullable_one_of() {
        let value_type = ValueType::OneOf(
            vec![ValueType::Array, ValueType::Table, ValueType::Null]
                .into_iter()
                .collect(),
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "(Array ^ Table)?");
    }

    #[test]
    fn nullable_any_of() {
        let value_type = ValueType::AnyOf(
            vec![ValueType::Array, ValueType::Table, ValueType::Null]
                .into_iter()
                .collect(),
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "(Array | Table)?");
    }

    #[test]
    fn nullable_all_of() {
        let value_type =
            ValueType::AllOf(vec![ValueType::Array, ValueType::Table, ValueType::Null]);
        pretty_assertions::assert_eq!(value_type.to_string(), "(Array & Table)?");
    }

    #[test]
    fn same_type_one_of() {
        let value_type = ValueType::OneOf(vec![
            ValueType::OneOf(vec![ValueType::Boolean, ValueType::Null]),
            ValueType::Boolean,
        ]);
        pretty_assertions::assert_eq!(value_type.to_string(), "Boolean?");
    }

    #[test]
    fn same_type_any_of() {
        let value_type = ValueType::AnyOf(vec![
            ValueType::OneOf(vec![ValueType::Boolean, ValueType::Null]),
            ValueType::OneOf(vec![ValueType::Boolean, ValueType::Null]),
        ]);
        pretty_assertions::assert_eq!(value_type.to_string(), "Boolean?");
    }

    #[test]
    fn same_type_all_of() {
        let value_type = ValueType::AllOf(vec![
            ValueType::OneOf(vec![ValueType::Boolean, ValueType::Null]),
            ValueType::Boolean,
            ValueType::Null,
        ]);
        pretty_assertions::assert_eq!(value_type.to_string(), "Boolean?");
    }

    #[test]
    fn nested_one_of() {
        let value_type = ValueType::OneOf(
            vec![
                ValueType::OneOf(vec![ValueType::Boolean, ValueType::String]),
                ValueType::Array,
                ValueType::Table,
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "(Boolean ^ String) ^ Array ^ Table"
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "Boolean ^ String ^ Array ^ Table");
    }

    #[test]
    fn nested_any_of() {
        let value_type = ValueType::AnyOf(
            vec![
                ValueType::AnyOf(vec![ValueType::Boolean, ValueType::String]),
                ValueType::Array,
                ValueType::Table,
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "(Boolean | String) | Array | Table"
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "Boolean | String | Array | Table");
    }

    #[test]
    fn nested_all_of() {
        let value_type = ValueType::AllOf(
            vec![
                ValueType::AllOf(vec![ValueType::Boolean, ValueType::String]),
                ValueType::Array,
                ValueType::Table,
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "(Boolean & String) & Array & Table"
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "Boolean & String & Array & Table");
    }

    #[test]
    fn nested_one_of_withnullable() {
        let value_type = ValueType::OneOf(
            vec![
                ValueType::OneOf(vec![ValueType::Boolean, ValueType::String]),
                ValueType::Array,
                ValueType::Table,
                ValueType::Null,
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "((Boolean ^ String) ^ Array ^ Table)?"
        );
        pretty_assertions::assert_eq!(
            value_type.to_string(),
            "(Boolean ^ String ^ Array ^ Table)?"
        );
    }

    #[test]
    fn nested_one_of_with_nested_nullable() {
        let value_type = ValueType::OneOf(
            vec![
                ValueType::OneOf(vec![ValueType::Boolean, ValueType::String, ValueType::Null]),
                ValueType::Array,
                ValueType::Table,
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "(Boolean ^ String)? ^ Array ^ Table"
        );
        pretty_assertions::assert_eq!(
            value_type.to_string(),
            "(Boolean ^ String ^ Array ^ Table)?"
        );
    }

    #[test]
    fn nested_any_of_with_nested_nullable() {
        let value_type = ValueType::AnyOf(
            vec![
                ValueType::AnyOf(vec![ValueType::Boolean, ValueType::String, ValueType::Null]),
                ValueType::Array,
                ValueType::Table,
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "(Boolean | String)? | Array | Table"
        );
        pretty_assertions::assert_eq!(
            value_type.to_string(),
            "(Boolean | String | Array | Table)?"
        );
    }

    #[test]
    fn nested_all_of_with_nested_nullable() {
        let value_type = ValueType::AllOf(
            vec![
                ValueType::AllOf(vec![ValueType::Boolean, ValueType::String, ValueType::Null]),
                ValueType::Array,
                ValueType::Table,
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "(Boolean & String)? & Array & Table"
        );
        pretty_assertions::assert_eq!(
            value_type.to_string(),
            "(Boolean & String & Array & Table)?"
        );
    }

    #[test]
    fn nested_one_of_any_of() {
        let value_type = ValueType::OneOf(
            vec![
                ValueType::OneOf(vec![ValueType::Boolean, ValueType::String]),
                ValueType::AnyOf(vec![ValueType::Array, ValueType::Table]),
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "(Boolean ^ String) ^ (Array | Table)"
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "Boolean ^ String ^ (Array | Table)");
    }

    #[test]
    fn nested_one_of_any_of_with_nullable() {
        let value_type = ValueType::OneOf(
            vec![
                ValueType::OneOf(vec![ValueType::Boolean, ValueType::String]),
                ValueType::AnyOf(vec![ValueType::Array, ValueType::Table, ValueType::Null]),
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "(Boolean ^ String) ^ (Array | Table)?"
        );
        pretty_assertions::assert_eq!(
            value_type.to_string(),
            "(Boolean ^ String ^ (Array | Table))?"
        );
    }

    #[test]
    fn slim_same_type() {
        let value_type = ValueType::OneOf(
            vec![
                ValueType::OneOf(vec![ValueType::Boolean, ValueType::Array]),
                ValueType::Boolean,
                ValueType::Array,
            ]
            .into_iter()
            .collect(),
        );
        pretty_assertions::assert_eq!(
            value_type.to_display(true),
            "(Boolean ^ Array) ^ Boolean ^ Array"
        );
        pretty_assertions::assert_eq!(value_type.to_string(), "Boolean ^ Array");
    }
}
