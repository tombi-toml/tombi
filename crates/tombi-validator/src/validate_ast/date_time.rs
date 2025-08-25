use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};

use crate::Validate;

impl Validate for tombi_ast::LocalDate {
    fn validate<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::SchemaAccessor],
        _current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
        _comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            // Basic date validation - always valid for now
            // Schema validation can be added here later if needed
            Ok(())
        }
        .boxed()
    }
}

impl Validate for tombi_ast::LocalDateTime {
    fn validate<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::SchemaAccessor],
        _current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
        _comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            // Basic date time validation - always valid for now
            // Schema validation can be added here later if needed
            Ok(())
        }
        .boxed()
    }
}

impl Validate for tombi_ast::LocalTime {
    fn validate<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::SchemaAccessor],
        _current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
        _comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            // Basic time validation - always valid for now
            // Schema validation can be added here later if needed
            Ok(())
        }
        .boxed()
    }
}

impl Validate for tombi_ast::OffsetDateTime {
    fn validate<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::SchemaAccessor],
        _current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
        _comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            // Basic date time validation - always valid for now
            // Schema validation can be added here later if needed
            Ok(())
        }
        .boxed()
    }
}
