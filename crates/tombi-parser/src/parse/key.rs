use tombi_syntax::{SyntaxKind::*, T};

use super::Parse;
use crate::{ErrorKind::*, parser::Parser, token_set::TS_KEY_FIRST};

impl Parse for tombi_ast::Keys {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();
        if eat_keys(p) {
            m.complete(p, KEYS);
        } else {
            m.complete(p, INVALID_TOKEN);
        };
    }
}

fn eat_keys(p: &mut Parser<'_>) -> bool {
    if p.nth_at(1, T![.]) {
        let mut has_error = false;
        // Dotted keys Mode
        loop {
            has_error |= !eat_key(p);
            if !p.eat(T![.]) {
                break;
            }

            if !p.at_ts(TS_KEY_FIRST) {
                p.error(crate::Error::new(
                    ForbiddenKeysLastPeriod,
                    p.current_range(),
                ));
                break;
            }
        }
        !has_error
    } else {
        eat_key(p)
    }
}

pub fn eat_key(p: &mut Parser<'_>) -> bool {
    let kind = p.current();
    match kind {
        BARE_KEY | BASIC_STRING | LITERAL_STRING => {
            let m = p.start();
            p.bump(kind);
            m.complete(p, kind);
            true
        }
        INTEGER_DEC | BOOLEAN | LOCAL_DATE => {
            let m = p.start();
            p.bump_remap(BARE_KEY);
            m.complete(p, BARE_KEY);
            true
        }
        FLOAT => {
            p.bump_float_key();
            true
        }
        _ => {
            let m = p.start();
            p.error(crate::Error::new(ExpectedKey, p.current_range()));
            // p.bump_remap(INVALID_TOKEN);
            m.complete(p, INVALID_TOKEN);
            false
        }
    }
}
