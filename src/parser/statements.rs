use super::error::*;
use super::parse_node::*;
use super::parser::*;

impl<'a> Parser<'a> {
    fn requires_semicolon(statement: &Statement) -> bool {
        matches!(
            statement,
            Statement::Expression(_) | Statement::Declaration(Declaration::Var(_))
        )
    }
    pub(super) fn statements(&mut self) -> ParseResultOption<Block> {
        let mut statements: Option<ParseNode<Block>> = None;
        while let Some(stmt) = self.statement()? {
            let end = if Self::requires_semicolon(&stmt.data) {
                let Some(semicolon) = self.symbol_eq(Symbol::Semicolon) else {
                    return Err(Error::new(stmt.range, ErrorType::ExpectedSemicolon));
                };
                semicolon.end()
            } else {
                stmt.end()
            };
            if let Some(stmts) = &mut statements {
                stmts.range.end = end;
                stmts.data.statements.push(stmt);
            } else {
                statements = Some(ParseNode::new(
                    stmt.range.clone(),
                    Block {
                        statements: vec![stmt],
                    },
                ));
            }
        }
        Ok(statements)
    }
    pub(super) fn statement(&mut self) -> ParseResultOption<Statement> {
        if let Some(declaration) = self.declaration()? {
            return Ok(Some(declaration.convert(Statement::Declaration)));
        }
        if let Some(if_stmt) = self.if_statement()? {
            return Ok(Some(if_stmt.convert(Statement::If)));
        }
        if let Some(while_stmt) = self.while_statement()? {
            return Ok(Some(while_stmt.convert(Statement::While)))
        }
        if let Some(block) = self.block()? {
            return Ok(Some(block.convert(Statement::Block)));
        }
        if let Some(expr) = self.expression()? {
            return Ok(Some(expr.convert(Statement::Expression)));
        }

        Ok(None)
    }
    fn onbreak_block(&mut self) -> ParseResultOption<Block> {
        let Some(onbreak_keyword) = self.keyword_eq(Keyword::OnBreak) else {
            return Ok(None);
        };
        let Some(block) = self.block()? else {
            return Err(Error::new(onbreak_keyword.range, ErrorType::ExpectedBlock));
        };

        Ok(Some(ParseNode::new(onbreak_keyword.start()..block.end(), block.data)))
    }
    fn oncontinue_block(&mut self) -> ParseResultOption<Block> {
        let Some(oncontinue_keyword) = self.keyword_eq(Keyword::OnContinue) else {
            return Ok(None);
        };
        let Some(block) = self.block()? else {
            return Err(Error::new(oncontinue_keyword.range, ErrorType::ExpectedBlock));
        };

        Ok(Some(ParseNode::new(oncontinue_keyword.start()..block.end(), block.data)))
    }
    pub(super) fn while_statement(&mut self) -> ParseResultOption<WhileStatement> {
        let Some(while_keyword) = self.keyword_eq(Keyword::While) else {
            return Ok(None);
        };
        let Some(condition) = self.expression()? else {
            return Err(Error::new(while_keyword.range, ErrorType::ExpectedExpr));
        };
        let Some(loop_block) = self.block()? else {
            return Err(Error::new(
                while_keyword.start()..condition.end(),
                ErrorType::ExpectedBlock,
            ));
        };
        let mut end = loop_block.end();
        let mut on_break = None;
        let mut on_continue = None;

        if let Some(onbreak) = self.onbreak_block()? {
            end = onbreak.end();
            on_break = Some(onbreak);
            if let Some(oncontinue) = self.oncontinue_block()? {
                end = oncontinue.end();
                on_continue = Some(oncontinue);
            }
        } else if let Some(oncontinue) = self.oncontinue_block()? {
            end = oncontinue.end();
            on_continue = Some(oncontinue);
            if let Some(onbreak) = self.onbreak_block()? {
                end = onbreak.end();
                on_break = Some(onbreak);
            }
        }
        
        Ok(Some(ParseNode::new(while_keyword.start()..end, WhileStatement {
            condition,
            loop_block,
            on_break,
            on_continue,
        })))
    }
    pub(super) fn if_statement(&mut self) -> ParseResultOption<IfStatement> {
        let Some(if_keyword) = self.keyword_eq(Keyword::If) else {
            return Ok(None);
        };
        let Some(condition) = self.expression()? else {
            return Err(Error::new(if_keyword.range, ErrorType::ExpectedExpr));
        };
        let Some(block) = self.block()? else {
            return Err(Error::new(
                if_keyword.start()..condition.end(),
                ErrorType::ExpectedBlock,
            ));
        };
        let mut end = block.end();
        let mut elifs = vec![];
        while let Some(elif) = self.keyword_eq(Keyword::Elif) {
            let Some(condition) = self.expression()? else {
                return Err(Error::new(elif.range, ErrorType::ExpectedExpr));
            };
            let Some(block) = self.block()? else {
                return Err(Error::new(
                    elif.start()..condition.end(),
                    ErrorType::ExpectedBlock,
                ));
            };
            end = block.end();
            elifs.push(ParseNode::new(
                elif.start()..block.end(),
                (condition, block),
            ));
        }
        let mut else_block: Option<ParseNode<ElseBlock>> = None;
        if let Some(else_keyword) = self.keyword_eq(Keyword::Else) {
            let Some(block) = self.block()? else {
                return Err(Error::new(else_keyword.range, ErrorType::ExpectedBlock));
            };
            end = block.end();
            else_block = Some(ParseNode::new(
                else_keyword.start()..block.end(),
                ElseBlock::Block(block.data),
            ));
        }

        while let Some(elif) = elifs.pop() {
            else_block = Some(ParseNode::new(
                elif.start()..else_block.as_ref().map_or(elif.end(), |b| b.end()),
                ElseBlock::If(
                    IfStatement {
                        condition: elif.data.0,
                        met_block: elif.data.1,
                        else_block,
                    }
                    .into(),
                ),
            ));
        }

        Ok(Some(ParseNode::new(
            if_keyword.start()..end,
            IfStatement {
                condition,
                met_block: block,
                else_block,
            },
        )))
    }
    pub(super) fn block(&mut self) -> ParseResultOption<Block> {
        let Some(lcurly) = self.symbol_eq(Symbol::LCurlyBracket) else {
            return Ok(None);
        };
        let statements = self.statements()?;
        let Some(rcurly) = self.symbol_eq(Symbol::RCurlyBracket) else {
            return Err(Error::new(
                statements
                    .as_ref()
                    .map(|s| lcurly.start()..s.end())
                    .unwrap_or(lcurly.range.clone()),
                ErrorType::ExpectedRCurly,
            ));
        };
        Ok(Some(ParseNode::new(
            lcurly.start()..rcurly.end(),
            statements.map_or(Block { statements: vec![] }, |stmts| stmts.data),
        )))
    }
}
