use std::iter::Peekable;

use crate::source::SourceIter;

use super::{error::*, parse_node::*};

pub type ParseResultOption<T> = Result<Option<ParseNode<T>>>;
pub type ParseResult<T> = Result<ParseNode<T>>;
pub type ParseOption<T> = Option<ParseNode<T>>;

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    pub(super) source: &'a str,
    pub(super) iter: Peekable<SourceIter<'a>>,
}
// contains mostly miscallenous methods that are often used across multiple stages of parsing
// such as symbols, identifiers, types etc.
impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            iter: SourceIter::from(source).peekable(),
            source,
        }
    }
    pub fn parse(&mut self) -> Result<Vec<ParseNode<Statement>>> {
        if let Some(stmts) = self.statements()? {
            return Ok(stmts.data.statements);
        }
        self.skip();
        if let Some((i, c)) = self.iter.next() {
            return Err(Error::new(i..i + c.len_utf8(), ErrorType::ExpectedEOF));
        }
        Ok(vec![])
    }
    pub(super) fn expression(&mut self) -> ParseResultOption<Expression> {
        self.operator()
    }
}
