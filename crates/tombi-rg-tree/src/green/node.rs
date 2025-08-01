use std::{
    borrow::{Borrow, Cow},
    fmt,
    iter::{self, FusedIterator},
    mem::{self, ManuallyDrop},
    ops, ptr, slice,
};

use countme::Count;

use crate::{
    arc::{Arc, HeaderSlice, ThinArc},
    green::{GreenElement, GreenElementRef, SyntaxKind},
    GreenToken, NodeOrToken,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    kind: SyntaxKind,
    text_len: tombi_text::RelativeOffset,
    text_relative_position: tombi_text::RelativePosition,
    _c: Count<GreenNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum GreenChild {
    Node {
        relative_offset: tombi_text::RelativeOffset,
        relative_position: tombi_text::RelativePosition,
        node: GreenNode,
    },
    Token {
        relative_offset: tombi_text::RelativeOffset,
        relative_position: tombi_text::RelativePosition,
        token: GreenToken,
    },
}

#[cfg(target_pointer_width = "64")]
crate::utility_types::static_assert!(mem::size_of::<GreenChild>() == mem::size_of::<usize>() * 3);

type Repr = HeaderSlice<GreenNodeHead, [GreenChild]>;
type ReprThin = HeaderSlice<GreenNodeHead, [GreenChild; 0]>;
#[repr(transparent)]
pub struct GreenNodeData {
    data: ReprThin,
}

impl PartialEq for GreenNodeData {
    fn eq(&self, other: &Self) -> bool {
        self.header() == other.header() && self.slice() == other.slice()
    }
}

/// Internal node in the immutable tree.
/// It has other nodes and tokens as children.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GreenNode {
    ptr: ThinArc<GreenNodeHead, GreenChild>,
}

impl ToOwned for GreenNodeData {
    type Owned = GreenNode;

    #[inline]
    fn to_owned(&self) -> GreenNode {
        let green = unsafe { GreenNode::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenNode::clone(&green)
    }
}

impl Borrow<GreenNodeData> for GreenNode {
    #[inline]
    fn borrow(&self) -> &GreenNodeData {
        self
    }
}

impl From<Cow<'_, GreenNodeData>> for GreenNode {
    #[inline]
    fn from(cow: Cow<'_, GreenNodeData>) -> Self {
        cow.into_owned()
    }
}

impl fmt::Debug for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenNode")
            .field("kind", &self.kind())
            .field("text_len", &self.text_len())
            .field("n_children", &self.children().len())
            .finish()
    }
}

impl fmt::Debug for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenNodeData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenNodeData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Display for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in self.children() {
            write!(f, "{child}")?;
        }
        Ok(())
    }
}

impl GreenNodeData {
    #[inline]
    fn header(&self) -> &GreenNodeHead {
        &self.data.header
    }

    #[inline]
    fn slice(&self) -> &[GreenChild] {
        self.data.slice()
    }

    /// Kind of this node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.header().kind
    }

    /// Returns the length of the text covered by this node.
    #[inline]
    pub fn text_len(&self) -> tombi_text::RawOffset {
        self.header().text_len
    }

    #[inline]
    pub fn text_relative_position(&self) -> tombi_text::RelativePosition {
        self.header().text_relative_position
    }

    /// Children of this node.
    #[inline]
    pub fn children(&self) -> Children<'_> {
        Children {
            raw: self.slice().iter(),
        }
    }

    pub(crate) fn child_at_span(
        &self,
        rel_span: tombi_text::Span,
    ) -> Option<(
        usize,
        tombi_text::RelativeOffset,
        tombi_text::RelativePosition,
        GreenElementRef<'_>,
    )> {
        let idx = self
            .slice()
            .binary_search_by(|it| {
                let child_range = it.rel_span();
                tombi_text::Span::ordering(child_range, rel_span)
            })
            // XXX: this handles empty ranges
            .unwrap_or_else(|it| it.saturating_sub(1));
        let child = &self
            .slice()
            .get(idx)
            .filter(|it| it.rel_span().contains_span(rel_span))?;
        Some((
            idx,
            child.relative_offset(),
            child.relative_position(),
            child.as_ref(),
        ))
    }

    #[must_use]
    pub fn replace_child(&self, index: usize, new_child: GreenElement) -> GreenNode {
        let mut replacement = Some(new_child);
        let children = self.children().enumerate().map(|(i, child)| {
            if i == index {
                replacement.take().unwrap()
            } else {
                child.to_owned()
            }
        });
        GreenNode::new(self.kind(), children)
    }
    #[must_use]
    pub fn insert_child(&self, index: usize, new_child: GreenElement) -> GreenNode {
        // https://github.com/rust-lang/rust/issues/34433
        self.splice_children(index..index, iter::once(new_child))
    }
    #[must_use]
    pub fn remove_child(&self, index: usize) -> GreenNode {
        self.splice_children(index..=index, iter::empty())
    }
    #[must_use]
    pub fn splice_children<R, I>(&self, range: R, replace_with: I) -> GreenNode
    where
        R: ops::RangeBounds<usize>,
        I: IntoIterator<Item = GreenElement>,
    {
        let mut children: Vec<_> = self.children().map(|it| it.to_owned()).collect();
        children.splice(range, replace_with);
        GreenNode::new(self.kind(), children)
    }
}

