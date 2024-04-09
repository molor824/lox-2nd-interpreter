use super::error::*;
use super::parse_node::*;
use super::parser::*;

impl<'a> Parser<'a> {
    pub(super) fn statements(&mut self) -> ParseResultOption<Block> {
        let mut stmts: Option<ParseNode<Vec<_>>> = None;
        while let Some(stmt) = self.statement()? {
            if let Some(stmts) = &mut stmts {
                stmts.range.end = stmt.end();
                stmts.data.push(stmt);
            } else {
                stmts = Some(ParseNode::new(stmt.range.clone(), vec![stmt]));
            }
        }
        Ok(stmts)
    }
    fn check_semicolon(&mut self, value: ParseNode<Statement>) -> ParseResultOption<Statement> {
        let Some(semicolon) = self.symbol_eq(Symbol::Semicolon) else {
            return Err(Error::new(value.range, ErrorType::ExpectedSemicolon));
        };
        Ok(Some(ParseNode::new(
            value.start()..semicolon.end(),
            value.data,
        )))
    }
    pub(super) fn statement(&mut self) -> ParseResultOption<Statement> {
        if let Some(declaration) = self.declaration()? {
            return self.check_semicolon(declaration.convert(Statement::Declaration));
        }
        if let Some(if_stmt) = self.if_statement()? {
            return Ok(Some(if_stmt.convert(Statement::If)));
        }
        // if let Some(while_stmt) = self.while_statement()? {}
        if let Some(block) = self.block()? {
            return Ok(Some(block.convert(Statement::Block)));
        }
        if let Some(expr) = self.expression()? {
            return self.check_semicolon(expr.convert(Statement::Expression));
        }

        Ok(None)
    }
    pub(super) fn while_statement(&mut self) -> ParseResultOption<WhileStatement> {
        let Some(while_keyword) = self.keyword_eq(Keyword::While) else {
            return Ok(None);
        };
        let Some(condition) = self.expression()? else {
            return Err(Error::new(while_keyword.range, ErrorType::ExpectedExpr));
        };
        let Some(block) = self.block()? else {
            return Err(Error::new(
                while_keyword.start()..condition.end(),
                ErrorType::ExpectedBlock,
            ));
        };

        todo!()
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
        Ok(statements.map(|s| ParseNode::new(lcurly.start()..rcurly.end(), s.data)))
    }
}
