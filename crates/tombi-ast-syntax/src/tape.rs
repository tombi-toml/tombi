use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{Direction, NodeOrToken, SyntaxKind, TokenAtOffset, WalkEvent};

const ROOT_PARENT: u32 = u32::MAX;
const DECODE_ERROR_SPAN: tombi_text::Span = tombi_text::Span::MAX;

/// Decoded TOML text for exactly one TOML version.
#[doc(hidden)]
pub struct DecodedTextResolver {
    version: tombi_toml_version::TomlVersion,
    source: Arc<Box<str>>,
    decoded: Arc<Box<str>>,
    token_ids: Box<[u32]>,
    spans: Box<[tombi_text::Span]>,
    errors: Box<[(u32, tombi_toml_text::ParseError)]>,
}

impl DecodedTextResolver {
    fn decoded_index(&self, token_id: u32) -> usize {
        self.token_ids
            .binary_search(&token_id)
            .expect("escaped basic string must be present in the decoded text storage")
    }

    #[inline]
    fn decoded_span(&self, index: usize) -> Result<tombi_text::Span, tombi_toml_text::ParseError> {
        let span = self.spans[index];
        if span == DECODE_ERROR_SPAN {
            let error_index = self
                .errors
                .binary_search_by_key(&(index as u32), |(index, _)| *index)
                .expect("decoded text error must be present");
            Err(self.errors[error_index].1.clone())
        } else {
            Ok(span)
        }
    }

    fn resolve(
        &self,
        tree: &Tree,
        token_id: u32,
    ) -> Result<(Arc<Box<str>>, tombi_text::Span), tombi_toml_text::ParseError> {
        debug_assert!(Arc::ptr_eq(&self.source, tree.source_arc()));
        let entry = tree.entry(token_id);
        if entry.needs_decode() {
            let index = self.decoded_index(token_id);
            let span = self.decoded_span(index)?;
            return Ok((Arc::clone(&self.decoded), span));
        }

        let content = tree.try_to_token_content(token_id, self.version)?;
        let Cow::Borrowed(content) = content else {
            unreachable!("escaped basic strings must use decoded text storage")
        };
        let source_start = self.source.as_ptr() as usize;
        let start = content.as_ptr() as usize - source_start;
        Ok((
            Arc::clone(&self.source),
            tombi_text::Span::new(
                (start as u32).into(),
                ((start + content.len()) as u32).into(),
            ),
        ))
    }

    fn resolve_raw(&self, tree: &Tree, token_id: u32) -> (Arc<Box<str>>, tombi_text::Span) {
        debug_assert!(Arc::ptr_eq(&self.source, tree.source_arc()));
        (Arc::clone(&self.source), tree.entry(token_id).span)
    }
}

