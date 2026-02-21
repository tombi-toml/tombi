use crate::{ArrayOfTable, AstNode, Keys, Table};

#[derive(Debug, Clone)]
pub enum TableOrArrayOfTable {
    Table(Table),
    ArrayOfTable(ArrayOfTable),
}

impl TableOrArrayOfTable {
    pub fn header(&self) -> Option<Keys> {
        match self {
            Self::Table(table) => table.header(),
            Self::ArrayOfTable(array_of_table) => array_of_table.header(),
        }
    }

    pub fn range(&self) -> tombi_text::Range {
        match self {
            Self::Table(table) => table.range(),
            Self::ArrayOfTable(array_of_table) => array_of_table.range(),
        }
    }
}

impl AstNode for TableOrArrayOfTable {
    #[inline]
    fn can_cast(kind: tombi_syntax::SyntaxKind) -> bool {
        Table::can_cast(kind) || ArrayOfTable::can_cast(kind)
    }

    #[inline]
    fn cast(syntax: tombi_syntax::SyntaxNode) -> Option<Self> {
        if Table::can_cast(syntax.kind()) {
            Some(TableOrArrayOfTable::Table(Table { syntax }))
        } else if ArrayOfTable::can_cast(syntax.kind()) {
            Some(TableOrArrayOfTable::ArrayOfTable(ArrayOfTable { syntax }))
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &tombi_syntax::SyntaxNode {
        match self {
            TableOrArrayOfTable::Table(table) => table.syntax(),
            TableOrArrayOfTable::ArrayOfTable(array_of_table) => array_of_table.syntax(),
        }
    }
}
