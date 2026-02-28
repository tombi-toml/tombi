use crate::{AstNode, KeyValue, Value};

#[derive(Debug, Clone)]
pub enum ValueOrKeyValue {
    Value(Value),
    KeyValue(KeyValue),
}

impl ValueOrKeyValue {
    pub fn range(&self) -> tombi_text::Range {
        match self {
            ValueOrKeyValue::Value(value) => value.range(),
            ValueOrKeyValue::KeyValue(key) => key.syntax().range(),
        }
    }
}

impl AstNode for ValueOrKeyValue {
    #[inline]
    fn can_cast(kind: tombi_syntax::SyntaxKind) -> bool {
        Value::can_cast(kind) || KeyValue::can_cast(kind)
    }

    #[inline]
    fn cast(syntax: tombi_syntax::SyntaxNode) -> Option<Self> {
        if Value::can_cast(syntax.kind()) {
            Value::cast(syntax).map(ValueOrKeyValue::Value)
        } else if KeyValue::can_cast(syntax.kind()) {
            Some(ValueOrKeyValue::KeyValue(KeyValue { syntax }))
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &tombi_syntax::SyntaxNode {
        match self {
            ValueOrKeyValue::Value(value) => value.syntax(),
            ValueOrKeyValue::KeyValue(key) => key.syntax(),
        }
    }
}