fn decode_escaped_basic_strings(
    tree: &Tree,
    version: tombi_toml_version::TomlVersion,
) -> DecodedTextResolver {
    let mut token_ids = Vec::new();
    let mut capacity = 0;
    for (token_id, entry) in tree.entries.iter().enumerate() {
        if entry.needs_decode() {
            token_ids.push(token_id as u32);
            capacity += entry.span.len() as usize;
        }
    }

    let mut decoded = String::with_capacity(capacity);
    let mut spans = Vec::with_capacity(token_ids.len());
    let mut errors = Vec::new();
    for &token_id in &token_ids {
        let entry = tree.entry(token_id);
        let text = &tree.source()[entry.span];
        let content = match entry.kind {
            SyntaxKind::BASIC_STRING => tombi_toml_text::try_from_basic_string(text, version),
            SyntaxKind::MULTI_LINE_BASIC_STRING => {
                tombi_toml_text::try_from_multi_line_basic_string(text, version)
            }
            _ => unreachable!(),
        };

        match content {
            Ok(Cow::Owned(content)) => {
                let start = tombi_text::Offset::of(&decoded);
                decoded.push_str(&content);
                spans.push(tombi_text::Span::new(
                    start,
                    tombi_text::Offset::of(&decoded),
                ));
            }
            Ok(Cow::Borrowed(_)) => {
                unreachable!("source-backed string content must not enter the decoded text pool")
            }
            Err(error) => {
                errors.push((spans.len() as u32, error));
                spans.push(DECODE_ERROR_SPAN);
            }
        }
    }

    DecodedTextResolver {
        version,
        source: Arc::clone(tree.source_arc()),
        decoded: Arc::new(decoded.into_boxed_str()),
        token_ids: token_ids.into_boxed_slice(),
        spans: spans.into_boxed_slice(),
        errors: errors.into_boxed_slice(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
enum EntryTag {
    Node,
    Token,
    TokenNeedsDecode,
}

impl EntryTag {
    const fn token(needs_decode: bool) -> Self {
        if needs_decode {
            Self::TokenNeedsDecode
        } else {
            Self::Token
        }
    }
}

/// A compact preorder entry. Nodes and tokens share the same tape.
///
/// `subtree_end` is exclusive. For tokens it is always `id + 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Entry {
    span: tombi_text::Span,
    parent: u32,
    subtree_end: u32,
    prev_sibling: u32,
    kind: SyntaxKind,
    tag: EntryTag,
}

impl Entry {
    #[inline]
    const fn is_token(self) -> bool {
        matches!(self.tag, EntryTag::Token | EntryTag::TokenNeedsDecode)
    }

    #[inline]
    const fn needs_decode(self) -> bool {
        matches!(self.tag, EntryTag::TokenNeedsDecode)
    }
}

#[derive(Debug)]
struct Tree {
    line_index: Arc<tombi_text::LineIndex>,
    entries: Box<[Entry]>,
    token_ids: Box<[u32]>,
    header_index: std::sync::OnceLock<crate::ast::HeaderIndex>,
}

impl Tree {
    #[inline]
    fn entry(&self, id: u32) -> Entry {
        self.entries[id as usize]
    }

    #[inline]
    fn source(&self) -> &str {
        self.line_index.text()
    }

    #[inline]
    fn source_arc(&self) -> &Arc<Box<str>> {
        self.line_index.text_arc()
    }

    fn text_token_id(&self, id: u32) -> u32 {
        let node_entry = self.entry(id);
        if node_entry.is_token() {
            id
        } else {
            (id + 1..node_entry.subtree_end)
                .find(|child_id| {
                    let child = self.entry(*child_id);
                    child.is_token() && child.kind == node_entry.kind
                })
                .expect("TOML text node must contain its token")
        }
    }

    fn try_to_content(
        &self,
        id: u32,
        version: tombi_toml_version::TomlVersion,
    ) -> Result<Cow<'_, str>, tombi_toml_text::ParseError> {
        let id = self.text_token_id(id);
        self.try_to_token_content(id, version)
    }

    fn try_to_token_content(
        &self,
        token_id: u32,
        version: tombi_toml_version::TomlVersion,
    ) -> Result<Cow<'_, str>, tombi_toml_text::ParseError> {
        let entry = self.entry(token_id);
        debug_assert!(entry.is_token());
        let text = &self.source()[entry.span];
        match entry.kind {
            SyntaxKind::BARE_KEY => tombi_toml_text::try_from_bare_key(text),
            SyntaxKind::BASIC_STRING => tombi_toml_text::try_from_basic_string(text, version),
            SyntaxKind::MULTI_LINE_BASIC_STRING => {
                tombi_toml_text::try_from_multi_line_basic_string(text, version)
            }
            SyntaxKind::LITERAL_STRING => tombi_toml_text::try_from_literal_string(text),
            SyntaxKind::MULTI_LINE_LITERAL_STRING => {
                tombi_toml_text::try_from_multi_line_literal_string(text)
            }
            kind => unreachable!("{kind:?} does not contain TOML text"),
        }
    }

    fn decoded_text_resolver(
        &self,
        version: tombi_toml_version::TomlVersion,
    ) -> DecodedTextResolver {
        decode_escaped_basic_strings(self, version)
    }

    #[inline]
    fn element(self: &Arc<Self>, id: u32) -> SyntaxElement {
        if self.entry(id).is_token() {
            NodeOrToken::Token(SyntaxToken::new(Arc::clone(self), id))
        } else {
            NodeOrToken::Node(SyntaxNode::new(Arc::clone(self), id))
        }
    }
}

/// Direct builder for a source-backed preorder syntax tape.
#[derive(Debug)]
#[doc(hidden)]
pub struct SyntaxTreeBuilder {
    line_index: Arc<tombi_text::LineIndex>,
    entries: Vec<Entry>,
    token_ids: Vec<u32>,
    open: Vec<u32>,
    last_child: Vec<u32>,
}

