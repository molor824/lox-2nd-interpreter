use std::ops::Range;

use crate::string_name::StringName;

use super::error::*;
use super::parse_node::*;
use super::parser::*;

impl<'a> Parser<'a> {
    pub(super) fn ident_or_keyword(&mut self) -> ParseOption<IdentKeyword> {
        self.skip();

        let mut range =
            if let Some((i, c)) = self.iter.next_if(|(_, c)| c.is_alphabetic() || *c == '_') {
                i..i + c.len_utf8()
            } else {
                return None;
            };

        while let Some((i, c)) = self.iter.next_if(|(_, c)| c.is_alphanumeric() || *c == '_') {
            range.end = i + c.len_utf8();
        }
        
        let ident = &self.source[range.clone()];

        if let Ok(k) = Keyword::try_from(ident) {
            return Some(ParseNode::new(range, IdentKeyword::Keyword(k)));
        }

        Some(ParseNode::new(range, IdentKeyword::Ident(ident.into())))
    }
    pub(super) fn ident(&mut self) -> ParseOption<StringName> {
        let old = self.iter.clone();
        let ident = self.ident_or_keyword()?;

        if let IdentKeyword::Ident(i) = ident.data {
            return Some(ident.convert(|_| i));
        }

        self.iter = old;
        None
    }
    pub(super) fn ident_if(
        &mut self,
        predicate: impl FnOnce(StringName) -> bool,
    ) -> ParseOption<StringName> {
        let old = self.iter.clone();

        let ident = self.ident()?;
        if predicate(ident.data) {
            return Some(ident);
        }

        self.iter = old;
        None
    }
    pub(super) fn ident_eq(&mut self, ident: StringName) -> ParseOption<StringName> {
        self.ident_if(|i| i == ident)
    }
    pub(super) fn keyword(&mut self) -> ParseOption<Keyword> {
        let old = self.iter.clone();

        let keyword = self.ident_or_keyword()?;

        if let IdentKeyword::Keyword(k) = keyword.data {
            return Some(keyword.convert(|_| k));
        }

        self.iter = old;
        None
    }
    pub(super) fn keyword_eq(&mut self, keyword: Keyword) -> ParseOption<Keyword> {
        self.keyword_if(|k| k == keyword)
    }
    pub(super) fn keyword_if(
        &mut self,
        predicate: impl FnOnce(Keyword) -> bool,
    ) -> ParseOption<Keyword> {
        let old = self.iter.clone();

        let keyword = self.keyword()?;
        if predicate(keyword.data) {
            return Some(keyword);
        }

        self.iter = old;
        None
    }
    pub(super) fn symbol(&mut self) -> ParseOption<Symbol> {
        self.skip();

        let mut last = self.iter.clone();
        let mut symbol = None;
        let mut range = 0..0;

        for len in 0..MAX_SYMBOL_LENGTH {
            if let Some((i, c)) = self.iter.next() {
                if len == 0 {
                    range.start = i;
                }
                range.end = i + c.len_utf8();

                let symbol_str = &self.source[range.clone()];

                if let Ok(s) = Symbol::try_from(symbol_str) {
                    symbol = Some(s);
                    last = self.iter.clone();
                }
            } else {
                break;
            }
        }

        self.iter = last;
        symbol.map(|s| ParseNode::new(range, s))
    }
    pub(super) fn symbol_if(
        &mut self,
        predicate: impl FnOnce(Symbol) -> bool,
    ) -> ParseOption<Symbol> {
        let old = self.iter.clone();

        let symbol = self.symbol()?;
        if predicate(symbol.data) {
            return Some(symbol);
        }

        self.iter = old;
        None
    }
    pub(super) fn symbol_eq(&mut self, symbol: Symbol) -> ParseOption<Symbol> {
        self.symbol_if(|s| s == symbol)
    }
    pub(super) fn skip(&mut self) {
        loop {
            if self.iter.next_if(|(_, c)| c.is_whitespace()).is_some() {
                continue;
            }

            let old = self.iter.clone();
            let is_slash = |(_, c): &(usize, char)| *c == '/';

            if self.iter.next_if(is_slash).is_some() {
                if self.iter.next_if(is_slash).is_some() {
                    for (_, c) in self.iter.by_ref() {
                        if c == '\n' {
                            break;
                        }
                    }
                    continue;
                }
                if self.iter.next_if(|(_, c)| *c == '*').is_some() {
                    while let Some((_, c)) = self.iter.next() {
                        if c == '*' && self.iter.next_if(|(_, c)| *c == '/').is_some() {
                            break;
                        }
                    }
                    continue;
                }
            }

            self.iter = old;
            break;
        }
    }
    fn parse_string(
        &mut self,
        mut range: Range<usize>,
        mut end_fn: impl FnMut(&mut Self) -> Option<Range<usize>>,
        is_raw: bool,
        incomplete_error: ErrorType,
    ) -> ParseResult<String> {
        let mut out = String::new();

        loop {
            if let Some(end) = end_fn(self) {
                return Ok(ParseNode::new(range.start..end.end, out));
            }

            if let Some((i, c)) = self.iter.next() {
                range.end = i + c.len_utf8();

                if c == '\r' {
                    if self.iter.next_if(|(i, c)| *c == '\n').is_some() {
                        range.end = i + c.len_utf8();
                    }
                    out.push('\n');
                    continue;
                }

                if c == '\\' && !is_raw {
                    if let Some((i, c)) = self.iter.next() {
                        range.end = i + c.len_utf8();
                        out.push(match c {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '0' => '\0',
                            '\\' => '\\',
                            '\'' => '\'',
                            '"' => '"',
                            'x' | 'u' => {
                                let is_ascii = c == 'x';
                                if !is_ascii {
                                    if let Some((i, c)) = self.iter.next_if(|(_, c)| *c == '{') {
                                        range.end = i + c.len_utf8();
                                    } else {
                                        return Err(Error::new(
                                            range,
                                            ErrorType::ExpectedLCurly,
                                        ));
                                    }
                                }

                                let mut char_code = 0;
                                for digit in 0..(if is_ascii { 2 } else { 4 }) {
                                    if let Some((i, c)) =
                                        self.iter.next_if(|(_, c)| c.is_ascii_hexdigit())
                                    {
                                        range.end = i + c.len_utf8();
                                        char_code = char_code * 16 + c.to_digit(16).unwrap_or(0);
                                    } else if !is_ascii {
                                        break;
                                    } else {
                                        return Err(Error::new(
                                            range,
                                            ErrorType::IncompleteCharCode,
                                        ));
                                    }
                                }

                                if !is_ascii {
                                    if let Some((i, c)) = self.iter.next_if(|(_, c)| *c == '}') {
                                        range.end = i + c.len_utf8();
                                    } else {
                                        return Err(Error::new(
                                            range,
                                            ErrorType::ExpectedRCurly,
                                        ));
                                    }
                                }

                                if let Some(c) = char::from_u32(char_code) {
                                    c
                                } else {
                                    return Err(Error::new(
                                        range,
                                        ErrorType::InvalidCharCode,
                                    ));
                                }
                            }
                            _ => return Err(Error::new(range, ErrorType::InvalidEscape)),
                        });
                    } else {
                        return Err(Error::new(range, ErrorType::IncompleteEscape));
                    }

                    continue;
                }

                out.push(c);
                continue;
            }

            break;
        }

        Err(Error::new(range, incomplete_error))
    }
    pub(super) fn char(&mut self) -> ParseResultOption<char> {
        self.skip();

        let range = if let Some((i, c)) = self.iter.next_if(|(_, c)| *c == '\'') {
            i..i + c.len_utf8()
        } else {
            return Ok(None);
        };

        let result = self.parse_string(
            range,
            |t| {
                t.iter
                    .next_if(|(_, c)| *c == '\'')
                    .map(|(i, c)| i..i + c.len_utf8())
            },
            false,
            ErrorType::IncompleteChar,
        )?;
        let mut chars = result.data.chars();

        let Some(ch) = chars.next() else {
            return Err(Error::new(result.range, ErrorType::EmptyChar));
        };
        if chars.next().is_some() {
            return Err(Error::new(result.range, ErrorType::TooManyChars));
        }

        Ok(Some(ParseNode::new(result.range, ch)))
    }
    pub(super) fn string(&mut self) -> ParseResultOption<String> {
        self.skip();

        let mut range = 0..0;

        let old = self.iter.clone();
        let nest_level = if let Some((i, c)) = self.iter.next_if(|(_, c)| *c == 'r') {
            range = i..i + c.len_utf8();

            let mut nest_level: u32 = 0;
            while let Some((i, c)) = self.iter.next_if(|(_, c)| *c == '(') {
                range.end = i + c.len_utf8();
                nest_level += 1;
            }

            Some(nest_level)
        } else {
            None
        };

        if let Some((i, c)) = self.iter.next_if(|(_, c)| *c == '"') {
            range.end = i + c.len_utf8();
            if nest_level.is_none() {
                range.start = i;
            }
        } else {
            self.iter = old;
            return Ok(None);
        }

        self.parse_string(
            range,
            |t| {
                let old = t.iter.clone();
                let mut range = if let Some((i, c)) = t.iter.next_if(|(_, c)| *c == '"') {
                    i..i + c.len_utf8()
                } else {
                    return None;
                };

                if let Some(nest_level) = nest_level {
                    let mut crnt_nest_level = 0;

                    loop {
                        if crnt_nest_level == nest_level {
                            break;
                        }

                        if let Some((i, c)) = t.iter.next_if(|(_, c)| *c == ')') {
                            range.end = i + c.len_utf8();
                            crnt_nest_level += 1;
                        } else {
                            t.iter = old;
                            return None;
                        }
                    }
                }

                Some(range)
            },
            nest_level.is_some(),
            ErrorType::IncompleteString,
        )
        .map(Some)
    }
    pub(super) fn number(&mut self) -> ParseResultOption<Number> {
        self.skip();

        let radix = self.radix();

        if let Some(radix) = radix {
            if let Some(integer) = self.integer(radix.data)? {
                Ok(Some(ParseNode::new(
                    radix.start()..integer.end(),
                    Number::Int(integer.data),
                )))
            } else {
                Err(Error::new(radix.range, ErrorType::ExpectedInteger))
            }
        } else {
            self.scientific()
        }
    }
    fn scientific(&mut self) -> ParseResultOption<Number> {
        let Some(mut number) = self.decimal()? else {
            return Ok(None);
        };

        if let Some((i, c)) = self.iter.next_if(|(_, c)| matches!(c, 'e' | 'E')) {
            number.range.end = i + c.len_utf8();
        } else {
            return Ok(Some(number));
        }

        let mut decimal = match number.data {
            Number::Int(i) => i as f64,
            Number::Real(r) => r,
        };

        let mut is_negative = false;
        if let Some((i, c)) = self.iter.next_if(|(_, c)| matches!(c, '+' | '-')) {
            number.range.end = i + c.len_utf8();
            is_negative = c == '-';
        }

        let Some(exponent) = self.integer(10)? else {
            return Err(Error::new(number.range, ErrorType::ExpectedInteger));
        };

        for _ in 0..exponent.data {
            decimal *= if is_negative { 0.1 } else { 10.0 };
        }

        Ok(Some(ParseNode::new(
            number.start()..exponent.end(),
            Number::Real(decimal),
        )))
    }
    fn decimal(&mut self) -> ParseResultOption<Number> {
        let Some(mut integer) = self.integer(10)? else {
            return Ok(None);
        };

        if self.iter.next_if(|(_, c)| *c == '.').is_some() {
            integer.range.end += 1;
        } else {
            return Ok(Some(integer.convert(Number::Int)));
        }

        let Some(decimal) = self.integer(10)? else {
            return Ok(Some(integer.convert(|i| Number::Real(i as f64))));
        };

        let mut real = decimal.data as f64;

        for _ in 0..decimal.range.len() {
            real *= 0.1;
        }
        real += integer.data as f64;

        Ok(Some(ParseNode::new(
            integer.start()..decimal.end(),
            Number::Real(real),
        )))
    }
    fn radix(&mut self) -> ParseOption<u32> {
        let mut radix = 10;
        let mut range = 0..0;
        let mut iter = self.iter.clone();

        if let Some((i, _)) = iter.next_if(|(_, c)| c.is_alphabetic()) {
            range.start = i;
            if let Some((i, c)) = iter.next() {
                range.end = i + c.len_utf8();
                match c {
                    'b' | 'B' => radix = 2,
                    'o' | 'O' => radix = 8,
                    'x' | 'X' => radix = 16,
                    _ => {}
                }
            }
        }
        if radix != 10 {
            self.iter = iter;
            return Some(ParseNode::new(range, radix));
        }

        None
    }
    fn integer(&mut self, radix: u32) -> ParseResultOption<u64> {
        let mut range = 0..0;
        let digit_predicate = |(_, c): &(usize, char)| c.is_digit(radix);

        if let Some((i, c)) = self.iter.next_if(digit_predicate) {
            range.start = i;
            range.end = i + c.len_utf8();

            while let Some((i, c)) = self.iter.next_if(digit_predicate) {
                range.end = i + c.len_utf8();
            }
        } else {
            return Ok(None);
        }

        let mut num: u64 = 0;
        for c in self.source[range.clone()].chars() {
            num = num
                .checked_mul(radix as u64)
                .and_then(|n| n.checked_add(c.to_digit(radix).unwrap() as u64))
                .ok_or(Error::new(range.clone(), ErrorType::IntOverflow))?;
        }

        Ok(Some(ParseNode::new(range, num)))
    }
}
