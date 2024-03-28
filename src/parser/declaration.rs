use super::parser::*;
use super::parse_node::*;
use super::error::*;

impl<'a> Parser<'a> {
    pub fn declaration(&mut self) -> ParseResultOption<AnyNode> {
        if let Some(v) = self.var_decl()? {
            return Ok(Some(v.convert(Into::into)));
        }
        Ok(None)
    }
    pub fn var_decl(&mut self) -> ParseResultOption<VarDecl> {
        let Some(decl_keyword) = self.keyword_eq(Keyword::VarDecl) else {
            return Ok(None);
        };

        // check if its a tuple unpacking
        let name = if let Some(lparen) = self.symbol_eq(Symbol::LParenthesis) {
            let args = self.arguments(|p| Ok(p.ident()))?;
            let Some(rparen) = self.symbol_eq(Symbol::RParenthesis) else {
                return Err(SyntaxError::new(
                    lparen.start()..args.end(),
                    ErrorType::ExpectedRParen,
                ));
            };

            // if its 1, then its just a regular variable name
            let range = lparen.start()..rparen.end();
            if args.data.len() != 1 {
                ParseNode::new(range, VarName::Tuple(args.data))
            } else {
                ParseNode::new(range, VarName::Name(args.data[0].data))
            }
        } else if let Some(name) = self.ident() {
            ParseNode::new(name.range, VarName::Name(name.data))
        } else {
            return Err(SyntaxError::new(
                decl_keyword.range,
                ErrorType::ExpectedVarName,
            ));
        };

        let value = if let Some(eq) = self.symbol_eq(Symbol::Assign) {
            let Some(value) = self.expression()? else {
                return Err(SyntaxError::new(decl_keyword.start()..eq.end(), ErrorType::ExpectedExpr));
            };
            Some(value)
        } else {
            None
        };

        Ok(Some(ParseNode::new(
            decl_keyword.start()..value.as_ref().map(|v| v.end()).unwrap_or(name.end()),
            VarDecl { name, value },
        )))
    }
}