impl SyntaxTreeBuilder {
    pub fn new(line_index: Arc<tombi_text::LineIndex>) -> Self {
        Self::with_capacity(line_index, 0)
    }

    pub fn with_capacity(line_index: Arc<tombi_text::LineIndex>, capacity: usize) -> Self {
        Self {
            line_index,
            entries: Vec::with_capacity(capacity),
            token_ids: Vec::with_capacity(capacity / 2),
            open: Vec::new(),
            last_child: Vec::new(),
        }
    }

    #[inline]
    fn parent_and_prev_sibling(&mut self, id: u32) -> (u32, u32) {
        let parent = self.open.last().copied().unwrap_or(ROOT_PARENT);
        let prev_sibling = self.last_child.last_mut().map_or(ROOT_PARENT, |last| {
            let previous = *last;
            *last = id;
            previous
        });
        (parent, prev_sibling)
    }

    pub fn start_node(&mut self, kind: SyntaxKind, offset: tombi_text::Offset) {
        let id = self.entries.len() as u32;
        let (parent, prev_sibling) = self.parent_and_prev_sibling(id);
        self.entries.push(Entry {
            span: tombi_text::Span::empty(offset),
            parent,
            subtree_end: id + 1,
            prev_sibling,
            kind,
            tag: EntryTag::Node,
        });
        self.open.push(id);
        self.last_child.push(ROOT_PARENT);
    }

    pub fn token(&mut self, kind: SyntaxKind, span: tombi_text::Span) {
        let id = self.entries.len() as u32;
        let (parent, prev_sibling) = self.parent_and_prev_sibling(id);
        let needs_decode = matches!(
            kind,
            SyntaxKind::BASIC_STRING | SyntaxKind::MULTI_LINE_BASIC_STRING
        ) && self.line_index.text()[span].contains('\\');
        self.entries.push(Entry {
            span,
            parent,
            subtree_end: id + 1,
            prev_sibling,
            kind,
            tag: EntryTag::token(needs_decode),
        });
        self.token_ids.push(id);
    }

    pub fn finish_node(&mut self, offset: tombi_text::Offset) {
        let id = self.open.pop().expect("finish_node without start_node");
        self.last_child.pop();
        let subtree_end = self.entries.len() as u32;
        let entry = &mut self.entries[id as usize];
        entry.span.end = offset;
        entry.subtree_end = subtree_end;
    }

    pub fn finish(self) -> SyntaxNode {
        assert!(self.open.is_empty(), "unclosed syntax nodes");
        assert!(!self.entries.is_empty(), "syntax tape has no root");
        let tree = Arc::new(Tree {
            line_index: self.line_index,
            token_ids: self.token_ids.into_boxed_slice(),
            entries: self.entries.into_boxed_slice(),
            header_index: Default::default(),
        });
        SyntaxNode::new(tree, 0)
    }
}

pub(crate) type SyntaxElement = NodeOrToken<SyntaxNode, SyntaxToken>;

/// Materialized syntax hierarchy intended only for diagnostics and parser
/// snapshot tests. Runtime consumers should use the typed TOML accessors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub enum DebugTree {
    Node {
        kind: SyntaxKind,
        children: Vec<DebugTree>,
    },
    Token {
        kind: SyntaxKind,
        text: String,
    },
}

/// A lightweight node cursor (`Arc` plus preorder id).
#[derive(Clone)]
pub struct SyntaxNode {
    tree: Arc<Tree>,
    id: u32,
}

impl SyntaxNode {
    #[inline]
    fn new(tree: Arc<Tree>, id: u32) -> Self {
        debug_assert!(!tree.entry(id).is_token());
        Self { tree, id }
    }

    #[inline]
    fn entry(&self) -> Entry {
        self.tree.entry(self.id)
    }

    #[inline]
    fn element(&self, id: u32) -> SyntaxElement {
        self.tree.element(id)
    }

    fn next_sibling_raw(&self) -> Option<SyntaxElement> {
        let entry = self.entry();
        let next = entry.subtree_end;
        (entry.parent != ROOT_PARENT
            && next < self.tree.entries.len() as u32
            && self.tree.entry(next).parent == entry.parent)
            .then(|| self.element(next))
    }

    fn prev_sibling_raw(&self) -> Option<SyntaxElement> {
        let prev = self.entry().prev_sibling;
        (prev != ROOT_PARENT).then(|| self.element(prev))
    }

