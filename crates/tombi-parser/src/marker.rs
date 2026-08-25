use drop_bomb::DropBomb;
use tombi_ast_syntax::SyntaxKind;

use crate::parser::Parser;

pub(crate) struct Marker {
    event_index: u32,
    bomb: DropBomb,
}

impl Marker {
    pub fn new(event_index: u32) -> Marker {
        Marker {
            event_index,
            bomb: DropBomb::new("Marker must be either completed or abandoned"),
        }
    }

    /// Finishes the syntax tree node and assigns `kind` to it.
    pub(crate) fn complete(mut self, p: &mut Parser<'_>, kind: SyntaxKind) {
        self.bomb.defuse();
        let idx = self.event_index as usize;
        p.events[idx].set_kind(kind);
        p.push_event(crate::event::Event::exit());
    }
}
