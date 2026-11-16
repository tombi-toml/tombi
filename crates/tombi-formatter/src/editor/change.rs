use std::ops::Range;

use itertools::Itertools;

use tombi_ast_syntax::{AstNode, SyntaxKind, SyntaxNode, SyntaxToken};

#[derive(Debug)]
pub(super) struct Change(ChangeKind);

#[derive(Debug)]
enum ChangeKind {
    AppendTop {
        text: String,
    },
    Append {
        base: Range<usize>,
        text: String,
        remove_before: Option<Range<usize>>,
        /// The text ends with a trailing comment, so a line break has to follow.
        trailing_comment: bool,
    },
    Remove {
        target: Range<usize>,
    },
    ReplaceRange {
        old: Range<usize>,
        new: Vec<SourcePart>,
    },
}

#[derive(Debug, Clone)]
pub(super) struct SourcePart {
    source_span: Range<usize>,
    kind: SyntaxKind,
}

impl Change {
    pub(super) fn append_top(text: String) -> Self {
        Self(ChangeKind::AppendTop { text })
    }

    pub(super) fn append<'t>(base: &impl AstNode<'t>, text: String) -> Self {
        Self(ChangeKind::Append {
            base: node_span(base.syntax()),
            text,
            remove_before: None,
            trailing_comment: false,
        })
    }

    /// Appends a comma followed by `comment` after `base`.
    pub(super) fn append_comma_with_comment<'t>(
        base: &impl AstNode<'t>,
        comment: &SyntaxToken,
    ) -> Self {
        Self(ChangeKind::Append {
            base: node_span(base.syntax()),
            text: format!(",{}", comment.text()),
            remove_before: None,
            trailing_comment: true,
        })
    }

    /// Like [`Self::append_comma_with_comment`], replacing the existing `comma`.
    pub(super) fn append_comma_with_comment_replacing<'t, 'u>(
        base: &impl AstNode<'t>,
        replaced: &impl AstNode<'u>,
        comment: &SyntaxToken,
    ) -> Self {
        Self(ChangeKind::Append {
            base: node_span(base.syntax()),
            text: format!(",{}", comment.text()),
            remove_before: Some(node_span(replaced.syntax())),
            trailing_comment: true,
        })
    }

    pub(super) fn remove_token(target: &SyntaxToken) -> Self {
        Self(ChangeKind::Remove {
            target: token_span(target),
        })
    }

    pub(super) fn replace_span(
        first: &SyntaxNode,
        last: &SyntaxNode,
        new: Vec<SourcePart>,
    ) -> Self {
        let old = node_span(first).start..node_span(last).end;
        Self(ChangeKind::ReplaceRange { old, new })
    }
}

