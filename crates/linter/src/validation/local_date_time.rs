use super::{validate_all_of, validate_any_of, validate_one_of, Validate};
use document_tree::LocalDateTime;
use schema_store::ValueType;

impl Validate for LocalDateTime {
    fn validate(
        &self,
        toml_version: config::TomlVersion,
        value_schema: &schema_store::ValueSchema,
        definitions: &schema_store::SchemaDefinitions,
    ) -> Result<(), Vec<crate::Error>> {
        let mut errors = vec![];

        match value_schema.value_type() {
            ValueType::LocalDateTime
            | ValueType::OneOf(_)
            | ValueType::AnyOf(_)
            | ValueType::AllOf(_) => {}
            ValueType::Null => return Ok(()),
            value_type => {
                return Err(vec![crate::Error {
                    kind: crate::ErrorKind::TypeMismatch {
                        expected: schema_store::ValueType::LocalDateTime,
                        actual: value_type,
                    },
                    range: self.range(),
                }]);
            }
        }

        let local_date_time_schema = match value_schema {
            schema_store::ValueSchema::LocalDateTime(local_date_time_schema) => {
                local_date_time_schema
            }
            schema_store::ValueSchema::OneOf(one_of_schema) => {
                return validate_one_of(self, toml_version, one_of_schema, definitions)
            }
            schema_store::ValueSchema::AnyOf(any_of_schema) => {
                return validate_any_of(self, toml_version, any_of_schema, definitions)
            }
            schema_store::ValueSchema::AllOf(all_of_schema) => {
                return validate_all_of(self, toml_version, all_of_schema, definitions)
            }
            _ => unreachable!("Expected a Local Date-Time schema"),
        };

        let value_string = self.node().to_string();
        if let Some(enumerate) = &local_date_time_schema.enumerate {
            if !enumerate.contains(&value_string) {
                errors.push(crate::Error {
                    kind: crate::ErrorKind::Eunmerate {
                        expected: enumerate.iter().map(ToString::to_string).collect(),
                        actual: value_string,
                    },
                    range: self.range(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
