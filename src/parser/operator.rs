use super::parser::*;
use super::parse_node::*;
use super::error::*;

// purely for operators precedence parsing

impl<'a> Parser<'a> {
    pub(super) fn operator(&mut self) -> ParseResultOption<AnyNode> {
        self.multiplication()
    }
    pub(super) fn multiplication(&mut self) -> ParseResultOption<AnyNode> {
        self.binary(|p| p.addition(), |s| matches!(s, Symbol::Mul | Symbol::Div | Symbol::Mod))
    }
    pub(super) fn addition(&mut self) -> ParseResultOption<AnyNode> {
        self.binary(|p| p.unary(), |s| matches!(s, Symbol::Add | Symbol::Sub))
    }
    pub(super) fn unary(&mut self) -> ParseResultOption<AnyNode> {
        let mut symbols = vec![];

        while let Some(s) = self.symbol_if(|s| matches!(s, Symbol::Not | Symbol::Add | Symbol::Sub)) {
            symbols.push(s);
        }

        let Some(mut operand) = self.power()? else {
            return Ok(None);
        };

        while let Some(s) = symbols.pop() {
            operand = ParseNode::new(
                s.start()..operand.end(),
                Unary {
                    operand,
                    operator: s.data,
                }
                .into(),
            );
        }

        Ok(Some(operand))
    }
    pub(super) fn power(&mut self) -> ParseResultOption<AnyNode> {
        self.binary(|p| p.suffix(), |s| s == Symbol::Pow)
    }
    pub(super) fn binary(
        &mut self,
        lower_fn: impl Fn(&mut Self) -> ParseResultOption<AnyNode>,
        op_check: impl Fn(Symbol) -> bool,
    ) -> ParseResultOption<AnyNode> {
        let mut left = match lower_fn(self)? {
            Some(l) => l,
            None => return Ok(None),
        };
        let mut parser = self.clone();
        loop {
            let Some(op) = parser.symbol() else {
                break;
            };
            if !op_check(op.data) {
                break;
            }

            let Some(right) = lower_fn(self)? else {
                return Err(SyntaxError::new(
                    left.start()..op.end(),
                    ErrorType::ExpectedExpr,
                ));
            };
            left = ParseNode::new(
                left.start()..right.end(),
                Binary {
                    left,
                    right,
                    operator: op.data,
                }
                .into(),
            );
        }
        Ok(Some(left))
    }
    // function call, field access, indexing
    pub(super) fn suffix(&mut self) -> ParseResultOption<AnyNode> {
        let Some(mut node) = self.primary()? else {
            return Ok(None);
        };

        while let Some(symbol) = self.symbol_if(|s| matches!(s, Symbol::Dot | Symbol::LSquareBracket | Symbol::LParenthesis)) {
            let range;
            let suffix;
            match symbol.data {
                Symbol::Dot => {
                    let Some(ident) = self.ident() else {
                        return Err(SyntaxError::new(symbol.range, ErrorType::ExpectedIdent));
                    };
                    range = symbol.start()..ident.end();
                    suffix = SuffixType::Property(ident.data);
                }
                Symbol::LSquareBracket => {
                    let Some(expr) = self.expression()? else {
                        return Err(SyntaxError::new(symbol.range, ErrorType::ExpectedExpr));
                    };
                    let Some(closing) = self.symbol_eq(Symbol::RSquareBracket) else {
                        return Err(SyntaxError::new(
                            symbol.start()..expr.end(),
                            ErrorType::ExpectedRSquare,
                        ));
                    };
                    range = symbol.start()..closing.end();
                    suffix = SuffixType::Index(expr);
                }
                Symbol::LParenthesis => {
                    let args = self.arguments(|p| p.expression())?;
                    let Some(closing) = self.symbol_eq(Symbol::RParenthesis) else {
                        return Err(SyntaxError::new(
                            symbol.start()..args.end(),
                            ErrorType::ExpectedRParen,
                        ));
                    };
                    range = symbol.start()..closing.end();
                    suffix = SuffixType::Call(args.data);
                }
                _ => unreachable!(),
            };
            let suffix = Suffix { node, suffix };
            node = ParseNode::new(range, suffix.into());
        }

        Ok(Some(node))
    }
}