impl SourcePart {
    pub(super) fn node<'t>(node: &impl AstNode<'t>) -> Self {
        Self {
            source_span: node_span(node.syntax()),
            kind: node.syntax().kind(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum RewriteError {
    InvalidSourceSpan(Range<usize>),
    /// Two replaced ranges overlap without one containing the other.
    OverlappingReplace(Range<usize>),
}

#[derive(Debug, PartialEq, Eq)]
struct Appended {
    base: Range<usize>,
    text: String,
    trailing_comment: bool,
}

/// Every change refers to the original source, so the result must not depend
/// on the order in which the rules emitted them. `apply` first collects all
/// changes and then renders the source once.
struct Plan<'a> {
    source: &'a str,
    top: String,
    /// Sorted, merged and non-overlapping.
    removed: Vec<Range<usize>>,
    /// Sorted by `base.end`; changes at the same offset keep their order.
    appends: Vec<Appended>,
    /// Sorted by `old.start`, then outermost first. Nested or disjoint only.
    replaces: Vec<(Range<usize>, Vec<SourcePart>)>,
    /// Parts that lie outside the range they replace (e.g. the trailing comma
    /// of a reordered array), as `(index into replaces, span)`. They are moved,
    /// so their original position is skipped.
    moved: Vec<(usize, Range<usize>)>,
}

pub(super) fn apply(root: &SyntaxNode, changes: Vec<Change>) -> Result<String, RewriteError> {
    let plan = Plan::new(root.text(), changes)?;
    let mut output = Output::default();
    output.push_str(&plan.top);
    plan.render(
        0..plan.source.len(),
        &mut vec![false; plan.replaces.len()],
        &mut output,
    );
    Ok(output.text)
}

/// Rendered text.
///
/// A trailing comment runs to the end of the line, so text that follows it has
/// to start on a new line. That is decided when the following text arrives,
/// wherever it comes from (the same part, a sibling part or the text after a
/// replaced range).
#[derive(Default)]
struct Output {
    text: String,
    after_trailing_comment: bool,
}

impl Output {
    fn push_str(&mut self, text: &str) {
        if self.after_trailing_comment {
            match text.trim_start_matches([' ', '\t']).chars().next() {
                // Still on the line of the comment.
                None => {}
                Some('\n' | '\r') => self.after_trailing_comment = false,
                Some(_) => {
                    self.text.push('\n');
                    self.after_trailing_comment = false;
                }
            }
        }
        self.text.push_str(text);
    }

    fn push_trailing_comment(&mut self, text: &str) {
        self.push_str(text);
        self.after_trailing_comment = true;
    }

    fn append(&mut self, other: Output) {
        self.push_str(&other.text);
        self.after_trailing_comment |= other.after_trailing_comment;
    }
}

impl<'a> Plan<'a> {
    fn new(source: &'a str, changes: Vec<Change>) -> Result<Self, RewriteError> {
        let mut top = String::new();
        let mut removed = Vec::new();
        let mut appends = Vec::new();
        let mut replaces: Vec<(Range<usize>, Vec<SourcePart>)> = Vec::new();

        for Change(change) in changes {
            match change {
                ChangeKind::AppendTop { text } => top.push_str(&text),
                ChangeKind::Append {
                    base,
                    text,
                    remove_before,
                    trailing_comment,
                } => {
                    validate_span(source, &base)?;
                    if let Some(replaced) = remove_before {
                        validate_span(source, &replaced)?;
                        removed.push(replaced);
                    }
                    // Rules may be evaluated more than once for the same node.
                    let appended = Appended {
                        base,
                        text,
                        trailing_comment,
                    };
                    if !appends.contains(&appended) {
                        appends.push(appended);
                    }
                }
                ChangeKind::Remove { target } => {
                    validate_span(source, &target)?;
                    removed.push(target);
                }
                ChangeKind::ReplaceRange { old, new } => {
                    validate_span(source, &old)?;
                    for part in &new {
                        validate_span(source, &part.source_span)?;
                    }
                    // The same range may be requested twice; the last one wins.
                    replaces.retain(|(existing, _)| existing != &old);
                    replaces.push((old, new));
                }
            }
        }

        removed.sort_by_key(|span| span.start);
        let mut merged: Vec<Range<usize>> = Vec::with_capacity(removed.len());
        for span in removed {
            match merged.last_mut() {
                Some(last) if span.start <= last.end => last.end = last.end.max(span.end),
                _ => merged.push(span),
            }
        }

        // Text that is already removed cannot be appended to.
        appends.retain(|appended| !is_removed(&merged, &appended.base));
        appends.sort_by_key(|appended| appended.base.end);

        replaces.sort_by(|(left, _), (right, _)| {
            left.start.cmp(&right.start).then(right.end.cmp(&left.end))
        });
        let mut enclosing: Vec<&Range<usize>> = Vec::new();
        for (old, _) in &replaces {
            while enclosing.last().is_some_and(|outer| outer.end <= old.start) {
                enclosing.pop();
            }
            if enclosing.last().is_some_and(|outer| old.end > outer.end) {
                return Err(RewriteError::OverlappingReplace(old.clone()));
            }
            enclosing.push(old);
        }

        let moved = replaces
            .iter()
            .enumerate()
            .flat_map(|(index, (old, parts))| {
                parts
                    .iter()
                    .map(|part| part.source_span.clone())
                    .filter(|span| span.start < old.start || old.end < span.end)
                    .map(move |span| (index, span))
            })
            .collect_vec();

        Ok(Self {
            source,
            top,
            removed: merged,
            appends,
            replaces,
            moved,
        })
    }

    /// Renders `span` of the source, applying every change that lies inside it.
    ///
    /// `active` marks the replacements currently being expanded, so a
    /// replacement is never applied to its own parts.
    fn render(&self, span: Range<usize>, active: &mut [bool], output: &mut Output) {
        let mut position = span.start;
        for (index, (old, parts)) in self.replaces.iter().enumerate() {
            if active[index] || old.start < position || old.end > span.end {
                continue;
            }
            self.render_plain(position..old.start, active, output);
            active[index] = true;
            let rendered = parts
                .iter()
                .map(|part| {
                    let mut rendered = Output::default();
                    self.render(part.source_span.clone(), active, &mut rendered);
                    rendered
                })
                .collect_vec();
            active[index] = false;
            output.append(separate_source_parts(parts, rendered));
            position = old.end;
        }
        self.render_plain(position..span.end, active, output);
    }

    /// Renders source text without replacements: removed text is skipped and
    /// appended text is inserted right after the node it is attached to.
    fn render_plain(&self, span: Range<usize>, active: &[bool], output: &mut Output) {
        let skipped = self
            .removed
            .iter()
            .chain(
                self.moved
                    .iter()
                    .filter(|(index, _)| !active[*index])
                    .map(|(_, moved)| moved),
            )
            .filter(|skipped| overlaps(skipped, &span))
            .cloned()
            .sorted_by_key(|skipped| skipped.start)
            .collect_vec();

        let first = self
            .appends
            .partition_point(|appended| appended.base.end <= span.start);
        let mut position = span.start;
        for appended in self.appends[first..]
            .iter()
            .take_while(|appended| appended.base.end <= span.end)
            // Appends attached to a moved part are emitted along with the part.
            .filter(|appended| {
                !self.moved.iter().any(|(index, moved)| {
                    !active[*index] && is_removed(std::slice::from_ref(moved), &appended.base)
                })
            })
        {
            push_unskipped(self.source, &skipped, position..appended.base.end, output);
            if appended.trailing_comment {
                output.push_trailing_comment(&appended.text);
            } else {
                output.push_str(&appended.text);
            }
            position = position.max(appended.base.end);
        }
        push_unskipped(self.source, &skipped, position..span.end, output);
    }
}

fn push_unskipped(source: &str, skipped: &[Range<usize>], span: Range<usize>, output: &mut Output) {
    let mut position = span.start;
    for skipped in skipped.iter().filter(|skipped| overlaps(skipped, &span)) {
        if position < skipped.start {
            output.push_str(&source[position..skipped.start]);
        }
        position = position.max(skipped.end);
    }
    if position < span.end {
        output.push_str(&source[position..span.end]);
    }
}

fn is_removed(removed: &[Range<usize>], target: &Range<usize>) -> bool {
    removed
        .iter()
        .any(|span| span.start <= target.start && target.end <= span.end)
}

fn separate_source_parts(parts: &[SourcePart], rendered: Vec<Output>) -> Output {
    let mut output = Output::default();
    let mut previous = None;
    for (index, (part, rendered)) in parts.iter().zip(rendered).enumerate() {
        if index > 0 {
            let next = rendered.text.chars().next();
            let prefix = if matches!(part.kind, SyntaxKind::TABLE | SyntaxKind::ARRAY_OF_TABLE)
                && next.is_some_and(|character| !character.is_whitespace())
            {
                if previous == Some('\n') { "\n" } else { "\n\n" }
            } else if previous
                .is_some_and(|character| !character.is_whitespace() && character != ',')
                && next.is_some_and(|character| !character.is_whitespace())
            {
                "\n"
            } else {
                ""
            };
            output.push_str(prefix);
        }
        previous = rendered.text.chars().next_back();
        output.append(rendered);
    }
    output
}

fn overlaps(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

fn validate_span(source: &str, span: &Range<usize>) -> Result<(), RewriteError> {
    if span.start <= span.end && source.get(span.clone()).is_some() {
        Ok(())
    } else {
        Err(RewriteError::InvalidSourceSpan(span.clone()))
    }
}

fn node_span(node: &SyntaxNode) -> Range<usize> {
    usize::from(node.span().start)..usize::from(node.span().end)
}

fn token_span(token: &SyntaxToken) -> Range<usize> {
    usize::from(token.span().start)..usize::from(token.span().end)
}