    fn parent_node(&self) -> Option<Self> {
        let parent = self.entry().parent;
        (parent != ROOT_PARENT).then(|| Self::new(Arc::clone(&self.tree), parent))
    }

    fn child_ids(&self) -> ChildIds {
        let entry = self.entry();
        ChildIds {
            tree: Arc::clone(&self.tree),
            next: self.id + 1,
            end: entry.subtree_end,
        }
    }
}

impl PartialEq for SyntaxNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.tree, &other.tree)
    }
}
impl Eq for SyntaxNode {}
impl Hash for SyntaxNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.tree).hash(state);
        self.id.hash(state);
    }
}

/// A lightweight token cursor (`Arc` plus preorder id).
#[derive(Clone)]
pub struct SyntaxToken {
    tree: Arc<Tree>,
    id: u32,
}

impl SyntaxToken {
    #[inline]
    fn new(tree: Arc<Tree>, id: u32) -> Self {
        debug_assert!(tree.entry(id).is_token());
        Self { tree, id }
    }

    #[inline]
    fn entry(&self) -> Entry {
        self.tree.entry(self.id)
    }

    fn parent_node(&self) -> Option<SyntaxNode> {
        let parent = self.entry().parent;
        (parent != ROOT_PARENT).then(|| SyntaxNode::new(Arc::clone(&self.tree), parent))
    }

    fn next_sibling_raw(&self) -> Option<SyntaxElement> {
        let entry = self.entry();
        let next = self.id + 1;
        (entry.parent != ROOT_PARENT
            && next < self.tree.entries.len() as u32
            && self.tree.entry(next).parent == entry.parent)
            .then(|| self.tree.element(next))
    }

    fn prev_sibling_raw(&self) -> Option<SyntaxElement> {
        let prev = self.entry().prev_sibling;
        (prev != ROOT_PARENT).then(|| self.tree.element(prev))
    }
}

impl PartialEq for SyntaxToken {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.tree, &other.tree)
    }
}
impl Eq for SyntaxToken {}
impl Hash for SyntaxToken {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.tree).hash(state);
        self.id.hash(state);
    }
}

#[derive(Clone)]
struct ChildIds {
    tree: Arc<Tree>,
    next: u32,
    end: u32,
}

impl Iterator for ChildIds {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.end {
            return None;
        }
        let id = self.next;
        let entry = self.tree.entry(id);
        self.next = entry.subtree_end;
        Some(id)
    }
}

