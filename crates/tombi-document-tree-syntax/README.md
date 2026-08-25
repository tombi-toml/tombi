# Document Tree

This crate provides Tombi's source-backed implementation of the semantic document-tree API.

```text
tombi_ast_syntax::Root -> tombi_document_tree_syntax::DocumentTree -> tombi_document::Document
```

In the process of converting to tombi_document_tree_syntax::DocumentTree,
syntax errors such as duplicate keys and different types of data assigned to the same key are detected.

The AST structure is not exposed by `DocumentTree`; semantic values, source-backed text, and ranges are retained.
