// SPDX-License-Identifier: Apache-2.0 or MIT
// Copyright 2021 [rijenkii](https://github.com/rijenkii)
//
// Ref: https://github.com/rijenkii/xdg-user-rs/blob/v0.2.1/src/parser.rs

pub(crate) struct LineParser<'a> {
    line: &'a mut Vec<u8>,
    pos: usize,
}

impl<'a> LineParser<'a> {
    pub(crate) fn new(line: &'a mut Vec<u8>) -> Self {
        Self { line, pos: 0 }
    }

    pub(crate) fn parse(mut self) -> Option<(&'a [u8], &'a [u8])> {
        self.skip_whitespace()?;
        let key = self.parse_key()?;
        self.skip_whitespace()?;
        self.expect_byte(b'=')?;
        self.skip_whitespace()?;
        let val = self.parse_val()?;
        Some((&self.line[key], &self.line[val]))
    }

    fn skip_whitespace(&mut self) -> Option<()> {
        while self.line.get(self.pos)? == &b' ' || self.line.get(self.pos)? == &b'\t' {
            self.pos += 1;
        }
        Some(())
    }

    fn parse_key(&mut self) -> Option<std::ops::Range<usize>> {
        let mut range = self.pos..0;
        while self.line.get(self.pos)? != &b' '
            && self.line.get(self.pos)? != &b'\t'
            && self.line.get(self.pos)? != &b'='
        {
            self.pos += 1;
        }
        range.end = self.pos;
        Some(range)
    }

    fn expect_byte(&mut self, b: u8) -> Option<()> {
        if self.line.get(self.pos)? == &b {
            self.pos += 1;
            Some(())
        } else {
            None
        }
    }

    fn parse_val(&mut self) -> Option<std::ops::Range<usize>> {
        self.expect_byte(b'"')?;
        let mut range = self.pos..0;

        // shamelessly ~~stolen~~ adapted from Vec::retain
        {
            let len = self.line.len();
            let mut del = 0;
            loop {
                match self.line.get(self.pos)? {
                    b'"' => break,
                    b'\\' => {
                        del += 1;
                        self.pos += 2;
                    }
                    _ => {
                        self.line.swap(self.pos - del, self.pos);
                        self.pos += 1;
                    }
                }
            }
            if del > 0 {
                self.line.truncate(len - del);
            }
        }

        range.end = self.pos;
        Some(range)
    }
}
