use tombi_ast_syntax::SyntaxKind;

/// A compact parser event.
///
/// The TOML grammar always starts a parent before parsing its children, so it
/// does not need the `forward_parent` machinery used by more general event
/// parsers. Token events refer either to the lexer's token array or to the
/// parser's small synthetic-token array.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub(crate) struct Event(u64);

#[derive(Debug, Clone, Copy)]
pub(crate) enum TokenSource {
    Input { index: u32, count: u8 },
    Synthetic { index: u32 },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Step {
    Token {
        kind: SyntaxKind,
        source: TokenSource,
    },
    Enter {
        kind: SyntaxKind,
    },
    Exit,
}

impl Event {
    const TAG_MASK: u64 = 0x0000_0000_0000_0003;
    const KIND_MASK: u64 = 0x0000_0000_0000_00FC;
    const TOKEN_INDEX_MASK: u64 = 0x0000_00FF_FFFF_FF00;
    const TOKEN_COUNT_MASK: u64 = 0x0000_FF00_0000_0000;
    const SYNTHETIC_TOKEN_MASK: u64 = 0x0001_0000_0000_0000;

    const TAG_SHIFT: u32 = Self::TAG_MASK.trailing_zeros();
    const KIND_SHIFT: u32 = Self::KIND_MASK.trailing_zeros();
    const TOKEN_INDEX_SHIFT: u32 = Self::TOKEN_INDEX_MASK.trailing_zeros();
    const TOKEN_COUNT_SHIFT: u32 = Self::TOKEN_COUNT_MASK.trailing_zeros();

    const ENTER_EVENT: u64 = 0;
    const EXIT_EVENT: u64 = 1;
    const TOKEN_EVENT: u64 = 2;

    #[inline]
    pub(crate) fn tombstone() -> Self {
        Self::enter(SyntaxKind::TOMBSTONE)
    }

    #[inline]
    pub(crate) fn enter(kind: SyntaxKind) -> Self {
        debug_assert!((kind as u16) < 1 << 6);
        Self(((kind as u16 as u64) << Self::KIND_SHIFT) | Self::ENTER_EVENT)
    }

    #[inline]
    pub(crate) const fn exit() -> Self {
        Self(Self::EXIT_EVENT)
    }

    #[inline]
    pub(crate) fn input_token(kind: SyntaxKind, index: usize, count: u8) -> Self {
        debug_assert!((kind as u16) < 1 << 6);
        let index = u32::try_from(index).expect("too many lexer tokens");
        Self(
            ((kind as u16 as u64) << Self::KIND_SHIFT)
                | ((u64::from(index)) << Self::TOKEN_INDEX_SHIFT)
                | ((count as u64) << Self::TOKEN_COUNT_SHIFT)
                | Self::TOKEN_EVENT,
        )
    }

    #[inline]
    pub(crate) fn synthetic_token(kind: SyntaxKind, index: usize) -> Self {
        debug_assert!((kind as u16) < 1 << 6);
        let index = u32::try_from(index).expect("too many synthetic tokens");
        Self(
            ((kind as u16 as u64) << Self::KIND_SHIFT)
                | ((u64::from(index)) << Self::TOKEN_INDEX_SHIFT)
                | Self::SYNTHETIC_TOKEN_MASK
                | Self::TOKEN_EVENT,
        )
    }

    #[inline]
    pub(crate) fn set_kind(&mut self, kind: SyntaxKind) {
        debug_assert_eq!(self.0 & Self::TAG_MASK, Self::ENTER_EVENT);
        debug_assert!((kind as u16) < 1 << 6);
        self.0 = (self.0 & !Self::KIND_MASK) | ((kind as u16 as u64) << Self::KIND_SHIFT);
    }

    #[inline]
    fn step(self) -> Step {
        let kind = || (((self.0 & Self::KIND_MASK) >> Self::KIND_SHIFT) as u16).into();
        match (self.0 & Self::TAG_MASK) >> Self::TAG_SHIFT {
            Self::ENTER_EVENT => Step::Enter { kind: kind() },
            Self::EXIT_EVENT => Step::Exit,
            Self::TOKEN_EVENT => {
                let index = ((self.0 & Self::TOKEN_INDEX_MASK) >> Self::TOKEN_INDEX_SHIFT) as u32;
                let source = if self.0 & Self::SYNTHETIC_TOKEN_MASK == 0 {
                    TokenSource::Input {
                        index,
                        count: ((self.0 & Self::TOKEN_COUNT_MASK) >> Self::TOKEN_COUNT_SHIFT) as u8,
                    }
                } else {
                    TokenSource::Synthetic { index }
                };
                Step::Token {
                    kind: kind(),
                    source,
                }
            }
            _ => unreachable!(),
        }
    }
}

const _: () = assert!(std::mem::size_of::<Event>() == std::mem::size_of::<u64>());
const _: () = assert!((SyntaxKind::__LAST as u16) < 1 << 6);

/// Consume parser events in syntax-tree order without materializing another
/// event vector.
#[inline]
pub(super) fn process(events: &[Event], mut sink: impl FnMut(Step)) {
    for &event in events {
        let step = event.step();
        if !matches!(
            step,
            Step::Enter {
                kind: SyntaxKind::TOMBSTONE
            }
        ) {
            sink(step);
        }
    }
}