impl ops::Deref for GreenNode {
    type Target = GreenNodeData;

    #[inline]
    fn deref(&self) -> &GreenNodeData {
        let repr: &Repr = &self.ptr;
        unsafe {
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenNodeData>(repr)
        }
    }
}

impl GreenNode {
    /// Creates new Node.
    #[inline]
    pub fn new<I>(kind: SyntaxKind, children: I) -> GreenNode
    where
        I: IntoIterator<Item = GreenElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut text_len: tombi_text::RawOffset = 0;
        let mut text_relative_position = Default::default();
        let children = children.into_iter().map(|el| {
            let relative_offset = text_len;
            let relative_position = text_relative_position;
            text_len += el.text_len();
            text_relative_position += el.text_relative_position();

            match el {
                NodeOrToken::Node(node) => GreenChild::Node {
                    relative_offset,
                    relative_position,
                    node,
                },
                NodeOrToken::Token(token) => GreenChild::Token {
                    relative_offset,
                    relative_position,
                    token,
                },
            }
        });

        let data = ThinArc::from_header_and_iter(
            GreenNodeHead {
                kind,
                text_len: 0,
                text_relative_position: Default::default(),
                _c: Count::new(),
            },
            children,
        );

        // XXX: fixup `text_len` after construction, because we can't iterate
        // `children` twice.
        let data = {
            let mut data = Arc::from_thin(data);
            Arc::get_mut(&mut data).unwrap().header.text_len = text_len;
            Arc::get_mut(&mut data)
                .unwrap()
                .header
                .text_relative_position = text_relative_position;
            Arc::into_thin(data)
        };

        GreenNode { ptr: data }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenNode) -> ptr::NonNull<GreenNodeData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenNodeData = &green;
        ptr::NonNull::from(green)
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenNodeData>) -> GreenNode {
        let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
        let arc = mem::transmute::<Arc<ReprThin>, ThinArc<GreenNodeHead, GreenChild>>(arc);
        GreenNode { ptr: arc }
    }
}

impl GreenChild {
    #[inline]
    pub(crate) fn as_ref(&self) -> GreenElementRef {
        match self {
            GreenChild::Node { node, .. } => NodeOrToken::Node(node),
            GreenChild::Token { token, .. } => NodeOrToken::Token(token),
        }
    }
    #[inline]
    pub(crate) fn relative_offset(&self) -> tombi_text::RelativeOffset {
        match self {
            GreenChild::Node {
                relative_offset, ..
            }
            | GreenChild::Token {
                relative_offset, ..
            } => *relative_offset,
        }
    }

    #[inline]
    pub(crate) fn relative_position(&self) -> tombi_text::RelativePosition {
        match self {
            GreenChild::Node {
                relative_position, ..
            }
            | GreenChild::Token {
                relative_position, ..
            } => *relative_position,
        }
    }

    #[inline]
    fn rel_span(&self) -> tombi_text::Span {
        let len = self.as_ref().text_len();
        tombi_text::Span::at(tombi_text::Offset::new(self.relative_offset()), len)
    }
}

#[derive(Debug, Clone)]
pub struct Children<'a> {
    pub(crate) raw: slice::Iter<'a, GreenChild>,
}

// NB: forward everything stable that iter::Slice specializes as of Rust 1.39.0
impl ExactSizeIterator for Children<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.raw.len()
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = GreenElementRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<GreenElementRef<'a>> {
        self.raw.next().map(GreenChild::as_ref)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.raw.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth(n).map(GreenChild::as_ref)
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let mut accum = init;
        for x in self {
            accum = f(accum, x);
        }
        accum
    }
}

impl DoubleEndedIterator for Children<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw.next_back().map(GreenChild::as_ref)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth_back(n).map(GreenChild::as_ref)
    }

    #[inline]
    fn rfold<Acc, Fold>(mut self, init: Acc, mut f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let mut accum = init;
        while let Some(x) = self.next_back() {
            accum = f(accum, x);
        }
        accum
    }
}

impl FusedIterator for Children<'_> {}
