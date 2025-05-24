use itertools::Itertools;
use tombi_ast::AstToken;

use super::{AppendSemanticTokens, SemanticTokensBuilder, TokenType};

impl AppendSemanticTokens for tombi_ast::Root {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        let key_values = self.key_values().collect_vec();

        if key_values.is_empty() {
            for comments in self.key_values_dangling_comments() {
                for comment in comments {
                    if let Some(file_schema_range) = builder.file_schema_range {
                        if comment.syntax().range().contains(file_schema_range.start) {
                            builder.add_schema_url_comment(comment, &file_schema_range);
                            continue;
                        }
                    }
                    builder.add_token(TokenType::COMMENT, comment.as_ref().syntax().clone().into());
                }
            }
        } else {
            for comments in self.key_values_begin_dangling_comments() {
                for comment in comments {
                    if let Some(file_schema_range) = builder.file_schema_range {
                        if comment.syntax().range().contains(file_schema_range.start) {
                            builder.add_schema_url_comment(comment, &file_schema_range);
                            continue;
                        }
                    }
                    builder.add_token(TokenType::COMMENT, comment.as_ref().syntax().clone().into());
                }
            }

            for key_value in self.key_values() {
                key_value.append_semantic_tokens(builder);
            }

            for comments in self.key_values_end_dangling_comments() {
                for comment in comments {
                    builder.add_token(TokenType::COMMENT, comment.as_ref().syntax().clone().into());
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
