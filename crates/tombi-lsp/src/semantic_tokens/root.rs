use itertools::Itertools;

use super::{AppendSemanticTokens, SemanticTokensBuilder};

impl AppendSemanticTokens for tombi_ast::Root {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        let key_values = self.key_values().collect_vec();

        let source_path = builder.text_document_uri.to_file_path().ok();
        let schema_document_directive =
            self.schema_document_comment_directive(source_path.as_deref());
        if key_values.is_empty() {
            for comments in self.key_values_dangling_comments() {
                for comment in comments {
                    if let Some(schema_comment_directive) = &schema_document_directive {
                        if comment
                            .syntax()
                            .range()
                            .contains(schema_comment_directive.directive_range.start)
                        {
                            builder.add_comment_directive(
                                &comment,
                                schema_comment_directive.directive_range,
                            );
                            continue;
                        }
                    }
                    if let Some(tombi_comment_directive) = comment.get_tombi_document_directive() {
                        builder.add_comment_directive(
                            &comment,
                            tombi_comment_directive.directive_range,
                        );
                    } else {
                        comment.append_semantic_tokens(builder);
                    }
                }
            }
        } else {
            for comments in self.key_values_begin_dangling_comments() {
                for comment in comments {
                    if let Some(schema_document_directive) = &schema_document_directive {
                        if comment
                            .syntax()
                            .range()
                            .contains(schema_document_directive.directive_range.start)
                        {
                            builder.add_comment_directive(
                                &comment,
                                schema_document_directive.directive_range,
                            );
                            continue;
                        }
                    }
                    if let Some(tombi_document_directive) = comment.get_tombi_document_directive() {
                        builder.add_comment_directive(
                            &comment,
                            tombi_document_directive.directive_range,
                        );
                    } else {
                        comment.append_semantic_tokens(builder);
                    }
                }
            }

            for key_value in self.key_values() {
                key_value.append_semantic_tokens(builder);
            }

            for comments in self.key_values_end_dangling_comments() {
                for comment in comments {
                    comment.append_semantic_tokens(builder);
                }
            }
        }

        for table_or_array_of_table in self.table_or_array_of_tables() {
            table_or_array_of_table.append_semantic_tokens(builder)
        }
    }
}

impl AppendSemanticTokens for tombi_ast::TableOrArrayOfTable {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        match self {
            Self::Table(table) => table.append_semantic_tokens(builder),
            Self::ArrayOfTable(array_of_table) => array_of_table.append_semantic_tokens(builder),
        }
    }
}
