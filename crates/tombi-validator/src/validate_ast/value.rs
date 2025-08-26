use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;

use crate::validate_ast::{Validate, ValueImpl};

impl Validate for tombi_ast::Value {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            match self {
                tombi_ast::Value::Array(array) => {
                    array
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::BasicString(string) => {
                    string
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::Boolean(boolean) => {
                    boolean
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::Float(float) => {
                    float
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::InlineTable(inline_table) => {
                    inline_table
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::IntegerBin(integer) => {
                    integer
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::IntegerDec(integer) => {
                    integer
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::IntegerHex(integer) => {
                    integer
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::IntegerOct(integer) => {
                    integer
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::LiteralString(string) => {
                    string
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::LocalDate(local_date) => {
                    local_date
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::LocalDateTime(local_date_time) => {
                    local_date_time
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::LocalTime(local_time) => {
                    local_time
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::MultiLineBasicString(string) => {
                    string
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::MultiLineLiteralString(string) => {
                    string
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
                tombi_ast::Value::OffsetDateTime(offset_date_time) => {
                    offset_date_time
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                }
            }
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::Value {
    fn value_type(&self) -> ValueType {
        match self {
            tombi_ast::Value::Array(_) => ValueType::Array,
            tombi_ast::Value::BasicString(_)
            | tombi_ast::Value::LiteralString(_)
            | tombi_ast::Value::MultiLineBasicString(_)
            | tombi_ast::Value::MultiLineLiteralString(_) => ValueType::String,
            tombi_ast::Value::Boolean(_) => ValueType::Boolean,
            tombi_ast::Value::Float(_) => ValueType::Float,
            tombi_ast::Value::InlineTable(_) => ValueType::Table,
            tombi_ast::Value::IntegerBin(_)
            | tombi_ast::Value::IntegerDec(_)
            | tombi_ast::Value::IntegerHex(_)
            | tombi_ast::Value::IntegerOct(_) => ValueType::Integer,
            tombi_ast::Value::LocalDate(_) => ValueType::LocalDate,
            tombi_ast::Value::LocalDateTime(_) => ValueType::LocalDateTime,
            tombi_ast::Value::LocalTime(_) => ValueType::LocalTime,
            tombi_ast::Value::OffsetDateTime(_) => ValueType::OffsetDateTime,
        }
    }

    fn range(&self) -> tombi_text::Range {
        match self {
            tombi_ast::Value::Array(v) => v.range(),
            tombi_ast::Value::BasicString(v) => v.range(),
            tombi_ast::Value::Boolean(v) => v.range(),
            tombi_ast::Value::Float(v) => v.range(),
            tombi_ast::Value::InlineTable(v) => v.range(),
            tombi_ast::Value::IntegerBin(v) => v.range(),
            tombi_ast::Value::IntegerDec(v) => v.range(),
            tombi_ast::Value::IntegerHex(v) => v.range(),
            tombi_ast::Value::IntegerOct(v) => v.range(),
            tombi_ast::Value::LiteralString(v) => v.range(),
            tombi_ast::Value::LocalDate(v) => v.range(),
            tombi_ast::Value::LocalDateTime(v) => v.range(),
            tombi_ast::Value::LocalTime(v) => v.range(),
            tombi_ast::Value::MultiLineBasicString(v) => v.range(),
            tombi_ast::Value::MultiLineLiteralString(v) => v.range(),
            tombi_ast::Value::OffsetDateTime(v) => v.range(),
        }
    }
}
