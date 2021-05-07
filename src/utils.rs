use lsp_types::*;
use rnix::TextRange;
use std::convert::TryFrom;

pub fn lookup_pos(code: &str, pos: Position) -> Option<usize> {
    let mut lines = code.split('\n');

    let mut offset = 0;
    for _ in 0..pos.line {
        let line = lines.next()?;

        offset += line.len() + 1;
    }

    lines.next().and_then(|line| {
        Some(
            offset
                + line
                    .chars()
                    .take(usize::try_from(pos.character).ok()?)
                    .map(char::len_utf8)
                    .sum::<usize>(),
        )
    })
}
pub fn offset_to_pos(code: &str, offset: usize) -> Position {
    let start_of_line = code[..offset].rfind('\n').map_or(0, |n| n + 1);
    Position {
        line: code[..start_of_line].chars().filter(|&c| c == '\n').count() as u64,
        character: code[start_of_line..offset]
            .chars()
            .map(|c| c.len_utf16() as u64)
            .sum(),
    }
}
pub fn range(code: &str, range: TextRange) -> Range {
    Range {
        start: offset_to_pos(code, usize::from(range.start())),
        end: offset_to_pos(code, usize::from(range.end())),
    }
}
