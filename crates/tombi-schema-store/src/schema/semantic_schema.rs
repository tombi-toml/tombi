use tombi_json::{Number, ObjectNode, Value, ValueNode};

use super::{Deprecation, referable_schema::ReferenceKind};
use crate::JsonSchemaDialect;

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub range: tombi_text::Range,
}

impl<T> Spanned<T> {
    fn new(value: T, range: tombi_text::Range) -> Self {
        Self { value, range }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaType {
    Null,
    Boolean,
    Object,
    Array,
    Number,
    String,
    Integer,
}

impl SchemaType {
    /// The JSON Schema type a TOML value instance is validated against.
    ///
    /// TOML date/time values are validated as strings, and `Incomplete` has no
    /// instance type yet.
    #[cfg(feature = "document-tree")]
    pub fn from_value_type(value_type: tombi_document_tree_syntax::ValueType) -> Option<Self> {
        use tombi_document_tree_syntax::ValueType;

        match value_type {
            ValueType::Boolean => Some(Self::Boolean),
            ValueType::Integer => Some(Self::Integer),
            ValueType::Float => Some(Self::Number),
            ValueType::String
            | ValueType::OffsetDateTime
            | ValueType::LocalDateTime
            | ValueType::LocalDate
            | ValueType::LocalTime => Some(Self::String),
            ValueType::Array => Some(Self::Array),
            ValueType::Table => Some(Self::Object),
            ValueType::Incomplete => None,
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "null" => Some(Self::Null),
            "boolean" => Some(Self::Boolean),
            "object" => Some(Self::Object),
            "array" => Some(Self::Array),
            "number" => Some(Self::Number),
            "string" => Some(Self::String),
            "integer" => Some(Self::Integer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAssertion {
    pub allowed: Vec<Spanned<SchemaType>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticSchema {
    Boolean(Spanned<bool>),
    Object(Box<SemanticSchemaObject>),
    Composite(SemanticCompositeSchema),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticCompositeKind {
    OneOf,
    AnyOf,
    AllOf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticCompositeSchema {
    pub kind: SemanticCompositeKind,
    pub schemas: Vec<SemanticSchema>,
    pub range: tombi_text::Range,
}

impl SemanticSchema {
    pub fn from_value_node(value: &ValueNode, dialect: Option<JsonSchemaDialect>) -> Option<Self> {
        match value {
            ValueNode::Bool(value) => Some(Self::Boolean(Spanned::new(value.value, value.range))),
            ValueNode::Object(object) => Some(Self::from_object_node(object, dialect)),
            _ => None,
        }
    }

    pub fn from_object_node(object: &ObjectNode, dialect: Option<JsonSchemaDialect>) -> Self {
        Self::Object(Box::new(SemanticSchemaObject::new(object, dialect)))
    }

    pub fn range(&self) -> tombi_text::Range {
        match self {
            Self::Boolean(value) => value.range,
            Self::Object(object) => object.range,
            Self::Composite(composite) => composite.range,
        }
    }

    pub fn composite(
        kind: SemanticCompositeKind,
        schemas: Vec<SemanticSchema>,
        range: tombi_text::Range,
    ) -> Self {
        Self::Composite(SemanticCompositeSchema {
            kind,
            schemas,
            range,
        })
    }

    /// `true` only when this schema object contains the JSON Schema `type`
    /// assertion.  Type-specific keywords such as `minimum`, `minLength`, and
    /// `unevaluatedProperties` deliberately do not count as type assertions.
    pub fn has_type_assertion(&self) -> bool {
        match self {
            Self::Object(object) => object.type_assertion.is_some(),
            Self::Composite(composite) => composite.schemas.iter().any(Self::has_type_assertion),
            Self::Boolean(_) => false,
        }
    }

    /// Returns whether this schema object itself contains `type`, without
    /// traversing referenced or applicator schemas.
    pub fn has_direct_type_assertion(&self) -> bool {
        matches!(self, Self::Object(object) if object.type_assertion.is_some())
    }

    /// Returns whether this schema object itself contains a finite, generic
    /// literal assertion. Unlike type-specific keywords, `const` and `enum`
    /// constrain every instance and therefore safely determine presentation
    /// types even when `type` is absent.
    pub fn has_direct_literal_assertion(&self) -> bool {
        matches!(self, Self::Object(object) if object.assertions.const_value.is_some() || object.assertions.enum_values.is_some())
    }

    /// Returns whether an instance type is admitted by the schema's explicit
    /// `type` assertion. Keywords for other types never imply a type here.
    pub fn accepts_instance_type(&self, instance_type: SchemaType) -> bool {
        match self {
            Self::Boolean(value) => value.value,
            Self::Object(object) => {
                object.type_assertion.as_ref().is_none_or(|assertion| {
                    assertion.allowed.iter().any(|allowed| {
                        allowed.value == instance_type
                            || (instance_type == SchemaType::Integer
                                && allowed.value == SchemaType::Number)
                    })
                }) && object
                    .assertions
                    .const_value
                    .as_ref()
                    .is_none_or(|value| literal_has_type(&value.value, instance_type))
                    && object.assertions.enum_values.as_ref().is_none_or(|values| {
                        values
                            .iter()
                            .any(|value| literal_has_type(&value.value, instance_type))
                    })
                    && object
                        .applicators
                        .all_of
                        .iter()
                        .all(|schema| schema.accepts_instance_type(instance_type))
                    && (object.applicators.any_of.is_empty()
                        || object
                            .applicators
                            .any_of
                            .iter()
                            .any(|schema| schema.accepts_instance_type(instance_type)))
                    && (object.applicators.one_of.is_empty()
                        || object
                            .applicators
                            .one_of
                            .iter()
                            .any(|schema| schema.accepts_instance_type(instance_type)))
            }
            Self::Composite(composite) => match composite.kind {
                SemanticCompositeKind::AllOf => composite
                    .schemas
                    .iter()
                    .all(|schema| schema.accepts_instance_type(instance_type)),
                SemanticCompositeKind::AnyOf | SemanticCompositeKind::OneOf => composite
                    .schemas
                    .iter()
                    .any(|schema| schema.accepts_instance_type(instance_type)),
            },
        }
    }

    pub fn has_direct_constraints_for_type(&self, instance_type: SchemaType) -> bool {
        let Self::Object(object) = self else {
            return false;
        };
        if object.assertions.const_value.is_some() || object.assertions.enum_values.is_some() {
            return true;
        }
        match instance_type {
            SchemaType::Null | SchemaType::Boolean => false,
            SchemaType::Integer | SchemaType::Number => {
                let constraints = &object.constraints.numeric;
                constraints.multiple_of.is_some()
                    || constraints.minimum.is_some()
                    || constraints.maximum.is_some()
                    || constraints.exclusive_minimum.is_some()
                    || constraints.exclusive_maximum.is_some()
            }
            SchemaType::String => {
                let constraints = &object.constraints.string;
                constraints.min_length.is_some()
                    || constraints.max_length.is_some()
                    || constraints.pattern.is_some()
                    || constraints.format.is_some()
                    || constraints.content_encoding.is_some()
                    || constraints.content_media_type.is_some()
                    || constraints.content_schema.is_some()
            }
            SchemaType::Array => {
                let constraints = &object.constraints.array;
                !constraints.prefix_items.is_empty()
                    || constraints.items.is_some()
                    || constraints.additional_items.is_some()
                    || constraints.contains.is_some()
                    || constraints.unevaluated_items.is_some()
                    || constraints.min_items.is_some()
                    || constraints.max_items.is_some()
                    || constraints.unique_items.is_some()
                    || constraints.min_contains.is_some()
                    || constraints.max_contains.is_some()
            }
            SchemaType::Object => {
                let constraints = &object.constraints.object;
                !constraints.properties.is_empty()
                    || !constraints.pattern_properties.is_empty()
                    || constraints.additional_properties.is_some()
                    || constraints.unevaluated_properties.is_some()
                    || constraints.property_names.is_some()
                    || !constraints.required.is_empty()
                    || !constraints.dependent_required.is_empty()
                    || !constraints.dependent_schemas.is_empty()
                    || constraints.min_properties.is_some()
                    || constraints.max_properties.is_some()
            }
        }
    }

    /// Creates a typed view used by LSP presentation and traversal.
    /// The semantic tree remains authoritative; this projection is always
    /// selected from the actual instance type and never inferred from keywords.
    pub fn schema_view_for_type(
        &self,
        instance_type: SchemaType,
        string_formats: Option<&[tombi_x_keyword::StringFormat]>,
    ) -> Option<super::SchemaView> {
        self.schema_view_for_type_with_collectors(instance_type, string_formats, None, None)
    }

    pub(crate) fn schema_view_for_type_with_collectors(
        &self,
        instance_type: SchemaType,
        string_formats: Option<&[tombi_x_keyword::StringFormat]>,
        anchor_collector: Option<&mut super::AnchorCollector>,
        dynamic_anchor_collector: Option<&mut super::DynamicAnchorCollector>,
    ) -> Option<super::SchemaView> {
        if !self.accepts_instance_type(instance_type) {
            return None;
        }
        if let Self::Composite(composite) = self {
            let schemas = composite
                .schemas
                .iter()
                .filter_map(|schema| {
                    schema
                        .schema_view_for_type(instance_type, string_formats)
                        .map(|value| super::Referable::Resolved {
                            schema_uri: None,
                            value: std::sync::Arc::new(value),
                            semantic_schema: Some(std::sync::Arc::new(schema.clone())),
                        })
                })
                .collect();
            let schemas = std::sync::Arc::new(tokio::sync::RwLock::new(schemas));
            return Some(match composite.kind {
                SemanticCompositeKind::OneOf => super::SchemaView::OneOf(super::OneOfSchema {
                    schemas,
                    ..Default::default()
                }),
                SemanticCompositeKind::AnyOf => super::SchemaView::AnyOf(super::AnyOfSchema {
                    schemas,
                    ..Default::default()
                }),
                SemanticCompositeKind::AllOf => super::SchemaView::AllOf(super::AllOfSchema {
                    schemas,
                    ..Default::default()
                }),
            });
        }
        let Self::Object(object) = self else {
            return None;
        };
        let type_name = match instance_type {
            SchemaType::Null => "null",
            SchemaType::Boolean => "boolean",
            SchemaType::Object => "object",
            SchemaType::Array => "array",
            SchemaType::Number => "number",
            SchemaType::String => "string",
            SchemaType::Integer
                if object.type_assertion.as_ref().is_some_and(|assertion| {
                    assertion
                        .allowed
                        .iter()
                        .any(|allowed| allowed.value == SchemaType::Number)
                        && !assertion
                            .allowed
                            .iter()
                            .any(|allowed| allowed.value == SchemaType::Integer)
                }) =>
            {
                "number"
            }
            SchemaType::Integer => "integer",
        };
        super::SchemaView::new_single(
            type_name,
            &object.source,
            string_formats,
            object.dialect,
            anchor_collector,
            dynamic_anchor_collector,
        )
    }

    /// Projects every TOML-representable instance type admitted by this schema.
    /// This is used when completion has no concrete value from which to choose
    /// a single typed presentation view.
    pub fn completion_projection(
        &self,
        string_formats: Option<&[tombi_x_keyword::StringFormat]>,
    ) -> Option<super::SchemaView> {
        self.completion_projection_with_collectors(string_formats, None, None)
    }

    pub(crate) fn completion_projection_with_collectors(
        &self,
        string_formats: Option<&[tombi_x_keyword::StringFormat]>,
        anchor_collector: Option<&mut super::AnchorCollector>,
        dynamic_anchor_collector: Option<&mut super::DynamicAnchorCollector>,
    ) -> Option<super::SchemaView> {
        let mut anchor_collector = anchor_collector;
        let mut dynamic_anchor_collector = dynamic_anchor_collector;
        let mut schemas = Vec::new();
        for instance_type in [
            SchemaType::Boolean,
            SchemaType::Integer,
            SchemaType::Number,
            SchemaType::String,
            SchemaType::Array,
            SchemaType::Object,
        ] {
            if instance_type == SchemaType::Integer
                && matches!(self, Self::Object(object) if object.type_assertion.as_ref().is_some_and(|assertion| {
                    assertion.allowed.iter().any(|allowed| allowed.value == SchemaType::Number)
                        && !assertion.allowed.iter().any(|allowed| allowed.value == SchemaType::Integer)
                }))
            {
                continue;
            }
            if let Some(value) = self.schema_view_for_type_with_collectors(
                instance_type,
                string_formats,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            ) {
                schemas.push(super::Referable::Resolved {
                    schema_uri: None,
                    value: std::sync::Arc::new(value),
                    semantic_schema: None,
                });
            }
        }
        if schemas.is_empty() && self.accepts_instance_type(SchemaType::Null) {
            return Some(super::SchemaView::Null);
        }
        match schemas.len() {
            // An object schema can be valid while admitting no instance, for
            // example `{ "type": "string", "const": true }` or
            // `{ "enum": [] }`. Keep that semantic result as an explicit
            // false view instead of dropping the schema from its parent.
            0 => Some(super::SchemaView::Nothing(self.range())),
            1 => schemas.into_iter().next().and_then(|schema| match schema {
                super::Referable::Resolved { value, .. } => std::sync::Arc::try_unwrap(value).ok(),
                super::Referable::Ref { .. } => None,
            }),
            // A set of admitted presentation types is inclusive. In
            // particular, a JSON integer is also a JSON number, so projecting
            // this as `oneOf` would incorrectly require exactly one type view
            // to validate.
            _ => Some(super::SchemaView::AnyOf(super::AnyOfSchema {
                schemas: std::sync::Arc::new(tokio::sync::RwLock::new(schemas)),
                ..Default::default()
            })),
        }
    }

    pub fn string_format(&self) -> Option<&str> {
        match self {
            Self::Boolean(_) => None,
            Self::Object(object) => object
                .constraints
                .string
                .format
                .as_ref()
                .map(|format| format.value.as_str())
                .or_else(|| {
                    object
                        .applicators
                        .all_of
                        .iter()
                        .find_map(SemanticSchema::string_format)
                }),
            Self::Composite(composite) => composite
                .schemas
                .iter()
                .find_map(SemanticSchema::string_format),
        }
    }

    pub fn has_applicators(&self) -> bool {
        match self {
            Self::Boolean(_) => false,
            Self::Object(object) => {
                !object.applicators.one_of.is_empty()
                    || !object.applicators.any_of.is_empty()
                    || !object.applicators.all_of.is_empty()
                    || object.applicators.not.is_some()
                    || object.applicators.if_schema.is_some()
            }
            Self::Composite(_) => true,
        }
    }

    pub fn has_references(&self) -> bool {
        match self {
            Self::Boolean(_) => false,
            Self::Composite(composite) => composite.schemas.iter().any(Self::has_references),
            Self::Object(object) => {
                object.references.primary().is_some()
                    || object
                        .applicators
                        .one_of
                        .iter()
                        .chain(&object.applicators.any_of)
                        .chain(&object.applicators.all_of)
                        .any(Self::has_references)
                    || object
                        .constraints
                        .object
                        .properties
                        .values()
                        .chain(object.constraints.object.pattern_properties.values())
                        .any(|property| property.schema.has_references())
                    || object
                        .constraints
                        .array
                        .prefix_items
                        .iter()
                        .any(Self::has_references)
                    || [
                        object.constraints.string.content_schema.as_deref(),
                        object.constraints.array.items.as_deref(),
                        object.constraints.array.additional_items.as_deref(),
                        object.constraints.array.contains.as_deref(),
                        object.constraints.array.unevaluated_items.as_deref(),
                        object.constraints.object.additional_properties.as_deref(),
                        object.constraints.object.unevaluated_properties.as_deref(),
                        object.constraints.object.property_names.as_deref(),
                        object.applicators.not.as_deref(),
                        object.applicators.if_schema.as_deref(),
                        object.applicators.then_schema.as_deref(),
                        object.applicators.else_schema.as_deref(),
                    ]
                    .into_iter()
                    .flatten()
                    .any(Self::has_references)
                    || object
                        .constraints
                        .object
                        .dependent_schemas
                        .values()
                        .any(Self::has_references)
            }
        }
    }

    /// Returns whether a node-local reference also has semantic siblings that
    /// must participate in the typed projection. Reference annotations are
    /// already applied to the resolved view and therefore do not count here.
    pub(crate) fn root_reference_has_projection_siblings(&self, instance_type: SchemaType) -> bool {
        match self {
            Self::Boolean(_) => false,
            Self::Composite(composite) => composite
                .schemas
                .iter()
                .any(|schema| schema.root_reference_has_projection_siblings(instance_type)),
            Self::Object(object) => {
                object.references.primary().is_some()
                    && (object.type_assertion.is_some()
                        || object.assertions.const_value.is_some()
                        || object.assertions.enum_values.is_some()
                        || self.has_direct_constraints_for_type(instance_type)
                        || !object.applicators.all_of.is_empty()
                        || !object.applicators.any_of.is_empty()
                        || !object.applicators.one_of.is_empty()
                        || object.applicators.not.is_some()
                        || object.applicators.if_schema.is_some()
                        || object.applicators.then_schema.is_some()
                        || object.applicators.else_schema.is_some())
            }
        }
    }

    /// Returns values that satisfy exactly one `oneOf` branch when every branch
    /// has a finite `const`/`enum` domain. Keeping the domains per branch until
    /// this point avoids incorrectly suggesting a value shared by two branches.
    pub fn exact_one_of_literal_candidates(&self) -> Option<Vec<Value>> {
        let Self::Object(object) = self else {
            return None;
        };
        if object.applicators.one_of.is_empty() {
            return None;
        }

        let branch_domains = object
            .applicators
            .one_of
            .iter()
            .map(SemanticSchema::finite_literal_candidates)
            .collect::<Option<Vec<_>>>()?;
        let mut candidates = Vec::new();
        for domain in &branch_domains {
            for candidate in domain {
                if candidates.iter().any(|value| value == candidate) {
                    continue;
                }
                let match_count = branch_domains
                    .iter()
                    .filter(|domain| domain.iter().any(|value| value == candidate))
                    .count();
                if match_count == 1 {
                    candidates.push(candidate.clone());
                }
            }
        }
        Some(candidates)
    }

    /// Produces a finite completion domain and filters it through assertions and
    /// type-specific constraints on the containing schema object.
    pub fn finite_literal_candidates(&self) -> Option<Vec<Value>> {
        if let Self::Composite(composite) = self {
            return composite.finite_literal_candidates();
        }
        let Self::Object(object) = self else {
            return None;
        };
        let mut candidates = if let Some(candidates) = self.exact_one_of_literal_candidates() {
            candidates
        } else if let Some(value) = &object.assertions.const_value {
            vec![value.value.clone()]
        } else if let Some(values) = &object.assertions.enum_values {
            values.iter().map(|value| value.value.clone()).collect()
        } else if let Some(values) = object
            .applicators
            .all_of
            .iter()
            .find_map(SemanticSchema::finite_literal_candidates)
        {
            values
        } else if !object.applicators.any_of.is_empty() {
            let domains = object
                .applicators
                .any_of
                .iter()
                .map(SemanticSchema::finite_literal_candidates)
                .collect::<Option<Vec<_>>>()?;
            domains
                .into_iter()
                .flatten()
                .fold(Vec::new(), |mut values, value| {
                    if !values.contains(&value) {
                        values.push(value);
                    }
                    values
                })
        } else {
            return None;
        };
        candidates.retain(|candidate| self.accepts_literal(candidate));
        Some(candidates)
    }

    pub fn accepts_literal(&self, value: &Value) -> bool {
        match self {
            Self::Boolean(schema) => schema.value,
            Self::Object(object) => {
                object.accepts_local_literal(value)
                    && object
                        .applicators
                        .all_of
                        .iter()
                        .all(|schema| schema.accepts_literal(value))
                    && (object.applicators.any_of.is_empty()
                        || object
                            .applicators
                            .any_of
                            .iter()
                            .any(|schema| schema.accepts_literal(value)))
                    && (object.applicators.one_of.is_empty()
                        || object
                            .applicators
                            .one_of
                            .iter()
                            .filter(|schema| schema.accepts_literal(value))
                            .count()
                            == 1)
                    && object
                        .applicators
                        .not
                        .as_deref()
                        .is_none_or(|schema| !schema.accepts_literal(value))
            }
            Self::Composite(composite) => match composite.kind {
                SemanticCompositeKind::OneOf => {
                    composite
                        .schemas
                        .iter()
                        .filter(|schema| schema.accepts_literal(value))
                        .count()
                        == 1
                }
                SemanticCompositeKind::AnyOf => composite
                    .schemas
                    .iter()
                    .any(|schema| schema.accepts_literal(value)),
                SemanticCompositeKind::AllOf => composite
                    .schemas
                    .iter()
                    .all(|schema| schema.accepts_literal(value)),
            },
        }
    }
}

impl SemanticCompositeSchema {
    fn finite_literal_candidates(&self) -> Option<Vec<Value>> {
        let domains = self
            .schemas
            .iter()
            .map(SemanticSchema::finite_literal_candidates)
            .collect::<Vec<_>>();
        let mut candidates = match self.kind {
            SemanticCompositeKind::AllOf => domains.iter().find_map(Clone::clone)?,
            SemanticCompositeKind::AnyOf | SemanticCompositeKind::OneOf => {
                if domains.iter().any(Option::is_none) {
                    return None;
                }
                domains
                    .into_iter()
                    .flatten()
                    .flatten()
                    .fold(Vec::new(), |mut candidates, value| {
                        if !candidates.contains(&value) {
                            candidates.push(value);
                        }
                        candidates
                    })
            }
        };
        candidates.retain(|value| match self.kind {
            SemanticCompositeKind::OneOf => {
                self.schemas
                    .iter()
                    .filter(|schema| schema.accepts_literal(value))
                    .count()
                    == 1
            }
            SemanticCompositeKind::AnyOf => self
                .schemas
                .iter()
                .any(|schema| schema.accepts_literal(value)),
            SemanticCompositeKind::AllOf => self
                .schemas
                .iter()
                .all(|schema| schema.accepts_literal(value)),
        });
        Some(candidates)
    }
}

impl SemanticSchemaObject {
    fn accepts_local_literal(&self, value: &Value) -> bool {
        if let Some(type_assertion) = &self.type_assertion
            && !type_assertion
                .allowed
                .iter()
                .any(|allowed| literal_has_type(value, allowed.value))
        {
            return false;
        }
        if let Some(const_value) = &self.assertions.const_value
            && const_value.value != *value
        {
            return false;
        }
        if let Some(enum_values) = &self.assertions.enum_values
            && !enum_values
                .iter()
                .any(|candidate| candidate.value == *value)
        {
            return false;
        }

        match value {
            Value::Number(number) => {
                let Some(actual) = number.as_f64() else {
                    return true;
                };
                let numeric = &self.constraints.numeric;
                numeric
                    .minimum
                    .as_ref()
                    .and_then(|limit| limit.value.as_f64())
                    .is_none_or(|limit| actual >= limit)
                    && numeric
                        .maximum
                        .as_ref()
                        .and_then(|limit| limit.value.as_f64())
                        .is_none_or(|limit| actual <= limit)
                    && numeric
                        .exclusive_minimum
                        .as_ref()
                        .and_then(|limit| limit.value.as_f64())
                        .is_none_or(|limit| actual > limit)
                    && numeric
                        .exclusive_maximum
                        .as_ref()
                        .and_then(|limit| limit.value.as_f64())
                        .is_none_or(|limit| actual < limit)
                    && numeric
                        .multiple_of
                        .as_ref()
                        .and_then(|multiple| multiple.value.as_f64())
                        .is_none_or(|multiple| {
                            multiple != 0.0
                                && (actual / multiple - (actual / multiple).round()).abs()
                                    <= f64::EPSILON * 8.0
                        })
            }
            Value::String(value) => {
                let length = value.chars().count();
                self.constraints
                    .string
                    .min_length
                    .as_ref()
                    .is_none_or(|minimum| length >= minimum.value)
                    && self
                        .constraints
                        .string
                        .max_length
                        .as_ref()
                        .is_none_or(|maximum| length <= maximum.value)
            }
            _ => true,
        }
    }
}

fn literal_has_type(value: &Value, schema_type: SchemaType) -> bool {
    match schema_type {
        SchemaType::Null => matches!(value, Value::Null),
        SchemaType::Boolean => matches!(value, Value::Bool(_)),
        SchemaType::Object => matches!(value, Value::Object(_)),
        SchemaType::Array => matches!(value, Value::Array(_)),
        SchemaType::Number => matches!(value, Value::Number(_)),
        SchemaType::String => matches!(value, Value::String(_)),
        SchemaType::Integer => matches!(value, Value::Number(number) if number.is_i64()),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticSchemaObject {
    source: ObjectNode,
    dialect: Option<JsonSchemaDialect>,
    pub range: tombi_text::Range,
    pub type_assertion: Option<TypeAssertion>,
    pub assertions: GenericAssertions,
    pub constraints: TypeConstraints,
    pub applicators: SemanticApplicators,
    pub annotations: SemanticAnnotations,
    pub references: References,
}

impl SemanticSchemaObject {
    pub fn new(object: &ObjectNode, dialect: Option<JsonSchemaDialect>) -> Self {
        Self {
            source: object.clone(),
            dialect,
            range: object.range,
            type_assertion: parse_type_assertion(object),
            assertions: GenericAssertions::new(object),
            constraints: TypeConstraints::new(object, dialect),
            applicators: SemanticApplicators::new(object, dialect),
            annotations: SemanticAnnotations::new(object),
            references: References::new(object),
        }
    }
}

fn parse_type_assertion(object: &ObjectNode) -> Option<TypeAssertion> {
    let allowed = match object.get("type") {
        Some(ValueNode::String(value)) => SchemaType::from_str(&value.value)
            .map(|value_type| vec![Spanned::new(value_type, value.range)])
            .unwrap_or_default(),
        Some(ValueNode::Array(values)) => values
            .items
            .iter()
            .filter_map(|value| {
                let ValueNode::String(value) = value else {
                    return None;
                };
                SchemaType::from_str(&value.value)
                    .map(|value_type| Spanned::new(value_type, value.range))
            })
            .collect(),
        _ => return None,
    };

    (!allowed.is_empty()).then_some(TypeAssertion { allowed })
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct GenericAssertions {
    pub const_value: Option<Spanned<Value>>,
    pub enum_values: Option<Vec<Spanned<Value>>>,
}

impl GenericAssertions {
    fn new(object: &ObjectNode) -> Self {
        Self {
            const_value: object
                .get("const")
                .map(|value| Spanned::new(value.into(), value.range())),
            enum_values: object.get("enum").and_then(|value| {
                value.as_array().map(|values| {
                    values
                        .items
                        .iter()
                        .map(|value| Spanned::new(value.into(), value.range()))
                        .collect()
                })
            }),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TypeConstraints {
    pub numeric: NumericConstraints,
    pub string: StringConstraints,
    pub array: ArrayConstraints,
    pub object: ObjectConstraints,
}

impl TypeConstraints {
    fn new(object: &ObjectNode, dialect: Option<JsonSchemaDialect>) -> Self {
        Self {
            numeric: NumericConstraints::new(object),
            string: StringConstraints::new(object, dialect),
            array: ArrayConstraints::new(object, dialect),
            object: ObjectConstraints::new(object, dialect),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct NumericConstraints {
    pub multiple_of: Option<Spanned<Number>>,
    pub minimum: Option<Spanned<Number>>,
    pub maximum: Option<Spanned<Number>>,
    pub exclusive_minimum: Option<Spanned<Number>>,
    pub exclusive_maximum: Option<Spanned<Number>>,
}

impl NumericConstraints {
    fn new(object: &ObjectNode) -> Self {
        Self {
            multiple_of: number_keyword(object, "multipleOf"),
            minimum: number_keyword(object, "minimum"),
            maximum: number_keyword(object, "maximum"),
            exclusive_minimum: number_keyword(object, "exclusiveMinimum"),
            exclusive_maximum: number_keyword(object, "exclusiveMaximum"),
        }
    }
}

fn number_keyword(object: &ObjectNode, keyword: &str) -> Option<Spanned<Number>> {
    let ValueNode::Number(value) = object.get(keyword)? else {
        return None;
    };
    Some(Spanned::new(value.value.clone(), value.range))
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StringConstraints {
    pub min_length: Option<Spanned<usize>>,
    pub max_length: Option<Spanned<usize>>,
    pub pattern: Option<Spanned<String>>,
    pub format: Option<Spanned<String>>,
    pub content_encoding: Option<Spanned<String>>,
    pub content_media_type: Option<Spanned<String>>,
    pub content_schema: Option<Box<SemanticSchema>>,
}

impl StringConstraints {
    fn new(object: &ObjectNode, dialect: Option<JsonSchemaDialect>) -> Self {
        Self {
            min_length: usize_keyword(object, "minLength"),
            max_length: usize_keyword(object, "maxLength"),
            pattern: string_keyword(object, "pattern"),
            format: string_keyword(object, "format"),
            content_encoding: string_keyword(object, "contentEncoding"),
            content_media_type: string_keyword(object, "contentMediaType"),
            content_schema: object
                .get("contentSchema")
                .and_then(|value| SemanticSchema::from_value_node(value, dialect))
                .map(Box::new),
        }
    }
}

fn usize_keyword(object: &ObjectNode, keyword: &str) -> Option<Spanned<usize>> {
    let ValueNode::Number(value) = object.get(keyword)? else {
        return None;
    };
    let value_usize = value.value.as_i64()?.try_into().ok()?;
    Some(Spanned::new(value_usize, value.range))
}

fn string_keyword(object: &ObjectNode, keyword: &str) -> Option<Spanned<String>> {
    let ValueNode::String(value) = object.get(keyword)? else {
        return None;
    };
    Some(Spanned::new(value.value.clone(), value.range))
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ArrayConstraints {
    pub prefix_items: Vec<SemanticSchema>,
    pub items: Option<Box<SemanticSchema>>,
    pub additional_items: Option<Box<SemanticSchema>>,
    pub contains: Option<Box<SemanticSchema>>,
    pub unevaluated_items: Option<Box<SemanticSchema>>,
    pub min_items: Option<Spanned<usize>>,
    pub max_items: Option<Spanned<usize>>,
    pub unique_items: Option<Spanned<bool>>,
    pub min_contains: Option<Spanned<usize>>,
    pub max_contains: Option<Spanned<usize>>,
}

impl ArrayConstraints {
    fn new(object: &ObjectNode, dialect: Option<JsonSchemaDialect>) -> Self {
        let mut prefix_items = Vec::new();
        let mut items = None;
        if let Some(ValueNode::Array(values)) = object.get("items") {
            prefix_items.extend(
                values
                    .items
                    .iter()
                    .filter_map(|value| SemanticSchema::from_value_node(value, dialect)),
            );
        } else {
            items = schema_keyword(object, "items", dialect);
        }
        if let Some(ValueNode::Array(values)) = object.get("prefixItems") {
            prefix_items.extend(
                values
                    .items
                    .iter()
                    .filter_map(|value| SemanticSchema::from_value_node(value, dialect)),
            );
        }

        Self {
            prefix_items,
            items,
            additional_items: schema_keyword(object, "additionalItems", dialect),
            contains: schema_keyword(object, "contains", dialect),
            unevaluated_items: schema_keyword(object, "unevaluatedItems", dialect),
            min_items: usize_keyword(object, "minItems"),
            max_items: usize_keyword(object, "maxItems"),
            unique_items: bool_keyword(object, "uniqueItems"),
            min_contains: usize_keyword(object, "minContains"),
            max_contains: usize_keyword(object, "maxContains"),
        }
    }
}

fn bool_keyword(object: &ObjectNode, keyword: &str) -> Option<Spanned<bool>> {
    let ValueNode::Bool(value) = object.get(keyword)? else {
        return None;
    };
    Some(Spanned::new(value.value, value.range))
}

fn schema_keyword(
    object: &ObjectNode,
    keyword: &str,
    dialect: Option<JsonSchemaDialect>,
) -> Option<Box<SemanticSchema>> {
    object
        .get(keyword)
        .and_then(|value| SemanticSchema::from_value_node(value, dialect))
        .map(Box::new)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticPropertySchema {
    pub name_range: tombi_text::Range,
    pub schema: SemanticSchema,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ObjectConstraints {
    pub properties: tombi_hashmap::IndexMap<String, SemanticPropertySchema>,
    pub pattern_properties: tombi_hashmap::IndexMap<String, SemanticPropertySchema>,
    pub additional_properties: Option<Box<SemanticSchema>>,
    pub unevaluated_properties: Option<Box<SemanticSchema>>,
    pub property_names: Option<Box<SemanticSchema>>,
    pub required: Vec<Spanned<String>>,
    pub dependent_required: tombi_hashmap::IndexMap<String, Vec<Spanned<String>>>,
    pub dependent_schemas: tombi_hashmap::IndexMap<String, SemanticSchema>,
    pub min_properties: Option<Spanned<usize>>,
    pub max_properties: Option<Spanned<usize>>,
}

impl ObjectConstraints {
    fn new(object: &ObjectNode, dialect: Option<JsonSchemaDialect>) -> Self {
        let properties = property_schemas(object, "properties", dialect);
        let pattern_properties = property_schemas(object, "patternProperties", dialect);
        let required = string_array_keyword(object, "required");
        let mut dependent_required = tombi_hashmap::IndexMap::new();
        let mut dependent_schemas = tombi_hashmap::IndexMap::new();

        for keyword in ["dependencies", "dependentRequired", "dependentSchemas"] {
            let Some(ValueNode::Object(dependencies)) = object.get(keyword) else {
                continue;
            };
            for (name, value) in &dependencies.properties {
                if let ValueNode::Array(_) = value {
                    dependent_required
                        .insert(name.value.clone(), string_array(value).unwrap_or_default());
                } else if let Some(schema) = SemanticSchema::from_value_node(value, dialect) {
                    dependent_schemas.insert(name.value.clone(), schema);
                }
            }
        }

        Self {
            properties,
            pattern_properties,
            additional_properties: schema_keyword(object, "additionalProperties", dialect),
            unevaluated_properties: schema_keyword(object, "unevaluatedProperties", dialect),
            property_names: schema_keyword(object, "propertyNames", dialect),
            required,
            dependent_required,
            dependent_schemas,
            min_properties: usize_keyword(object, "minProperties"),
            max_properties: usize_keyword(object, "maxProperties"),
        }
    }
}

fn property_schemas(
    object: &ObjectNode,
    keyword: &str,
    dialect: Option<JsonSchemaDialect>,
) -> tombi_hashmap::IndexMap<String, SemanticPropertySchema> {
    let Some(ValueNode::Object(properties)) = object.get(keyword) else {
        return Default::default();
    };
    properties
        .properties
        .iter()
        .filter_map(|(name, value)| {
            SemanticSchema::from_value_node(value, dialect).map(|schema| {
                (
                    name.value.clone(),
                    SemanticPropertySchema {
                        name_range: name.range,
                        schema,
                    },
                )
            })
        })
        .collect()
}

fn string_array_keyword(object: &ObjectNode, keyword: &str) -> Vec<Spanned<String>> {
    object
        .get(keyword)
        .and_then(string_array)
        .unwrap_or_default()
}

fn string_array(value: &ValueNode) -> Option<Vec<Spanned<String>>> {
    let ValueNode::Array(values) = value else {
        return None;
    };
    Some(
        values
            .items
            .iter()
            .filter_map(|value| {
                let ValueNode::String(value) = value else {
                    return None;
                };
                Some(Spanned::new(value.value.clone(), value.range))
            })
            .collect(),
    )
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SemanticApplicators {
    pub all_of: Vec<SemanticSchema>,
    pub any_of: Vec<SemanticSchema>,
    pub one_of: Vec<SemanticSchema>,
    pub not: Option<Box<SemanticSchema>>,
    pub if_schema: Option<Box<SemanticSchema>>,
    pub then_schema: Option<Box<SemanticSchema>>,
    pub else_schema: Option<Box<SemanticSchema>>,
}

impl SemanticApplicators {
    fn new(object: &ObjectNode, dialect: Option<JsonSchemaDialect>) -> Self {
        Self {
            all_of: schema_array_keyword(object, "allOf", dialect),
            any_of: schema_array_keyword(object, "anyOf", dialect),
            one_of: schema_array_keyword(object, "oneOf", dialect),
            not: schema_keyword(object, "not", dialect),
            if_schema: schema_keyword(object, "if", dialect),
            then_schema: schema_keyword(object, "then", dialect),
            else_schema: schema_keyword(object, "else", dialect),
        }
    }
}

fn schema_array_keyword(
    object: &ObjectNode,
    keyword: &str,
    dialect: Option<JsonSchemaDialect>,
) -> Vec<SemanticSchema> {
    let Some(ValueNode::Array(values)) = object.get(keyword) else {
        return Vec::new();
    };
    values
        .items
        .iter()
        .filter_map(|value| SemanticSchema::from_value_node(value, dialect))
        .collect()
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SemanticAnnotations {
    pub title: Option<Spanned<String>>,
    pub description: Option<Spanned<String>>,
    pub default: Option<Spanned<Value>>,
    pub examples: Vec<Spanned<Value>>,
    pub deprecation: Option<Deprecation>,
}

impl SemanticAnnotations {
    fn new(object: &ObjectNode) -> Self {
        Self {
            title: string_keyword(object, "title"),
            description: string_keyword(object, "description"),
            default: object
                .get("default")
                .map(|value| Spanned::new(value.into(), value.range())),
            examples: object
                .get("examples")
                .and_then(|value| value.as_array())
                .map(|values| {
                    values
                        .items
                        .iter()
                        .map(|value| Spanned::new(value.into(), value.range()))
                        .collect()
                })
                .unwrap_or_default(),
            deprecation: Deprecation::new(object),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct References {
    pub r#ref: Option<Spanned<String>>,
    pub dynamic_ref: Option<Spanned<String>>,
    pub recursive_ref: Option<Spanned<String>>,
}

impl References {
    fn new(object: &ObjectNode) -> Self {
        Self {
            r#ref: string_keyword(object, "$ref"),
            dynamic_ref: string_keyword(object, "$dynamicRef"),
            recursive_ref: string_keyword(object, "$recursiveRef"),
        }
    }

    pub fn primary(&self) -> Option<(ReferenceKind, &Spanned<String>)> {
        self.r#ref
            .as_ref()
            .map(|reference| (ReferenceKind::Ref, reference))
            .or_else(|| {
                self.dynamic_ref
                    .as_ref()
                    .map(|reference| (ReferenceKind::DynamicRef, reference))
            })
            .or_else(|| {
                self.recursive_ref
                    .as_ref()
                    .map(|reference| (ReferenceKind::RecursiveRef, reference))
            })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::SchemaView;

    use super::*;

    fn parse(json: &str) -> SemanticSchemaObject {
        let value = ValueNode::from_str(json).unwrap();
        let object = value.as_object().unwrap();
        SemanticSchemaObject::new(object, Some(JsonSchemaDialect::Draft2019_09))
    }

    #[test]
    fn type_specific_keywords_do_not_create_a_type_assertion() {
        let schema = parse(r#"{"minimum": 0, "minLength": 1, "unevaluatedProperties": false}"#);

        assert!(schema.type_assertion.is_none());
        assert_eq!(
            schema.constraints.numeric.minimum.unwrap().value.as_i64(),
            Some(0)
        );
        assert_eq!(schema.constraints.string.min_length.unwrap().value, 1);
        std::assert_matches!(
            schema.constraints.object.unevaluated_properties.as_deref(),
            Some(SemanticSchema::Boolean(Spanned { value: false, .. }))
        );
    }

    #[test]
    fn type_specific_keywords_only_affect_matching_views() {
        let schema = SemanticSchema::Object(Box::new(parse(
            r#"{"minimum": 10, "properties": {"name": {"type": "string"}}}"#,
        )));

        for instance_type in [
            SchemaType::Null,
            SchemaType::Boolean,
            SchemaType::Integer,
            SchemaType::Number,
            SchemaType::String,
            SchemaType::Array,
            SchemaType::Object,
        ] {
            assert!(schema.accepts_instance_type(instance_type));
        }

        let SchemaView::Float(number) = schema
            .schema_view_for_type(SchemaType::Number, None)
            .unwrap()
        else {
            panic!("number projection must be a float view");
        };
        assert_eq!(number.minimum, Some(10.0));

        std::assert_matches!(
            schema.schema_view_for_type(SchemaType::String, None),
            Some(SchemaView::String(_))
        );
        let SchemaView::Table(object) = schema
            .schema_view_for_type(SchemaType::Object, None)
            .unwrap()
        else {
            panic!("object projection must be a table view");
        };
        assert!(
            object
                .properties
                .blocking_read()
                .contains_key(&crate::SchemaAccessor::Key("name".into()))
        );
    }

    #[test]
    fn root_reference_projection_requires_structural_siblings() {
        let root_reference =
            SemanticSchema::Object(Box::new(parse(r##"{"$ref":"#/$defs/item"}"##)));
        let nested_reference =
            SemanticSchema::Object(Box::new(parse(r##"{"items":{"$ref":"#/$defs/item"}}"##)));

        assert!(root_reference.has_references());
        assert!(!root_reference.root_reference_has_projection_siblings(SchemaType::Object));
        assert!(nested_reference.has_references());
        assert!(!nested_reference.root_reference_has_projection_siblings(SchemaType::Array));

        let sibling_reference = SemanticSchema::Object(Box::new(parse(
            r##"{"$ref":"#/$defs/item","properties":{"local":{"type":"string"}}}"##,
        )));
        assert!(sibling_reference.root_reference_has_projection_siblings(SchemaType::Object));
    }

    #[test]
    fn preserves_same_type_one_of_branches() {
        let schema = parse(
            r#"{
                "oneOf": [
                    {"type": "string", "enum": ["red", "shared"]},
                    {"type": "string", "enum": ["blue", "shared"]}
                ]
            }"#,
        );

        assert_eq!(schema.applicators.one_of.len(), 2);
        for branch in &schema.applicators.one_of {
            let SemanticSchema::Object(branch) = branch else {
                panic!("branch must be an object schema");
            };
            assert_eq!(
                branch.type_assertion.as_ref().unwrap().allowed[0].value,
                SchemaType::String
            );
            assert_eq!(branch.assertions.enum_values.as_ref().unwrap().len(), 2);
        }

        assert_eq!(
            SemanticSchema::Object(Box::new(schema))
                .exact_one_of_literal_candidates()
                .unwrap(),
            vec![Value::String("red".into()), Value::String("blue".into())]
        );
    }

    #[test]
    fn keeps_outer_and_inner_unevaluated_properties_separate() {
        let schema = parse(
            r#"{
                "oneOf": [
                    {
                        "type": "object",
                        "properties": {"uri": {"type": "string"}},
                        "unevaluatedProperties": true
                    }
                ],
                "unevaluatedProperties": false
            }"#,
        );

        std::assert_matches!(
            schema.constraints.object.unevaluated_properties.as_deref(),
            Some(SemanticSchema::Boolean(Spanned { value: false, .. }))
        );
        let SemanticSchema::Object(branch) = &schema.applicators.one_of[0] else {
            panic!("branch must be an object schema");
        };
        std::assert_matches!(
            branch.constraints.object.unevaluated_properties.as_deref(),
            Some(SemanticSchema::Boolean(Spanned { value: true, .. }))
        );
    }

    #[test]
    fn applicators_restrict_completion_instance_types() {
        let schema = SemanticSchema::Object(Box::new(parse(
            r#"{"oneOf":[{"type":"string"},{"type":"object"}]}"#,
        )));
        assert!(schema.accepts_instance_type(SchemaType::String));
        assert!(schema.accepts_instance_type(SchemaType::Object));
        assert!(!schema.accepts_instance_type(SchemaType::Boolean));
        assert!(!schema.accepts_instance_type(SchemaType::Integer));
    }
}
