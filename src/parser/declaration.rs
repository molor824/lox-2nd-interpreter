use super::error::*;
use super::parse_node::*;
use super::parser::*;

impl<'a> Parser<'a> {
    pub fn declaration(&mut self) -> ParseResultOption<AnyNode> {
        if let Some(v) = self.var_decl()? {
            return Ok(Some(v.convert(Into::into)));
        }
        Ok(None)
    }
    pub fn var_decl(&mut self) -> ParseResultOption<VarDecl> {
        let Some(decl_keyword) = self.keyword_eq(Keyword::Var) else {
            return Ok(None);
        };

        // check if its a tuple unpacking
        let Some(name) = self.ident() else {
            return Err(SyntaxError::new(
                decl_keyword.range,
                ErrorType::ExpectedVarName,
            ));
        };

        let value = if let Some(eq) = self.symbol_eq(Symbol::Assign) {
            let Some(value) = self.expression()? else {
                return Err(SyntaxError::new(
                    decl_keyword.start()..eq.end(),
                    ErrorType::ExpectedExpr,
                ));
            };
            Some(value)
        } else {
            None
        };

        Ok(Some(ParseNode::new(
            decl_keyword.start()..value.as_ref().map(|v| v.end()).unwrap_or(name.end()),
            VarDecl {
                name: name.data,
                value,
            },
        )))
    }
}
