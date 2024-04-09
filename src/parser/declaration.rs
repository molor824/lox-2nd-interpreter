use crate::string_name::StringName;

use super::error::*;
use super::parse_node::*;
use super::parser::*;

impl<'a> Parser<'a> {
    pub(super) fn declaration(&mut self) -> ParseResultOption<Declaration> {
        if let Some(v) = self.var_decl()? {
            return Ok(Some(v.convert(Declaration::Var)));
        }
        Ok(None)
    }
    pub(super) fn func_decl(&mut self) -> ParseResultOption<FuncDecl> {
        let Some(decl_keyword) = self.keyword_eq(Keyword::Func) else {
            return Ok(None);
        };
        let Some(name) = self.suffix()? else {
            return Err(Error::new(decl_keyword.range, ErrorType::ExpectedFuncName));
        };
        todo!()
    }
    pub(super) fn var_name(&mut self) -> ParseResultOption<Option<StringName>> {
        let Some(name) = self.ident() else {
            return Ok(None);
        };
        Ok(Some(ParseNode::new(
            name.range,
            if name.data.as_str() == "_" {
                None
            } else {
                Some(name.data)
            },
        )))
    }
    pub(super) fn var_decl(&mut self) -> ParseResultOption<VarDecl> {
        let Some(decl_keyword) = self.keyword_eq(Keyword::Var) else {
            return Ok(None);
        };

        let name = if let Some(lsquare) = self.symbol_eq(Symbol::LSquareBracket) {
            let mut end = lsquare.end();
            let mut start_names = vec![];
            let mut end_names = vec![];
            let mut reached_dots = false;

            loop {
                end = if let Some(dots) = self.symbol_eq(Symbol::Dots) {
                    if reached_dots {
                        return Err(Error::new(dots.range, ErrorType::ExtraDots));
                    }
                    reached_dots = true;
                    dots.end()
                } else if let Some(name) = self.var_name()? {
                    let end = name.end();
                    (if reached_dots {
                        &mut end_names
                    } else {
                        &mut start_names
                    })
                    .push(name);
                    end
                } else {
                    break;
                };
                if let Some(comma) = self.symbol_eq(Symbol::Comma) {
                    end = comma.end();
                } else {
                    break;
                }
            }

            let Some(rsquare) = self.symbol_eq(Symbol::RSquareBracket) else {
                return Err(Error::new(lsquare.start()..end, ErrorType::ExpectedRSquare));
            };

            ParseNode::new(
                lsquare.start()..rsquare.end(),
                VarNameType::Array {
                    start_names,
                    end_names,
                },
            )
        } else if let Some(lcurly) = self.symbol_eq(Symbol::LCurlyBracket) {
            todo!()
        } else {
            let Some(name) = self.var_name()? else {
                return Err(Error::new(decl_keyword.range, ErrorType::ExpectedVarName));
            };
            name.convert(VarNameType::Ident)
        };

        let value = if let Some(eq) = self.symbol_eq(Symbol::Assign) {
            let Some(value) = self.expression()? else {
                return Err(Error::new(
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
                pattern: name,
                value,
            },
        )))
    }
}