impl SyntaxNode {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.entry().kind
    }

    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.entry().span
    }

    /// The line index of the whole source, to convert spans into ranges.
    #[inline]
    pub fn line_index(&self) -> &Arc<tombi_text::LineIndex> {
        &self.tree.line_index
    }

    pub fn text(&self) -> &str {
        &self.tree.source()[self.span()]
    }

    #[inline]
    #[doc(hidden)]
    pub fn try_to_content(
        &self,
        version: tombi_toml_version::TomlVersion,
    ) -> Result<Cow<'_, str>, tombi_toml_text::ParseError> {
        self.tree.try_to_content(self.id, version)
    }

    #[inline]
    #[doc(hidden)]
    pub fn decoded_text_resolver(
        &self,
        version: tombi_toml_version::TomlVersion,
    ) -> DecodedTextResolver {
        self.tree.decoded_text_resolver(version)
    }

    #[inline]
    #[doc(hidden)]
    pub fn resolve_text(
        &self,
        resolver: &DecodedTextResolver,
    ) -> Result<(Arc<Box<str>>, tombi_text::Span), tombi_toml_text::ParseError> {
        resolver.resolve(&self.tree, self.tree.text_token_id(self.id))
    }

    #[inline]
    #[doc(hidden)]
    pub fn resolve_raw_text(
        &self,
        resolver: &DecodedTextResolver,
    ) -> (Arc<Box<str>>, tombi_text::Span) {
        resolver.resolve_raw(&self.tree, self.tree.text_token_id(self.id))
    }

    #[doc(hidden)]
    pub fn debug_tree(&self) -> DebugTree {
        let mut stack = Vec::<(SyntaxKind, Vec<DebugTree>)>::new();
        for event in self.preorder_with_tokens() {
            match event {
                WalkEvent::Enter(NodeOrToken::Node(node)) => {
                    stack.push((node.kind(), Vec::new()));
                }
                WalkEvent::Enter(NodeOrToken::Token(token)) => stack
                    .last_mut()
                    .expect("a token must have a parent node")
                    .1
                    .push(DebugTree::Token {
                        kind: token.kind(),
                        text: token.text().to_owned(),
                    }),
                WalkEvent::Leave(NodeOrToken::Node(_)) => {
                    let (kind, children) = stack.pop().expect("entered node must be left");
                    let tree = DebugTree::Node { kind, children };
                    if let Some(parent) = stack.last_mut() {
                        parent.1.push(tree);
                    } else {
                        return tree;
                    }
                }
                WalkEvent::Leave(NodeOrToken::Token(_)) => {
                    unreachable!("tokens have no leave event")
                }
            }
        }
        unreachable!("a syntax node always emits enter and leave events")
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        self.parent_node()
    }

    #[inline]
    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    /// Lazily built, per-tree index shared by every node of this tree.
    pub(crate) fn header_index(&self) -> &crate::ast::HeaderIndex {
        self.tree
            .header_index
            .get_or_init(|| crate::ast::HeaderIndex::build(self))
    }

    pub(crate) fn node_at(&self, id: u32) -> Self {
        Self::new(Arc::clone(&self.tree), id)
    }

    pub(crate) fn ancestors(&self) -> impl Iterator<Item = Self> {
        std::iter::successors(self.parent(), Self::parent)
    }

    pub(crate) fn child_nodes(&self) -> impl Iterator<Item = Self> + use<> {
        let tree = Arc::clone(&self.tree);
        let filter_tree = Arc::clone(&tree);
        self.child_ids()
            .filter(move |id| !filter_tree.entry(*id).is_token())
            .map(move |id| Self::new(Arc::clone(&tree), id))
    }

    pub(crate) fn child_elements(&self) -> impl Iterator<Item = SyntaxElement> + use<> {
        let tree = Arc::clone(&self.tree);
        self.child_ids().map(move |id| tree.element(id))
    }

    pub(crate) fn last_child(&self) -> Option<Self> {
        self.child_nodes().last()
    }

    pub(crate) fn first_child_or_token(&self) -> Option<SyntaxElement> {
        self.child_elements().next()
    }

    pub(crate) fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.next_sibling_raw()
    }

    pub(crate) fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.prev_sibling_raw()
    }

    pub(crate) fn first_token(&self) -> Option<SyntaxToken> {
        let entry = self.entry();
        (self.id + 1..entry.subtree_end).find_map(|id| {
            self.tree
                .entry(id)
                .is_token()
                .then(|| SyntaxToken::new(Arc::clone(&self.tree), id))
        })
    }

    pub(crate) fn last_token(&self) -> Option<SyntaxToken> {
        let entry = self.entry();
        (self.id + 1..entry.subtree_end).rev().find_map(|id| {
            self.tree
                .entry(id)
                .is_token()
                .then(|| SyntaxToken::new(Arc::clone(&self.tree), id))
        })
    }

    pub(crate) fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElement> {
        let first = Some(NodeOrToken::Node(self.clone()));
        std::iter::successors(first, move |element| match direction {
            Direction::Next => element.next_sibling_or_token(),
            Direction::Prev => element.prev_sibling_or_token(),
        })
    }

    pub(crate) fn descendants(&self) -> impl Iterator<Item = Self> {
        let tree = Arc::clone(&self.tree);
        let filter_tree = Arc::clone(&tree);
        let entry = self.entry();
        (self.id..entry.subtree_end)
            .filter(move |id| !filter_tree.entry(*id).is_token())
            .map(move |id| Self::new(Arc::clone(&tree), id))
    }

    pub(crate) fn preorder_with_tokens(&self) -> PreorderWithTokens {
        PreorderWithTokens::new(self.clone())
    }

    pub(crate) fn token_at_offset(&self, offset: tombi_text::Offset) -> TokenAtOffset<SyntaxToken> {
        let subtree = self.id + 1..self.entry().subtree_end;
        let token_ids = &self.tree.token_ids;
        let first = token_ids.partition_point(|id| *id < subtree.start);
        let last = token_ids.partition_point(|id| *id < subtree.end);
        let tokens = &token_ids[first..last];
        let index = tokens.partition_point(|id| self.tree.entry(*id).span.end <= offset);
        let token = |id: u32| SyntaxToken::new(Arc::clone(&self.tree), id);
        let left = index
            .checked_sub(1)
            .and_then(|index| tokens.get(index))
            .copied();
        let right = tokens.get(index).copied();
        match (left, right) {
            (Some(left), Some(right))
                if self.tree.entry(left).span.end == offset
                    && self.tree.entry(right).span.start == offset =>
            {
                TokenAtOffset::Between(token(left), token(right))
            }
            (Some(id), None) if self.tree.entry(id).span.end == offset => {
                TokenAtOffset::Single(token(id))
            }
            (_, Some(id)) if self.tree.entry(id).span.contains(offset) => {
                TokenAtOffset::Single(token(id))
            }
            (Some(id), _) if self.tree.entry(id).span.contains(offset) => {
                TokenAtOffset::Single(token(id))
            }
            _ => TokenAtOffset::None,
        }
    }
}

