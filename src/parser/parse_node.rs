use std::{fmt, ops::Range};

use crate::string_name::*;

#[derive(Clone)]
pub struct ParseNode<T> {
    pub data: T,
    pub range: Range<usize>,
}
impl<T: fmt::Debug> fmt::Debug for ParseNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}
impl<T> ParseNode<T> {
    pub const fn new(range: Range<usize>, data: T) -> Self {
        Self { data, range }
    }
    pub const fn start(&self) -> usize {
        self.range.start
    }
    pub const fn end(&self) -> usize {
        self.range.end
    }
    pub fn convert<T2>(self, conversion: impl FnOnce(T) -> T2) -> ParseNode<T2> {
        ParseNode::new(self.range, conversion(self.data))
    }
    pub fn replace<T2>(self, value: T2) -> ParseNode<T2> {
        ParseNode::new(self.range, value)
    }
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Var(VarDecl),
    Func(FuncDecl),
}
#[derive(Debug, Clone)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
    If(IfStatement),
    Block(Block),
}
#[derive(Debug, Clone)]
pub enum Expression {
    None,
    Bool(bool),
    Int(u64),
    Real(f64),
    String(String),
    Char(char),
    Array(Vec<ParseNode<Expression>>),
    Dictionary(Dictionary),
    Variable(StringName),
    Binary(Binary),
    Unary(Unary),
    Suffix(Suffix),
    Grouping(Box<ParseNode<Expression>>),
}
#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: ParseNode<Expression>,
    pub loop_block: ParseNode<Block>,
    pub on_break: Option<ParseNode<Block>>,
    pub on_continue: Option<ParseNode<Block>>,
}
#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: ParseNode<Expression>,
    pub met_block: ParseNode<Block>,
    pub else_block: Option<ParseNode<ElseBlock>>,
}
#[derive(Debug, Clone)]
pub enum ElseBlock {
    Block(Block),
    If(Box<IfStatement>),
}
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub pattern: ParseNode<VarNameType>,
    pub value: Option<ParseNode<Expression>>,
}
#[derive(Debug, Clone)]
pub enum VarNameType {
    Ident(Option<StringName>), // single identifier
    Array { // unpacks an array into variables. eg. `var [a, b, _, d] = ["a", 3, "something", false]` a = "a", b = 3, d = false
        start_names: Vec<ParseNode<Option<StringName>>>, // `var [a, b, ..., c, d] = [1, 2, 3, 4, 5]` a = 1, b = 2, c = 4, d = 5
        end_names: Vec<ParseNode<Option<StringName>>>, // `var [..., a, b] = [0, 1, 2, 3, 4]` a = 3, b = 4
    },
    Object(Vec<(StringName, ParseNode<StringName>)>), // unpacks an object into variables. eq. `var {a, b, c} = {a: 1, b: 2, c: 3}` a = 1, b = 2, c = 3
    // you can also assign custom names to the object keys. eq. `var {x: a, y: b} = {x: 0, y: 1}` a = 0, b = 1
}
pub type Dictionary = Vec<ParseNode<(ParseNode<DictionaryKey>, ParseNode<Expression>)>>;
#[derive(Debug, Clone)]
pub enum DictionaryKey {
    Ident(StringName),
    Expr(Expression),
}
#[derive(Debug, Clone)]
pub enum SuffixType {
    Call(Vec<ParseNode<Expression>>),
    Index(Box<ParseNode<Expression>>),
    Property(StringName),
}
#[derive(Debug, Clone)]
pub struct Suffix {
    pub node: Box<ParseNode<Expression>>,
    pub suffix: SuffixType,
}
#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<ParseNode<Expression>>,
    pub right: Box<ParseNode<Expression>>,
    pub operator: BinaryOperator,
}
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Not,
    And,
    Or,
    Xor,
    LogicalAnd,
    LogicalOr,
    LeftShift,
    RightShift,
    Eq,
    NotEq,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    Assign,
}
impl From<Keyword> for BinaryOperator {
    fn from(keyword: Keyword) -> Self {
        match keyword {
            Keyword::And => BinaryOperator::LogicalAnd,
            Keyword::Or => BinaryOperator::LogicalOr,
            _ => panic!("invalid keyword: {:?}", keyword),
        }
    }
}
impl From<Symbol> for BinaryOperator {
    fn from(symbol: Symbol) -> Self {
        match symbol {
            Symbol::Add => BinaryOperator::Add,
            Symbol::Sub => BinaryOperator::Sub,
            Symbol::Mul => BinaryOperator::Mul,
            Symbol::Div => BinaryOperator::Div,
            Symbol::Mod => BinaryOperator::Mod,
            Symbol::Pow => BinaryOperator::Pow,
            Symbol::Not => BinaryOperator::Not,
            Symbol::And => BinaryOperator::And,
            Symbol::Or => BinaryOperator::Or,
            Symbol::Xor => BinaryOperator::Xor,
            Symbol::LeftShift => BinaryOperator::LeftShift,
            Symbol::RightShift => BinaryOperator::RightShift,
            Symbol::Eq => BinaryOperator::Eq,
            Symbol::NotEq => BinaryOperator::NotEq,
            Symbol::Greater => BinaryOperator::Greater,
            Symbol::Less => BinaryOperator::Less,
            Symbol::GreaterEq => BinaryOperator::GreaterEq,
            Symbol::LessEq => BinaryOperator::LessEq,
            Symbol::Assign => BinaryOperator::Assign,
            _ => panic!("invalid symbol: {:?}", symbol),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Unary {
    pub operand: Box<ParseNode<Expression>>,
    pub operator: UnaryOperator,
}
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    Negate,
}
impl From<Symbol> for UnaryOperator {
    fn from(symbol: Symbol) -> Self {
        match symbol {
            Symbol::Not => UnaryOperator::Not,
            Symbol::Sub => UnaryOperator::Negate,
            _ => panic!("invalid symbol: {:?}", symbol),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum Number {
    Int(u64),
    Real(f64),
}
#[derive(Debug, Clone, Copy)]
pub enum IdentKeyword {
    Ident(StringName),
    Keyword(Keyword),
}
#[derive(Debug, Clone)]
pub struct FuncDecl {
    pub name: Box<ParseNode<Expression>>,
    pub value: Box<ParseNode<Expression>>,
}
pub type Block = Vec<ParseNode<Statement>>;
#[derive(Debug, Clone)]
pub struct Closure {
    pub args: Vec<ParseNode<StringName>>,
    pub block: Block,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    And,
    Or,
    None,
    True,
    False,
    Var,
    Func,
    If,
    Elif,
    Else,
    For,
    While,
    Break,
    Continue,
    OnBreak,
    OnContinue,
}
impl TryFrom<&str> for Keyword {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "none" => Keyword::None,
            "true" => Keyword::True,
            "false" => Keyword::False,
            "let" => Keyword::Var,
            "func" => Keyword::Func,
            "if" => Keyword::If,
            "elif" => Keyword::Elif,
            "else" => Keyword::Else,
            "while" => Keyword::While,
            "for" => Keyword::For,
            "break" => Keyword::Break,
            "continue" => Keyword::Continue,
            "onbreak" => Keyword::OnBreak,
            "oncontinue" => Keyword::OnContinue,
            "and" => Keyword::And,
            "or" => Keyword::Or,    
            _ => return Err(()),
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Not,
    And,
    Or,
    Xor,
    LeftShift,
    RightShift,
    Eq,
    NotEq,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    LParenthesis,
    RParenthesis,
    LSquareBracket,
    RSquareBracket,
    LCurlyBracket,
    RCurlyBracket,
    Dot,
    Dots,
    Assign,
    Comma,
    Semicolon,
    Colon,
    Return,
}
impl TryFrom<&str> for Symbol {
    type Error = ();
    fn try_from(symbol: &str) -> Result<Self, Self::Error> {
        Ok(match symbol {
            "+" => Symbol::Add,
            "-" => Symbol::Sub,
            "*" => Symbol::Mul,
            "/" => Symbol::Div,
            "%" => Symbol::Mod,
            "**" => Symbol::Pow,
            "!" => Symbol::Not,
            "&" => Symbol::And,
            "|" => Symbol::Or,
            "^" => Symbol::Xor,
            "<<" => Symbol::LeftShift,
            ">>" => Symbol::RightShift,
            "==" => Symbol::Eq,
            "!=" => Symbol::NotEq,
            ">" => Symbol::Greater,
            "<" => Symbol::Less,
            ">=" => Symbol::GreaterEq,
            "<=" => Symbol::LessEq,
            "(" => Symbol::LParenthesis,
            ")" => Symbol::RParenthesis,
            "[" => Symbol::LSquareBracket,
            "]" => Symbol::RSquareBracket,
            "{" => Symbol::LCurlyBracket,
            "}" => Symbol::RCurlyBracket,
            "." => Symbol::Dot,
            "..." => Symbol::Dots,
            "=" => Symbol::Assign,
            "," => Symbol::Comma,
            ";" => Symbol::Semicolon,
            ":" => Symbol::Colon,
            "<-" => Symbol::Return,    
            _ => return Err(()),
        })
    }
}
pub const MAX_SYMBOL_LENGTH: usize = 5;
