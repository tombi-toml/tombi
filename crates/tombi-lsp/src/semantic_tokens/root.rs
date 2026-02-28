use tombi_ast::DanglingCommentGroupOr;

use super::{AppendSemanticTokens, SemanticTokensBuilder};

impl AppendSemanticTokens for tombi_ast::Root {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        let source_path = builder.text_document_uri.to_file_path().ok();
        let schema_document_directive =
            self.schema_document_comment_directive(source_path.as_deref());
        for comment_group in self.dangling_comment_groups() {
            for comment in comment_group.comments() {
                if let Some(schema_document_directive) = &schema_document_directive
                    && comment
                        .syntax()
                        .range()
                        .contains(schema_document_directive.directive_range.start)
                {
                    builder
                        .add_comment_directive(&comment, schema_document_directive.directive_range);
                    continue;
                }
                if let Some(tombi_document_directive) = comment.get_tombi_document_directive() {
                    builder
                        .add_comment_directive(&comment, tombi_document_directive.directive_range);
                } else {
                    comment.append_semantic_tokens(builder);
                }
            }
        }

        for group in self.key_value_groups() {
            match group {
                DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                    for comment in comment_group.comments() {
                        comment.append_semantic_tokens(builder);
                    }
                }
                DanglingCommentGroupOr::ItemGroup(key_value_group) => {
                    for key_value in key_value_group.key_values() {
                        key_value.append_semantic_tokens(builder);
                    }
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
