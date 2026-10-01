use std::collections::{HashMap, HashSet};

use tombi_toml_version::TomlVersion;

use crate::{AstNode, TableOrArrayOfTable};

/// Per-header results that used to be computed by walking over the sibling headers before or
/// after each header, which made document conversion `O(headers^2)` (or worse).
#[derive(Debug, Clone, Default)]
pub(crate) struct HeaderInfo {
    /// For each key-prefix of the header (prefix length `i + 1` at index `i`), the number of
    /// preceding `[[array_of_tables]]` headers that equal that prefix.
    pub(crate) array_of_tables_counts: Vec<usize>,
    /// The number of distinct shorter key-prefixes of the header that were already declared by
    /// a preceding `[table]` / `[[array_of_tables]]` header.
    pub(crate) parent_header_count: usize,
    /// The last of the headers that are nested under this header.
    pub(crate) last_sub_table: Option<u32>,
}

/// Built once per tree with a single forward pass over the root headers, so querying a header
/// costs `O(header depth)`.
///
/// Keys are always compared by their content in the latest TOML version.
#[derive(Debug, Default)]
pub(crate) struct HeaderIndex {
    headers: HashMap<u32, HeaderInfo>,
}

/// Interns key paths, so that a key path is compared and hashed as a single integer instead of
/// being re-hashed key by key (which would make a deeply nested header `O(depth^2)`).
#[derive(Default)]
struct KeyPathInterner {
    children: HashMap<u32, HashMap<String, u32>>,
    next_id: u32,
}

impl KeyPathInterner {
    /// Returns the id of the key path made of `parent` followed by `key`.
    fn intern(&mut self, parent: u32, key: &str) -> u32 {
        let children = self.children.entry(parent).or_default();
        if let Some(id) = children.get(key) {
            return *id;
        }
        // Id 0 is the empty key path.
        self.next_id += 1;
        children.insert(key.to_owned(), self.next_id);
        self.next_id
    }
}

impl HeaderIndex {
    pub(crate) fn build(root: &tombi_ast_syntax::SyntaxNode) -> Self {
        let toml_version = TomlVersion::latest();
        let mut headers: HashMap<u32, HeaderInfo> = HashMap::new();
        let mut interner = KeyPathInterner::default();

        // The trailing run of headers sharing the same first key. A header with a different
        // (or unconvertible) first key starts a new run.
        let mut run_first_key: Option<u32> = None;
        let mut run_headers: HashSet<u32> = HashSet::new();
        // The same, restricted to `[[...]]` headers, which `[table]` headers never interrupt.
        let mut aot_run_first_key: Option<u32> = None;
        let mut aot_run_counts: HashMap<u32, usize> = HashMap::new();
        // Headers whose sub tables are still being collected, from outermost to innermost, as
        // `(node id, key path id, key count)`.
        let mut open_headers: Vec<(u32, u32, usize)> = vec![];
        let mut prev_header: Option<u32> = None;

        // Closes `open_headers` entries: every header seen since an entry was opened is nested
        // under it, so the previous header is its last sub table.
        fn close(
            headers: &mut HashMap<u32, HeaderInfo>,
            open_headers: &mut Vec<(u32, u32, usize)>,
            prev_header: Option<u32>,
            keep: impl Fn(u32, usize) -> bool,
        ) {
            while let Some((id, path_id, key_count)) = open_headers.last() {
                if keep(*path_id, *key_count) {
                    break;
                }
                if prev_header != Some(*id)
                    && let Some(info) = headers.get_mut(id)
                {
                    info.last_sub_table = prev_header;
                }
                open_headers.pop();
            }
        }

        for node in root.child_nodes() {
            let Some(item) = TableOrArrayOfTable::cast(node) else {
                continue;
            };
            let is_array_of_table = matches!(item, TableOrArrayOfTable::ArrayOfTable(_));
            let Some(header) = item.header() else {
                close(&mut headers, &mut open_headers, prev_header, |_, _| false);
                prev_header = Some(node.id());
                continue;
            };

            // Key path ids of the leading keys that can be converted: `path_ids[i]` is the id of
            // the first `i + 1` keys.
            let mut path_ids: Vec<u32> = Vec::new();
            let mut key_count = 0;
            for key in header.keys() {
                key_count += 1;
                if path_ids.len() + 1 == key_count
                    && let Ok(content) = key.try_to_content(toml_version)
                {
                    let parent = path_ids.last().copied().unwrap_or_default();
                    path_ids.push(interner.intern(parent, &content));
                }
            }
            let is_complete = key_count > 0 && path_ids.len() == key_count;
            let first = path_ids.first().copied();

            let same_run = first.is_some() && first == run_first_key;
            let same_aot_run = first.is_some() && first == aot_run_first_key;

            let mut array_of_tables_counts = vec![0; key_count];
            if same_aot_run {
                for (count, path_id) in array_of_tables_counts.iter_mut().zip(&path_ids) {
                    *count = aot_run_counts.get(path_id).copied().unwrap_or_default();
                }
            }
            let parent_header_count = if same_run {
                // Only the strictly shorter key paths are parents.
                path_ids[..path_ids.len().min(key_count - 1)]
                    .iter()
                    .filter(|path_id| run_headers.contains(path_id))
                    .count()
            } else {
                0
            };
            headers.insert(
                node.id(),
                HeaderInfo {
                    array_of_tables_counts,
                    parent_header_count,
                    last_sub_table: None,
                },
            );

            // Sub tables are the following headers that this header is a strict prefix of.
            close(
                &mut headers,
                &mut open_headers,
                prev_header,
                |open_path_id, open_key_count| {
                    // Only the keys up to the open header's length have to be convertible.
                    open_key_count < key_count
                        && path_ids.len() >= open_key_count
                        && path_ids[open_key_count - 1] == open_path_id
                },
            );
            prev_header = Some(node.id());

            if !same_run {
                run_first_key = first;
                run_headers.clear();
            }
            if is_complete {
                run_headers.insert(path_ids[key_count - 1]);
            }

            if is_array_of_table {
                if let Some(first) = first {
                    if !same_aot_run {
                        aot_run_first_key = Some(first);
                        aot_run_counts.clear();
                    }
                    if is_complete {
                        *aot_run_counts.entry(path_ids[key_count - 1]).or_default() += 1;
                    }
                } else {
                    aot_run_first_key = None;
                    aot_run_counts.clear();
                }
            }

            if is_complete {
                open_headers.push((node.id(), path_ids[key_count - 1], key_count));
            }
        }
        close(&mut headers, &mut open_headers, prev_header, |_, _| false);

        Self { headers }
    }

    pub(crate) fn get(&self, node: &tombi_ast_syntax::SyntaxNode) -> HeaderInfo {
        self.headers.get(&node.id()).cloned().unwrap_or_default()
    }
}

pub(crate) fn header_info(node: &tombi_ast_syntax::SyntaxNode) -> HeaderInfo {
    let root = node.ancestors().last().unwrap_or(*node);
    root.header_index().get(node)
}
