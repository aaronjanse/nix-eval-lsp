// https://github.com/nix-community/rnix-lsp

// MIT License

// Copyright (c) 2020 jD91mZM2

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use lsp_types::*;
use rnix::TextRange;
use std::convert::TryFrom;

pub fn pos_to_offset(code: &str, pos: Position) -> Option<usize> {
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

pub fn range(code: &str, range: TextRange) -> Range {
    fn offset_to_pos(code: &str, offset: usize) -> Position {
        let start_of_line = code[..offset].rfind('\n').map_or(0, |n| n + 1);
        Position {
            line: code[..start_of_line].chars().filter(|&c| c == '\n').count() as u64,
            character: code[start_of_line..offset]
                .chars()
                .map(|c| c.len_utf16() as u64)
                .sum(),
        }
    }

    Range {
        start: offset_to_pos(code, usize::from(range.start())),
        end: offset_to_pos(code, usize::from(range.end())),
    }
}
