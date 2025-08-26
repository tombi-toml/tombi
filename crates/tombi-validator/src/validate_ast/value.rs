use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};

use crate::validate_ast::Validate;

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
