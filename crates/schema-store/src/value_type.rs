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
    pub fn into_nullable(self) -> Self {
        match self {
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
            | ValueType::Table => ValueType::AnyOf(vec![self, ValueType::Null]),
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
        match self {
            ValueType::OneOf(types) => {
                let mut simplified_types = Vec::new();
                for t in types {
                    match t.simplify() {
                        ValueType::OneOf(nested_types) => {
                            let mut has_nullable = false;
                            simplified_types.extend(nested_types.into_iter().filter_map(|t| {
                                if let ValueType::Null = t {
                                    has_nullable = true;
                                    None
                                } else {
                                    Some(t)
                                }
                            }));
                            if has_nullable {
                                simplified_types.push(ValueType::Null);
                            }
                        }
                        ValueType::AnyOf(nested_types) => {
                            let mut has_nullable = false;
                            simplified_types.push(ValueType::AnyOf(
                                nested_types
                                    .into_iter()
                                    .filter_map(|t| {
                                        if let ValueType::Null = t {
                                            has_nullable = true;
                                            None
                                        } else {
                                            Some(t)
                                        }
                                    })
                                    .collect(),
                            ));
                            if has_nullable {
                                simplified_types.push(ValueType::Null);
                            }
                        }
                        ValueType::AllOf(nested_types) => {
                            let mut has_nullable = false;
                            simplified_types.push(ValueType::AllOf(
                                nested_types
                                    .into_iter()
                                    .filter_map(|t| {
                                        if let ValueType::Null = t {
                                            has_nullable = true;
                                            None
                                        } else {
                                            Some(t)
                                        }
                                    })
                                    .collect(),
                            ));
                            if has_nullable {
                                simplified_types.push(ValueType::Null);
                            }
                        }
                        other => {
                            simplified_types.push(other);
                        }
                    }
                }
                ValueType::OneOf(simplified_types)
            }
            ValueType::AnyOf(types) => {
                let mut simplified_types = Vec::new();
                for t in types {
                    match t.simplify() {
                        ValueType::OneOf(nested_types) => {
                            let mut has_nullable = false;
                            simplified_types.push(ValueType::OneOf(
                                nested_types
                                    .into_iter()
                                    .filter_map(|t| {
                                        if let ValueType::Null = t {
                                            has_nullable = true;
                                            None
                                        } else {
                                            Some(t)
                                        }
                                    })
                                    .collect(),
                            ));
                            if has_nullable {
                                simplified_types.push(ValueType::Null);
                            }
                        }
                        ValueType::AnyOf(nested_types) => {
                            let mut has_nullable = false;
                            simplified_types.extend(nested_types.into_iter().filter_map(|t| {
                                if let ValueType::Null = t {
                                    has_nullable = true;
                                    None
                                } else {
                                    Some(t)
                                }
                            }));
                            if has_nullable {
                                simplified_types.push(ValueType::Null);
                            }
                        }
                        ValueType::AllOf(nested_types) => {
                            let mut has_nullable = false;
                            simplified_types.push(ValueType::AllOf(
                                nested_types
                                    .into_iter()
                                    .filter_map(|t| {
                                        if let ValueType::Null = t {
                                            has_nullable = true;
                                            None
                                        } else {
                                            Some(t)
                                        }
                                    })
                                    .collect(),
                            ));
                            if has_nullable {
                                simplified_types.push(ValueType::Null);
                            }
                        }
                        other => {
                            simplified_types.push(other);
                        }
                    }
                }
                ValueType::AnyOf(simplified_types)
            }
            ValueType::AllOf(types) => {
                let mut simplified_types = Vec::new();
                for t in types {
                    match t.simplify() {
                        ValueType::OneOf(nested_types) => {
                            let mut has_nullable = false;
                            simplified_types.push(ValueType::OneOf(
                                nested_types
                                    .into_iter()
                                    .filter_map(|t| {
                                        if let ValueType::Null = t {
                                            has_nullable = true;
                                            None
                                        } else {
                                            Some(t)
                                        }
                                    })
                                    .collect(),
                            ));
                            if has_nullable {
                                simplified_types.push(ValueType::Null);
                            }
                        }
                        ValueType::AnyOf(nested_types) => {
                            let mut has_nullable = false;
                            simplified_types.push(ValueType::AnyOf(
                                nested_types
                                    .into_iter()
                                    .filter_map(|t| {
                                        if let ValueType::Null = t {
                                            has_nullable = true;
                                            None
                                        } else {
                                            Some(t)
                                        }
                                    })
                                    .collect(),
                            ));
                            if has_nullable {
                                simplified_types.push(ValueType::Null);
                            }
                        }
                        ValueType::AllOf(nested_types) => {
                            let mut has_nullable = false;
                            simplified_types.extend(nested_types.into_iter().filter_map(|t| {
                                if let ValueType::Null = t {
                                    has_nullable = true;
                                    None
                                } else {
                                    Some(t)
                                }
                            }));
                            if has_nullable {
                                simplified_types.push(ValueType::Null);
                            }
                        }
                        other => {
                            simplified_types.push(other);
                        }
                    }
                }
                ValueType::AllOf(simplified_types)
            }
            other => other.to_owned(),
        }
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.simplify().to_display(true))
    }
}

impl From<document_tree::ValueType> for ValueType {
    fn from(value_type: document_tree::ValueType) -> Self {
        match value_type {
            document_tree::ValueType::Boolean => ValueType::Boolean,
            document_tree::ValueType::Integer => ValueType::Integer,
            document_tree::ValueType::Float => ValueType::Float,
            document_tree::ValueType::String => ValueType::String,
            document_tree::ValueType::OffsetDateTime => ValueType::OffsetDateTime,
            document_tree::ValueType::LocalDateTime => ValueType::LocalDateTime,
            document_tree::ValueType::LocalDate => ValueType::LocalDate,
            document_tree::ValueType::LocalTime => ValueType::LocalTime,
            document_tree::ValueType::Array => ValueType::Array,
            document_tree::ValueType::Table => ValueType::Table,
        }
    }
}

fn fmt_composit_types(types: &[ValueType], separator: char, is_root: bool) -> String {
    let mut nullable = false;
    let non_null_types = types
        .into_iter()
        .filter(|t| {
            if let ValueType::Null = t {
                nullable = true;
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    if nullable {
        if non_null_types.len() == 1 {
            format!("{}?", non_null_types[0].to_display(false))
        } else {
            format!(
                "({})?",
                non_null_types
                    .iter()
                    .map(|t| t.to_display(false))
                    .join(&format!(" {} ", separator)),
            )
        }
    } else {
        if is_root {
            format!(
                "{}",
                non_null_types
                    .iter()
                    .map(|t| t.to_display(false))
                    .join(&format!(" {} ", separator))
            )
        } else {
            if non_null_types.len() == 1 {
                format!("{}", non_null_types[0].to_display(false))
            } else {
                format!(
                    "({})",
                    non_null_types
                        .iter()
                        .map(|t| t.to_display(false))
                        .join(&format!(" {} ", separator)),
                )
            }
        }
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
}