impl fmt::Debug for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @{}", self.kind(), self.span())
    }
}
impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.tree.source()[self.span()])
    }
}

impl SyntaxToken {
    pub fn kind(&self) -> SyntaxKind {
        self.entry().kind
    }
    pub fn span(&self) -> tombi_text::Span {
        self.entry().span
    }
    pub fn text(&self) -> &str {
        &self.tree.source()[self.span()]
    }
    pub(crate) fn parent(&self) -> Option<SyntaxNode> {
        self.parent_node()
    }
    pub(crate) fn parent_ancestors(&self) -> impl Iterator<Item = SyntaxNode> {
        std::iter::successors(self.parent(), SyntaxNode::parent)
    }
    pub(crate) fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.next_sibling_raw()
    }
    pub(crate) fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.prev_sibling_raw()
    }
}

impl fmt::Debug for SyntaxToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @{} {:?}", self.kind(), self.span(), self.text())
    }
}
impl fmt::Display for SyntaxToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.text())
    }
}

impl From<SyntaxNode> for SyntaxElement {
    fn from(node: SyntaxNode) -> Self {
        NodeOrToken::Node(node)
    }
}

impl From<SyntaxToken> for SyntaxElement {
    fn from(token: SyntaxToken) -> Self {
        NodeOrToken::Token(token)
    }
}

impl SyntaxElement {
    pub(crate) fn span(&self) -> tombi_text::Span {
        match self {
            NodeOrToken::Node(node) => node.span(),
            NodeOrToken::Token(token) => token.span(),
        }
    }
    pub(crate) fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(node) => node.kind(),
            NodeOrToken::Token(token) => token.kind(),
        }
    }
    pub(crate) fn next_sibling_or_token(&self) -> Option<Self> {
        match self {
            NodeOrToken::Node(node) => node.next_sibling_or_token(),
            NodeOrToken::Token(token) => token.next_sibling_or_token(),
        }
    }
    pub(crate) fn prev_sibling_or_token(&self) -> Option<Self> {
        match self {
            NodeOrToken::Node(node) => node.prev_sibling_or_token(),
            NodeOrToken::Token(token) => token.prev_sibling_or_token(),
        }
    }
}

pub(crate) struct PreorderWithTokens {
    tree: Arc<Tree>,
    next: u32,
    end: u32,
    leaving: Vec<u32>,
}
impl PreorderWithTokens {
    fn new(root: SyntaxNode) -> Self {
        let entry = root.entry();
        Self {
            tree: root.tree,
            next: root.id,
            end: entry.subtree_end,
            leaving: Vec::new(),
        }
    }
}
impl Iterator for PreorderWithTokens {
    type Item = WalkEvent<SyntaxElement>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.leaving.last().copied()
            && self.next >= self.tree.entry(id).subtree_end
        {
            self.leaving.pop();
            return Some(WalkEvent::Leave(self.tree.element(id)));
        }
        if self.next >= self.end {
            return None;
        }
        let id = self.next;
        let entry = self.tree.entry(id);
        self.next += 1;
        if !entry.is_token() {
            self.leaving.push(id);
        }
        Some(WalkEvent::Enter(self.tree.element(id)))
    }
}

#[cfg(test)]
mod tests {
    use super::Entry;

    #[test]
    fn entry_is_compact() {
        assert_eq!(std::mem::size_of::<Entry>(), 24);
    }
}
