# Document Tree

This crate is a library for representing the tree structure of documents using AST.

```text
ast::Root -> tombi_document_tree::DocumentTree -> tombi_document::Document
```

In the process of converting to tombi_document_tree::DocumentTree,
syntax errors such as duplicate keys and different types of data assigned to the same key are detected.

The information of the original AST is lost when converted to a DocumentTree, and only the necessary information for data is left.
