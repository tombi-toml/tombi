use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};

use crate::Validate;

impl Validate for tombi_ast::Root {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = Vec::new();

            // Validate key-values in root
            for key_value in self.key_values() {
                if let Err(mut errs) = key_value
                    .validate(accessors, current_schema, schema_context, comment_context)
                    .await
                {
                    diagnostics.append(&mut errs);
                }
            }

            // Validate tables and array of tables
            for table_or_array in self.table_or_array_of_tables() {
                match table_or_array {
                    tombi_ast::TableOrArrayOfTable::Table(table) => {
                        if let Err(mut errs) = table
                            .validate(accessors, current_schema, schema_context, comment_context)
                            .await
                        {
                            diagnostics.append(&mut errs);
                        }
                    }
                    tombi_ast::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                        if let Err(mut errs) = array_of_table
                            .validate(accessors, current_schema, schema_context, comment_context)
                            .await
                        {
                            diagnostics.append(&mut errs);
                        }
                    }
                }
            }

            if diagnostics.is_empty() {
                Ok(())
            } else {
                Err(diagnostics)
            }
        }
        .boxed()
    }
}
