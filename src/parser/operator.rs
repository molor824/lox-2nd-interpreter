use super::error::*;
use super::parse_node::*;
use super::parser::*;

// purely for operators precedence parsing

impl<'a> Parser<'a> {
    pub(super) fn operator(&mut self) -> ParseResultOption<Expression> {
        self.logical_or()
    }
    fn is_assign_operator(symbol: Symbol) -> bool {
        matches!(symbol, Symbol::Add | Symbol::Sub | Symbol::Mul | Symbol::Div | Symbol::Mod | Symbol::LeftShift | Symbol::RightShift)
    }
    pub(super) fn assign(&mut self) -> ParseResultOption<Expression> {
        let Some(mut left) = self.logical_or()? else {
            return Ok(None);
        };
        loop {
            if let Some(assignable_operator) 
        }
    }
    pub(super) fn logical_or(&mut self) -> ParseResultOption<Expression> {
        self.binary_kw(|p| p.logical_and(), |k| matches!(k, Keyword::Or))
    }
    pub(super) fn logical_and(&mut self) -> ParseResultOption<Expression> {
        self.binary_kw(|p| p.bitwise_or(), |k| matches!(k, Keyword::And))
    }
    pub(super) fn bitwise_or(&mut self) -> ParseResultOption<Expression> {
        self.binary(|p| p.bitwise_xor(), |s| matches!(s, Symbol::Or))
    }
    pub(super) fn bitwise_xor(&mut self) -> ParseResultOption<Expression> {
        self.binary(|p| p.bitwise_and(), |s| matches!(s, Symbol::Xor))
    }
    pub(super) fn bitwise_and(&mut self) -> ParseResultOption<Expression> {
        self.binary(|p| p.equality(), |s| matches!(s, Symbol::And))
    }
    pub(super) fn equality(&mut self) -> ParseResultOption<Expression> {
        self.binary(
            |p| p.comparison(),
            |s| matches!(s, Symbol::Eq | Symbol::NotEq),
        )
    }
    pub(super) fn comparison(&mut self) -> ParseResultOption<Expression> {
        self.binary(
            |p| p.bit_shift(),
            |s| {
                matches!(
                    s,
                    Symbol::Less | Symbol::Greater | Symbol::LessEq | Symbol::GreaterEq
                )
            },
        )
    }
    pub(super) fn bit_shift(&mut self) -> ParseResultOption<Expression> {
        self.binary(
            |p| p.addition(),
            |s| matches!(s, Symbol::LeftShift | Symbol::RightShift),
        )
    }
    pub(super) fn addition(&mut self) -> ParseResultOption<Expression> {
        self.binary(
            |p| p.multiplication(),
            |s| matches!(s, Symbol::Add | Symbol::Sub),
        )
    }
    pub(super) fn multiplication(&mut self) -> ParseResultOption<Expression> {
        self.binary(
            |p| p.unary(),
            |s| matches!(s, Symbol::Mul | Symbol::Div | Symbol::Mod),
        )
    }
    pub(super) fn unary(&mut self) -> ParseResultOption<Expression> {
        let mut symbols = vec![];

        while let Some(s) = self.symbol_if(|s| matches!(s, Symbol::Not | Symbol::Add | Symbol::Sub))
        {
            symbols.push(s);
        }

        let Some(mut operand) = self.power()? else {
            return Ok(None);
        };

        while let Some(s) = symbols.pop() {
            operand = ParseNode::new(
                s.start()..operand.end(),
                Expression::Unary(Unary {
                    operand: operand.into(),
                    operator: s.data,
                }),
            );
        }

        Ok(Some(operand))
    }
    pub(super) fn power(&mut self) -> ParseResultOption<Expression> {
        self.binary(|p| p.suffix(), |s| s == Symbol::Pow)
    }
    pub(super) fn binary_kw(
        &mut self,
        lower_fn: impl Fn(&mut Self) -> ParseResultOption<Expression>,
        kw_check: impl Fn(Keyword) -> bool,
    ) -> ParseResultOption<Expression> {
        let mut left = match lower_fn(self)? {
            Some(l) => l,
            None => return Ok(None),
        };
        let mut parser = self.clone();
        while let Some(kw) = parser.keyword_if(kw_check) {
            let Some(right) = lower_fn(self)? else {
                return Err(Error::new(left.start()..kw.end(), ErrorType::ExpectedExpr));
            };
            left = ParseNode::new(
                left.start()..right.end(),
                Expression::Binary(Binary {
                    left: left.into(),
                    right: right.into(),
                    operator: SymbolKeyword::Keyword(kw.data),
                }),
            );
        }
        Ok(Some(left))
    }
    pub(super) fn binary(
        &mut self,
        lower_fn: impl Fn(&mut Self) -> ParseResultOption<Expression>,
        op_check: impl Fn(Symbol) -> bool,
    ) -> ParseResultOption<Expression> {
        let mut left = match lower_fn(self)? {
            Some(l) => l,
            None => return Ok(None),
        };
        while let Some(op) = self.symbol_if(op_check) {
            let Some(right) = lower_fn(self)? else {
                return Err(Error::new(left.start()..op.end(), ErrorType::ExpectedExpr));
            };
            left = ParseNode::new(
                left.start()..right.end(),
                Expression::Binary(Binary {
                    left: left.into(),
                    right: right.into(),
                    operator: SymbolKeyword::Symbol(op.data),
                }),
            );
        }
        Ok(Some(left))
    }
    // function call, field access, indexing
    pub(super) fn suffix(&mut self) -> ParseResultOption<Expression> {
        let Some(mut node) = self.primary()? else {
            return Ok(None);
        };

        while let Some(symbol) = self.symbol_if(|s| {
            matches!(
                s,
                Symbol::Dot | Symbol::LSquareBracket | Symbol::LParenthesis
            )
        }) {
            let range;
            let suffix;
            match symbol.data {
                Symbol::Dot => {
                    let Some(ident) = self.ident() else {
                        return Err(Error::new(symbol.range, ErrorType::ExpectedIdent));
                    };
                    range = symbol.start()..ident.end();
                    suffix = SuffixType::Property(ident.data);
                }
                Symbol::LSquareBracket => {
                    let Some(expr) = self.expression()? else {
                        return Err(Error::new(symbol.range, ErrorType::ExpectedExpr));
                    };
                    let Some(closing) = self.symbol_eq(Symbol::RSquareBracket) else {
                        return Err(Error::new(
                            symbol.start()..expr.end(),
                            ErrorType::ExpectedRSquare,
                        ));
                    };
                    range = symbol.start()..closing.end();
                    suffix = SuffixType::Index(expr.into());
                }
                Symbol::LParenthesis => {
                    let args = self.arguments(|p| p.expression())?;
                    let Some(closing) = self.symbol_eq(Symbol::RParenthesis) else {
                        return Err(Error::new(
                            symbol.start()..args.last().map_or(symbol.end(), |i| i.end()),
                            ErrorType::ExpectedRParen,
                        ));
                    };
                    range = symbol.start()..closing.end();
                    suffix = SuffixType::Call(args);
                }
                _ => unreachable!(),
            };
            let suffix = Suffix {
                node: node.into(),
                suffix,
            };
            node = ParseNode::new(range, Expression::Suffix(suffix));
        }

        Ok(Some(node))
    }
}
