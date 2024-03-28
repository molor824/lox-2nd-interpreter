use super::parser::*;
use super::parse_node::*;
use super::error::*;

impl<'a> Parser<'a> {
    pub(super) fn primary(&mut self) -> ParseResultOption<AnyNode> {
        if let Some(tuple) = self.tuple()? {
            return Ok(Some(tuple));
        }
        if let Some(number) = self.number()? {
            return Ok(Some(number.convert(Into::into)));
        }
        if let Some(ident) = self.ident() {
            return Ok(Some(ident.convert(AnyNode::Ident)));
        }
        if let Some(keyword) = self.keyword_if(|k| matches!(k, Keyword::True | Keyword::False)) {
            return Ok(Some(keyword.convert(|k| AnyNode::Bool(k == Keyword::True))))
        }
        if let Some(none) = self.keyword_eq(Keyword::None) {
            return Ok(Some(none.convert(|_| AnyNode::None)));
        }
        if let Some(string) = self.string()? {
            return Ok(Some(string.convert(AnyNode::String)));
        }
        if let Some(character) = self.char()? {
            return Ok(Some(character.convert(AnyNode::Char)));
        }

        Ok(None)
    }
    pub(super) fn block(&mut self) -> ParseResultOption<Vec<ParseNode<AnyNode>>> {
        let Some(lbrace) = self.symbol_eq(Symbol::LCurlyBracket) else {
            return Ok(None);
        };

        todo!()
    }
    pub(super) fn arguments<T>(
        &mut self,
        arg_fn: impl Fn(&mut Self) -> ParseResultOption<T>,
    ) -> ParseResult<Vec<ParseNode<T>>> {
        let mut args = vec![];
        let mut range = 0..0;
        while let Some(arg) = arg_fn(self)? {
            if args.is_empty() {
                range = arg.range.clone();
            }
            range.end = arg.end();
            args.push(arg);

            if let Some(comma) = self.symbol_eq(Symbol::Comma) {
                range.end = comma.end();
            } else {
                return Ok(ParseNode::new(range, args));
            }
        }

        Ok(ParseNode::new(range, args))
    }
    pub(super) fn tuple(&mut self) -> ParseResultOption<AnyNode> {
        let Some(lparen) = self.symbol_eq(Symbol::LParenthesis) else {
            return Ok(None);
        };

        let args = self.arguments(Self::expression)?;
        let Some(rparen) = self.symbol_eq(Symbol::RParenthesis) else {
            return Err(SyntaxError::new(
                lparen.start()..args.end(),
                ErrorType::ExpectedRParen,
            ));
        };

        let range = lparen.start()..rparen.end();

        if args.data.len() == 1 {
            return Ok(Some(ParseNode::new(range, AnyNode::Grouping(args.data[0].clone().into()))));
        }

        Ok(Some(ParseNode::new(
            range,
            AnyNode::Tuple(args.data),
        )))
    }
}
