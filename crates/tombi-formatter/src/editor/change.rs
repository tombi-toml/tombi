use std::ops::Range;

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

    pub(super) fn append(base: &impl AstNode, text: String) -> Self {
        Self(ChangeKind::Append {
            base: node_span(base.syntax()),
            text,
            remove_before: None,
        })
    }

    pub(super) fn append_replacing(
        base: &impl AstNode,
        replaced: &impl AstNode,
        text: String,
    ) -> Self {
        Self(ChangeKind::Append {
            base: node_span(base.syntax()),
            text,
            remove_before: Some(node_span(replaced.syntax())),
        })
    }

    pub(super) fn remove_token(target: &SyntaxToken) -> Self {
        Self(ChangeKind::Remove {
            target: token_span(target),
        })
    }

    pub(super) fn replace_range(
        first: &SyntaxNode,
        last: &SyntaxNode,
        new: Vec<SourcePart>,
    ) -> Self {
        let old = node_span(first).start..node_span(last).end;
        Self(ChangeKind::ReplaceRange { old, new })
    }
}

impl SourcePart {
    pub(super) fn node(node: &impl AstNode) -> Self {
        Self {
            source_span: node_span(node.syntax()),
            kind: node.syntax().kind(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum RewriteError {
    TargetNotFound { operation: &'static str },
    InvalidSourceSpan(Range<usize>),
}

#[derive(Debug, Clone)]
enum Piece {
    Source(Range<usize>),
    Generated {
        text: String,
        anchor: Option<Range<usize>>,
    },
    Marker,
}

pub(super) fn apply(root: &SyntaxNode, changes: Vec<Change>) -> Result<String, RewriteError> {
    let source = root.text();
    let mut pieces = vec![Piece::Source(0..source.len())];
    for Change(change) in changes {
        match change {
            ChangeKind::AppendTop { text } => {
                let insert_at = pieces
                    .iter()
                    .take_while(|piece| matches!(piece, Piece::Generated { anchor: None, .. }))
                    .count();
                pieces.insert(insert_at, Piece::Generated { text, anchor: None });
            }
            ChangeKind::Append {
                base,
                text,
                remove_before,
            } => {
                if let Some(replaced) = remove_before {
                    // Rule changes can overlap. Removal is intentionally
                    // idempotent when another earlier change already removed it.
                    remove_source_span(&mut pieces, replaced);
                }
                let mut insert_at =
                    after_source_span(&mut pieces, &base).ok_or(RewriteError::TargetNotFound {
                        operation: "append",
                    })?;
                while matches!(
                    pieces.get(insert_at),
                    Some(Piece::Generated { anchor: Some(anchor), .. }) if anchor == &base
                ) {
                    insert_at += 1;
                }
                pieces.insert(
                    insert_at,
                    Piece::Generated {
                        text,
                        anchor: Some(base),
                    },
                );
            }
            ChangeKind::Remove { target } => {
                // Rule changes can overlap. Removal is intentionally idempotent.
                remove_source_span(&mut pieces, target);
            }
            ChangeKind::ReplaceRange { old, new } => {
                let materialized = new
                    .iter()
                    .map(|part| {
                        if let Some(extracted) = extract_source_span(&pieces, &part.source_span) {
                            return Ok(extracted);
                        }

                        validate_span(source, &part.source_span)?;
                        Ok(vec![Piece::Generated {
                            text: source[part.source_span.clone()].to_owned(),
                            anchor: Some(part.source_span.clone()),
                        }])
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                insert_marker(&mut pieces, old.start).ok_or(RewriteError::TargetNotFound {
                    operation: "replace range",
                })?;
                remove_source_span(&mut pieces, old);
                for part in &new {
                    remove_source_span(&mut pieces, part.source_span.clone());
                }
                let replacement = separated_source_parts(source, new, materialized);
                let marker = pieces
                    .iter()
                    .position(|piece| matches!(piece, Piece::Marker))
                    .ok_or(RewriteError::TargetNotFound {
                        operation: "replacement marker",
                    })?;
                pieces.splice(marker..=marker, replacement);
            }
        }
    }

    let mut output = String::with_capacity(source.len());
    for piece in pieces {
        match piece {
            Piece::Source(span) => {
                validate_span(source, &span)?;
                output.push_str(&source[span]);
            }
            Piece::Generated { text, .. } => output.push_str(&text),
            Piece::Marker => unreachable!("all replacement markers are consumed"),
        }
    }
    Ok(output)
}

fn separated_source_parts(
    source: &str,
    parts: Vec<SourcePart>,
    materialized: Vec<Vec<Piece>>,
) -> Vec<Piece> {
    let mut pieces = Vec::with_capacity(parts.len() * 2);
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            let prefix = if matches!(part.kind, SyntaxKind::TABLE | SyntaxKind::ARRAY_OF_TABLE)
                && first_piece_char(source, &materialized[index])
                    .is_some_and(|character| !character.is_whitespace())
            {
                if last_piece_char(source, &materialized[index - 1]) == Some('\n') {
                    "\n"
                } else {
                    "\n\n"
                }
            } else if last_piece_char(source, &materialized[index - 1])
                .is_some_and(|character| !character.is_whitespace() && character != ',')
                && first_piece_char(source, &materialized[index])
                    .is_some_and(|character| !character.is_whitespace())
            {
                "\n"
            } else {
                ""
            };
            if !prefix.is_empty() {
                pieces.push(Piece::Generated {
                    text: prefix.to_owned(),
                    anchor: Some(part.source_span.clone()),
                });
            }
        }
        pieces.extend(materialized[index].iter().cloned());
    }
    pieces
}

fn extract_source_span(pieces: &[Piece], target: &Range<usize>) -> Option<Vec<Piece>> {
    let first = pieces
        .iter()
        .position(|piece| matches!(piece, Piece::Source(span) if overlaps(span, target)))?;
    let last = pieces
        .iter()
        .rposition(|piece| matches!(piece, Piece::Source(span) if overlaps(span, target)))?;
    let mut extracted = Vec::with_capacity(last - first + 1);
    for piece in &pieces[first..=last] {
        match piece {
            Piece::Source(span) => {
                let start = span.start.max(target.start);
                let end = span.end.min(target.end);
                if start < end {
                    extracted.push(Piece::Source(start..end));
                }
            }
            Piece::Generated { text, anchor } => extracted.push(Piece::Generated {
                text: text.clone(),
                anchor: anchor.clone(),
            }),
            Piece::Marker => {}
        }
    }
    Some(extracted)
}

fn first_piece_char(source: &str, pieces: &[Piece]) -> Option<char> {
    pieces.iter().find_map(|piece| match piece {
        Piece::Source(span) => source[span.clone()].chars().next(),
        Piece::Generated { text, .. } => text.chars().next(),
        Piece::Marker => None,
    })
}

fn last_piece_char(source: &str, pieces: &[Piece]) -> Option<char> {
    pieces.iter().rev().find_map(|piece| match piece {
        Piece::Source(span) => source[span.clone()].chars().next_back(),
        Piece::Generated { text, .. } => text.chars().next_back(),
        Piece::Marker => None,
    })
}

fn insert_marker(pieces: &mut Vec<Piece>, offset: usize) -> Option<()> {
    let index = pieces.iter().position(|piece| match piece {
        Piece::Source(span) => span.start <= offset && offset <= span.end,
        Piece::Generated { .. } | Piece::Marker => false,
    })?;
    let Piece::Source(span) = pieces.remove(index) else {
        unreachable!()
    };
    let mut replacement = Vec::with_capacity(3);
    if span.start < offset {
        replacement.push(Piece::Source(span.start..offset));
    }
    replacement.push(Piece::Marker);
    if offset < span.end {
        replacement.push(Piece::Source(offset..span.end));
    }
    pieces.splice(index..index, replacement);
    Some(())
}

fn after_source_span(pieces: &mut Vec<Piece>, target: &Range<usize>) -> Option<usize> {
    let mut last_overlap = None;
    for index in 0..pieces.len() {
        let Piece::Source(span) = &pieces[index] else {
            continue;
        };
        if overlaps(span, target) {
            last_overlap = Some(index + 1);
            if span.start < target.end && target.end < span.end {
                let span = span.clone();
                pieces[index] = Piece::Source(span.start..target.end);
                pieces.insert(index + 1, Piece::Source(target.end..span.end));
                return Some(index + 1);
            }
            if span.end == target.end {
                return Some(index + 1);
            }
            continue;
        }
        if last_overlap.is_some() {
            return last_overlap;
        }
        if span.start == target.end {
            return Some(index);
        }
    }
    last_overlap
}

fn remove_source_span(pieces: &mut Vec<Piece>, target: Range<usize>) -> bool {
    let mut found = false;
    let mut output = Vec::with_capacity(pieces.len() + 2);
    for piece in pieces.drain(..) {
        match piece {
            Piece::Source(span) if overlaps(&span, &target) => {
                found = true;
                if span.start < target.start {
                    output.push(Piece::Source(span.start..target.start));
                }
                if target.end < span.end {
                    output.push(Piece::Source(target.end..span.end));
                }
            }
            Piece::Generated {
                anchor: Some(anchor),
                ..
            } if overlaps(&anchor, &target) => {
                found = true;
            }
            piece => output.push(piece),
        }
    }
    *pieces = output;
    found
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
