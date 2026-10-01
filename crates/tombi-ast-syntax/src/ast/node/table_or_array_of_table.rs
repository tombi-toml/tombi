use crate::{ArrayOfTable, AstNode, Keys, Table};

#[derive(Debug, Clone)]
pub enum TableOrArrayOfTable<'t> {
    Table(Table<'t>),
    ArrayOfTable(ArrayOfTable<'t>),
}

impl<'t> TableOrArrayOfTable<'t> {
    pub fn header(&self) -> Option<Keys<'t>> {
        match self {
            Self::Table(table) => table.header(),
            Self::ArrayOfTable(array_of_table) => array_of_table.header(),
        }
    }

    pub fn span(&self) -> tombi_text::Span {
        match self {
            Self::Table(table) => table.span(),
            Self::ArrayOfTable(array_of_table) => array_of_table.span(),
        }
    }
}

impl<'t> AstNode<'t> for TableOrArrayOfTable<'t> {
    #[inline]
    fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool {
        Table::can_cast(kind) || ArrayOfTable::can_cast(kind)
    }

    #[inline]
    fn cast(syntax: tombi_ast_syntax::SyntaxNode<'t>) -> Option<Self> {
        if Table::can_cast(syntax.kind()) {
            Some(TableOrArrayOfTable::Table(Table { syntax }))
        } else if ArrayOfTable::can_cast(syntax.kind()) {
            Some(TableOrArrayOfTable::ArrayOfTable(ArrayOfTable { syntax }))
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &tombi_ast_syntax::SyntaxNode<'t> {
        match self {
            TableOrArrayOfTable::Table(table) => table.syntax(),
            TableOrArrayOfTable::ArrayOfTable(array_of_table) => array_of_table.syntax(),
        }
    }
}
