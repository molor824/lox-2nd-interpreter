use crate::string_name::StringName;

use super::error::*;
use super::parse_node::*;
use super::parser::*;

impl<'a> Parser<'a> {
    pub(super) fn declaration(&mut self) -> ParseResultOption<Declaration> {
        if let Some(v) = self.var_decl()? {
            return Ok(Some(v.convert(Declaration::Var)));
        }
        if let Some(f) = self.func_decl()? {
            return Ok(Some(f.convert(Declaration::Func)));
        }
        Ok(None)
    }
    pub(super) fn func_decl(&mut self) -> ParseResultOption<FuncDecl> {
        let Some(decl_keyword) = self.keyword_eq(Keyword::Func) else {
            return Ok(None);
        };
        let Some(name) = self.ident() else {
            return Err(Error::new(decl_keyword.range, ErrorType::ExpectedIdent));
        };
        let Some(lparen) = self.symbol_eq(Symbol::LParenthesis) else {
            return Err(Error::new(
                decl_keyword.start()..name.end(),
                ErrorType::ExpectedLParen,
            ));
        };
        let params = self.arguments(|p| p.func_param())?;
        let Some(rparen) = self.symbol_eq(Symbol::RParenthesis) else {
            return Err(Error::new(
                lparen.start()..params.last().map_or(lparen.end(), |p| p.end()),
                ErrorType::ExpectedRParen,
            ));
        };
        let block = if let Some(block) = self.block()? {
            block.convert(FuncBlock::Block)
        } else if let Some(eq) = self.symbol_eq(Symbol::RightArrow) {
            let Some(expr) = self.expression()? else {
                return Err(Error::new(eq.range, ErrorType::ExpectedExpr));
            };
            expr.convert(FuncBlock::ReturnExpr)
        } else {
            return Err(Error::new(
                decl_keyword.start()..rparen.end(),
                ErrorType::ExpectedFuncBlock,
            ));
        };
        Ok(Some(ParseNode::new(
            decl_keyword.start()..block.end(),
            FuncDecl {
                name,
                params,
                block,
            },
        )))
    }
    // pretty much variable declaration without the var keyword
    pub(super) fn func_param(&mut self) -> ParseResultOption<VarDecl> {
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
                    if reached_dots {
                        end_names.push(name);
                    } else {
                        start_names.push(name);
                    }
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
                return Ok(None);
            };
            name.convert(VarNameType::Ident)
        };

        let value = if let Some(eq) = self.symbol_eq(Symbol::Assign) {
            let Some(value) = self.expression()? else {
                return Err(Error::new(name.start()..eq.end(), ErrorType::ExpectedExpr));
            };
            Some(value)
        } else {
            None
        };

        Ok(Some(ParseNode::new(
            name.start()..value.as_ref().map(|v| v.end()).unwrap_or(name.end()),
            VarDecl {
                pattern: name,
                value,
            },
        )))
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
        let Some(decl) = self.func_param()? else {
            return Err(Error::new(decl_keyword.range, ErrorType::ExpectedVarName));
        };
        
        Ok(Some(ParseNode::new(decl_keyword.start()..decl.end(), decl.data)))
    }
}
