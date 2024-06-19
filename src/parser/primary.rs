use super::error::*;
use super::parse_node::*;
use super::parser::*;

impl<'a> Parser<'a> {
    pub(super) fn primary(&mut self) -> ParseResultOption<Expression> {
        if let Some(number) = self.number()? {
            return Ok(Some(number.convert(|n| match n {
                Number::Int(i) => Expression::Int(i),
                Number::Real(r) => Expression::Real(r),
            })));
        }
        if let Some(string) = self.string()? {
            return Ok(Some(string.convert(Expression::String)));
        }
        if let Some(character) = self.char()? {
            return Ok(Some(character.convert(Expression::Char)));
        }
        if let Some(boolean) = self.keyword_if(|k| matches!(k, Keyword::True | Keyword::False)) {
            return Ok(Some(
                boolean.convert(|k| Expression::Bool(k == Keyword::True)),
            ));
        }
        if let Some(none) = self.keyword_eq(Keyword::None) {
            return Ok(Some(none.convert(|_| Expression::None)));
        }
        if let Some(ident) = self.ident() {
            if ident.data.as_str() == "_" {
                return Err(Error::new(ident.range, ErrorType::UnderscoreVariable));
            }
            return Ok(Some(ident.convert(Expression::Variable)));
        }
        if let Some(group) = self.grouping()? {
            return Ok(Some(
                group.convert(|group| Expression::Grouping(group.into())),
            ));
        }
        if let Some(array) = self.array()? {
            return Ok(Some(array.convert(Expression::Array)));
        }
        if let Some(dict) = self.dictionary()? {
            return Ok(Some(dict.convert(Expression::Dictionary)));
        }

        Ok(None)
    }
    pub(super) fn dictionary(&mut self) -> ParseResultOption<Dictionary> {
        let Some(lcurly) = self.symbol_eq(Symbol::LCurlyBracket) else {
            return Ok(None);
        };

        let items = self.arguments(|p| {
            let key = if let Some(lsquare) = p.symbol_eq(Symbol::LSquareBracket) {
                let Some(key) = p.expression()? else {
                    return Err(Error::new(lsquare.range, ErrorType::ExpectedExpr));
                };
                let Some(rsquare) = p.symbol_eq(Symbol::RSquareBracket) else {
                    return Err(Error::new(
                        lsquare.start()..key.end(),
                        ErrorType::ExpectedRSquare,
                    ));
                };
                ParseNode::new(
                    lsquare.start()..rsquare.end(),
                    DictionaryKey::Expr(key.data),
                )
            } else {
                let Some(ident) = p.ident() else {
                    return Ok(None);
                };
                ident.convert(DictionaryKey::Ident)
            };

            let Some(assign) = p.symbol_eq(Symbol::Assign) else {
                return Err(Error::new(key.range.clone(), ErrorType::ExpectedAssign));
            };

            let Some(value) = p.expression()? else {
                return Err(Error::new(
                    key.start()..assign.end(),
                    ErrorType::ExpectedExpr,
                ));
            };

            Ok(Some(ParseNode::new(key.start()..value.end(), (key, value))))
        })?;

        let Some(rcurly) = self.symbol_eq(Symbol::RCurlyBracket) else {
            return Err(Error::new(
                lcurly.start()..items.last().map_or(lcurly.end(), |i| i.end()),
                ErrorType::ExpectedRCurly,
            ));
        };

        Ok(Some(ParseNode::new(
            lcurly.start()..rcurly.end(),
            items.into_iter().map(|i| i.data).collect(),
        )))
    }
    pub(super) fn grouping(&mut self) -> ParseResultOption<ParseNode<Expression>> {
        let Some(lparen) = self.symbol_eq(Symbol::LParenthesis) else {
            return Ok(None);
        };

        let Some(expr) = self.expression()? else {
            return Err(Error::new(lparen.range, ErrorType::ExpectedExpr));
        };

        let Some(rparen) = self.symbol_eq(Symbol::RParenthesis) else {
            return Err(Error::new(
                lparen.start()..expr.end(),
                ErrorType::ExpectedRParen,
            ));
        };

        Ok(Some(ParseNode::new(lparen.start()..rparen.end(), expr)))
    }
    pub(super) fn array(&mut self) -> ParseResultOption<Vec<ParseNode<Expression>>> {
        let Some(lbracket) = self.symbol_eq(Symbol::LSquareBracket) else {
            return Ok(None);
        };

        let args = self.arguments(|p| p.expression())?;

        let Some(rbracket) = self.symbol_eq(Symbol::RSquareBracket) else {
            return Err(Error::new(
                lbracket.start()..(args.last().map_or(lbracket.end(), |a| a.end())),
                ErrorType::ExpectedRSquare,
            ));
        };

        Ok(Some(ParseNode::new(lbracket.start()..rbracket.end(), args)))
    }
    pub(super) fn arguments<T>(
        &mut self,
        arg_fn: impl Fn(&mut Self) -> ParseResultOption<T>,
    ) -> Result<Vec<ParseNode<T>>> {
        let mut args = vec![];
        while let Some(arg) = arg_fn(self)? {
            args.push(arg);
            if self.symbol_eq(Symbol::Comma).is_none() {
                break;
            }
        }

        Ok(args)
    }
}
