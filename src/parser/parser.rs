use std::iter::Peekable;

use crate::source::SourceIter;

use super::{error::*, parse_node::*};

pub type ParseResultOption<T> = Result<Option<ParseNode<T>>, SyntaxError>;
pub type ParseResult<T> = Result<ParseNode<T>, SyntaxError>;
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
    pub fn parse(&mut self) -> Result<Vec<ParseNode<AnyNode>>, SyntaxError> {
        self.statements(|p| {
            if let Some(node) = p.declaration()? {
                return Ok(Some(node));
            }
            p.expression()
        })
        .map(|node| node.data)
    }
    pub(super) fn expression(&mut self) -> ParseResultOption<AnyNode> {
        self.operator()
    }
    // statements one after another, used in blocks and top level code
    pub(super) fn statements(
        &mut self,
        statement: impl Fn(&mut Parser) -> ParseResultOption<AnyNode>,
    ) -> ParseResult<Vec<ParseNode<AnyNode>>> {
        let mut nodes = vec![];
        let mut range = 0..0;

        while let Some(node) = statement(self)? {
            if self.symbol_eq(Symbol::Semicolon).is_none() {
                return Err(SyntaxError::new(node.range, ErrorType::ExpectedSeperator));
            }
            if nodes.is_empty() {
                range.start = node.range.start;
            }
            range.end = node.range.end;
            nodes.push(node);
        }

        Ok(ParseNode::new(range, nodes))
    }
}
