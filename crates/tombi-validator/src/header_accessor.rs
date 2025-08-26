use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HeaderAccessor {
    Key(tombi_ast::Key),
    Index {
        index: usize,
        range: tombi_text::Range,
    },
}

impl HeaderAccessor {
    pub fn range(&self) -> tombi_text::Range {
        match self {
            HeaderAccessor::Key(key) => key.range(),
            HeaderAccessor::Index { range, .. } => *range,
        }
    }
}

impl From<tombi_ast::Key> for HeaderAccessor {
    fn from(key: tombi_ast::Key) -> Self {
        HeaderAccessor::Key(key)
    }
}

pub trait GetHeaderAccessors {
    fn get_header_accessor(&self) -> Option<Vec<HeaderAccessor>>;
}

impl GetHeaderAccessors for tombi_ast::Table {
    fn get_header_accessor(&self) -> Option<Vec<HeaderAccessor>> {
        let array_of_tables_keys = self
            .array_of_tables_keys()
            .map(|keys| keys.into_iter().collect_vec())
            .counts();

        let mut header_keys = vec![];
        let mut accessors = vec![];
        for key in self.header()?.keys() {
            let range = key.range();
            header_keys.push(key.clone());
            accessors.push(HeaderAccessor::Key(key));

            if let Some(new_index) = array_of_tables_keys.get(&header_keys) {
                accessors.push(HeaderAccessor::Index {
                    index: *new_index,
                    range,
                });
            }
        }

        Some(accessors)
    }
}

impl GetHeaderAccessors for tombi_ast::ArrayOfTable {
    fn get_header_accessor(&self) -> Option<Vec<HeaderAccessor>> {
        let array_of_tables_keys = self
            .array_of_tables_keys()
            .map(|keys| keys.into_iter().collect_vec())
            .counts();

        let mut header_keys = vec![];
        let mut accessors = vec![];
        for key in self.header()?.keys() {
            let range = key.range();
            header_keys.push(key.clone());
            accessors.push(HeaderAccessor::Key(key));

            if let Some(new_index) = array_of_tables_keys.get(&header_keys) {
                accessors.push(HeaderAccessor::Index {
                    index: *new_index,
                    range,
                });
            }
        }

        accessors.push(HeaderAccessor::Index {
            index: *array_of_tables_keys.get(&header_keys).unwrap_or(&0),
            range: self.header()?.range(),
        });

        Some(accessors)
    }
}
