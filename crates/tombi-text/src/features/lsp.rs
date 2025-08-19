use unicode_segmentation::UnicodeSegmentation;

impl From<crate::Position> for tower_lsp::lsp_types::Position {
    fn from(val: crate::Position) -> Self {
        tower_lsp::lsp_types::Position {
            line: val.line,
            character: val.column,
        }
    }
}

impl From<tower_lsp::lsp_types::Position> for crate::Position {
    fn from(val: tower_lsp::lsp_types::Position) -> Self {
        Self::new(val.line, val.character)
    }
}

impl From<crate::Range> for tower_lsp::lsp_types::Range {
    fn from(val: crate::Range) -> Self {
        tower_lsp::lsp_types::Range {
            start: val.start.into(),
            end: val.end.into(),
        }
    }
}

impl From<tower_lsp::lsp_types::Range> for crate::Range {
    fn from(val: tower_lsp::lsp_types::Range) -> Self {
        Self::new(val.start.into(), val.end.into())
    }
}

impl crate::Offset {
    pub fn from_source(source: &str, position: tower_lsp::lsp_types::Position) -> Self {
        let mut line = 0;
        let mut column = 0;
        let mut offset = 0;
        for c in UnicodeSegmentation::graphemes(source, true) {
            if line == position.line && column == position.character {
                return Self::new(offset as u32);
            }
            if matches!(c, "\n" | "\r\n") {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
            offset += c.len();
        }
        Self::new(offset as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_from_source_multibyte() {
        let source = "こんにちは🦅世界";

        // デバッグ用：各文字のバイト数を確認
        println!("Source: {source}");
        for (i, c) in UnicodeSegmentation::graphemes(source, true).enumerate() {
            println!("Character {}: '{}' ({} bytes)", i, c, c.len());
        }

        // 最初の文字（こ）の位置
        let pos = tower_lsp::lsp_types::Position::new(0, 0);
        let offset = crate::Offset::from_source(source, pos);
        println!("Position (0, 0) -> offset: {}", offset.raw);
        assert_eq!(offset.raw, 0);

        // 絵文字（🦅）の位置
        let pos = tower_lsp::lsp_types::Position::new(0, 5);
        let offset = crate::Offset::from_source(source, pos);
        println!("Position (0, 5) -> offset: {}", offset.raw);
        assert_eq!(offset.raw, 15); // "こんにちは" は15バイト

        // 最後の文字（界）の位置
        let pos = tower_lsp::lsp_types::Position::new(0, 7);
        let offset = crate::Offset::from_source(source, pos);
        println!("Position (0, 7) -> offset: {}", offset.raw);
        assert_eq!(offset.raw, 22); // "こんにちは��世" は22バイト
    }
}
