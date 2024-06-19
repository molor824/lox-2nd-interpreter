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

#[derive(Clone)]
pub enum Declaration {
    Var(VarDecl),
    Func(FuncDecl),
}
impl fmt::Debug for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(decl) => decl.fmt(f),
            Self::Func(decl) => decl.fmt(f),
        }
    }
}
#[derive(Clone)]
pub struct FuncDecl {
    pub name: ParseNode<StringName>,
    pub params: Vec<ParseNode<VarDecl>>,
    pub block: ParseNode<FuncBlock>,
}
impl fmt::Debug for FuncDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(funcdecl: {}; params:", self.name.data)?;
        for param in &self.params {
            write!(f, " {:?}", param.data)?;
        }
        write!(f, "; block: {:?})", self.block.data)?;

        Ok(())
    }
}
#[derive(Clone)]
pub enum FuncBlock {
    Block(Block),
    ReturnExpr(Expression),
}
impl fmt::Debug for FuncBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Block(block) => block.fmt(f),
            Self::ReturnExpr(expr) => write!(f, "(return: {:?})", expr),
        }
    }
}
#[derive(Clone)]
pub struct VarDecl {
    pub pattern: ParseNode<VarNameType>,
    pub value: Option<ParseNode<Expression>>,
}
impl fmt::Debug for VarDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(vardecl: {:?}", self.pattern.data)?;
        if let Some(value) = &self.value {
            write!(f, "; assign: {:?}", value.data)?;
        }
        write!(f, ")")
    }
}
#[derive(Clone)]
pub enum VarNameType {
    Ident(Option<StringName>), // single identifier
    Array {
        // unpacks an array into variables. eg. `var [a, b, _, d] = ["a", 3, "something", false]` a = "a", b = 3, d = false
        start_names: Vec<ParseNode<Option<StringName>>>, // `var [a, b, ..., c, d] = [1, 2, 3, 4, 5]` a = 1, b = 2, c = 4, d = 5
        end_names: Vec<ParseNode<Option<StringName>>>, // `var [..., a, b] = [0, 1, 2, 3, 4]` a = 3, b = 4
    },
    Object(Vec<(StringName, ParseNode<StringName>)>), // unpacks an object into variables. eq. `var {a, b, c} = {a: 1, b: 2, c: 3}` a = 1, b = 2, c = 3
                                                      // you can also assign custom names to the object keys. eq. `var {x: a, y: b} = {x: 0, y: 1}` a = 0, b = 1
}
impl fmt::Debug for VarNameType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarNameType::Ident(name) => write!(f, "(ident: {})", name.map_or("_", |n| n.as_str())),
            VarNameType::Array {
                start_names,
                end_names,
            } => {
                write!(f, "(start names:")?;
                for name in start_names {
                    write!(f, " {}", name.data.map_or("_", |x| x.as_str()))?;
                }
                if end_names.is_empty() {
                    return write!(f, ")");
                }
                write!(f, "; end names:")?;
                for name in end_names {
                    write!(f, " {}", name.data.map_or("_", |x| x.as_str()))?;
                }
                write!(f, ")")
            }
            VarNameType::Object(names) => {
                write!(f, "(object:")?;
                for (field, name) in names {
                    write!(f, " ({} {})", field, name.data)?;
                }
                write!(f, ")")
            }
        }
    }
}
#[derive(Clone)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
    If(IfStatement),
    While(WhileStatement),
    Block(Block),
}
impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declaration(decl) => decl.fmt(f),
            Self::Expression(expr) => expr.fmt(f),
            Self::If(if_stmt) => if_stmt.fmt(f),
            Self::While(while_stmt) => while_stmt.fmt(f),
            Self::Block(block) => block.fmt(f),
        }
    }
}
#[derive(Clone)]
pub struct Block {
    pub statements: Vec<ParseNode<Statement>>,
}
impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "(block:")?;
        for statement in &self.statements {
            let result = format!("{:?}", statement.data);
            for line in result.lines() {
                writeln!(f, "  {}", line)?;
            }
        }
        write!(f, ")")
    }
}
#[derive(Clone)]
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
    Assign(Assign),
    Unary(Unary),
    Suffix(Suffix),
    Grouping(Box<ParseNode<Expression>>),
}
impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Bool(b) => b.fmt(f),
            Self::Int(i) => i.fmt(f),
            Self::Real(r) => r.fmt(f),
            Self::String(s) => s.fmt(f),
            Self::Char(c) => c.fmt(f),
            Self::Array(arr) => {
                write!(f, "(array:")?;
                for expr in arr {
                    write!(f, " {:?}", expr.data)?;
                }
                write!(f, ")")
            }
            Self::Dictionary(dict) => {
                write!(f, "(dictionary:")?;
                for expr in dict {
                    write!(f, " ({:?} {:?})", expr.0.data, expr.1.data)?;
                }
                write!(f, ")")
            }
            Self::Variable(name) => write!(f, "{}", name),
            Self::Binary(bin) => bin.fmt(f),
            Self::Unary(un) => un.fmt(f),
            Self::Suffix(suf) => suf.fmt(f),
            Self::Grouping(expr) => expr.fmt(f),
        }
    }
}
#[derive(Clone)]
pub struct Closure {
    pub args: Vec<ParseNode<StringName>>,
    pub block: Block,
}
#[derive(Clone)]
pub struct WhileStatement {
    pub condition: ParseNode<Expression>,
    pub loop_block: ParseNode<Block>,
    pub on_break: Option<ParseNode<Block>>,
    pub on_continue: Option<ParseNode<Block>>,
}
impl fmt::Debug for WhileStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(while: (condition: {:?}) {:?}",
            self.condition.data, self.loop_block.data
        )?;
        if let Some(block) = &self.on_break {
            write!(f, "; on_break: {:?}", block.data)?;
        }
        if let Some(block) = &self.on_continue {
            write!(f, "; on_continue: {:?}", block.data)?;
        }
        write!(f, ")")
    }
}
#[derive(Clone)]
pub struct IfStatement {
    pub condition: ParseNode<Expression>,
    pub met_block: ParseNode<Block>,
    pub else_block: Option<ParseNode<ElseBlock>>,
}
impl fmt::Debug for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(if: (condition: {:?}) {:?}",
            self.condition.data, self.met_block.data
        )?;
        if let Some(else_block) = &self.else_block {
            write!(f, "; else {:?}", else_block.data)?;
        }
        write!(f, ")")
    }
}
#[derive(Clone)]
pub enum ElseBlock {
    Block(Block),
    If(Box<IfStatement>),
}
impl fmt::Debug for ElseBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElseBlock::Block(block) => block.fmt(f),
            ElseBlock::If(if_statement) => (*if_statement).fmt(f),
        }
    }
}
pub type Dictionary = Vec<(ParseNode<DictionaryKey>, ParseNode<Expression>)>;
#[derive(Clone)]
pub enum DictionaryKey {
    Ident(StringName),
    Expr(Expression),
}
impl fmt::Debug for DictionaryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DictionaryKey::Ident(name) => write!(f, "{}", *name),
            DictionaryKey::Expr(expr) => write!(f, "{:?}", *expr),
        }
    }
}
#[derive(Clone)]
pub enum SuffixType {
    Call(Vec<ParseNode<Expression>>),
    Index(Box<ParseNode<Expression>>),
    Property(StringName),
}
#[derive(Clone)]
pub struct Suffix {
    pub node: Box<ParseNode<Expression>>,
    pub suffix: SuffixType,
}
impl fmt::Debug for Suffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(node: {:?}; suffix: ", self.node.data)?;
        match &self.suffix {
            SuffixType::Call(args) => {
                write!(f, "(call:")?;
                for arg in args {
                    write!(f, " {:?}", arg.data)?;
                }
                write!(f, ")")
            }
            SuffixType::Index(expr) => write!(f, "(index: {:?})", expr.data),
            SuffixType::Property(name) => write!(f, "(property: {})", *name),
        }?;
        write!(f, ")")
    }
}
#[derive(Clone)]
pub struct Assign {
    pub left: Box<ParseNode<Expression>>,
}
#[derive(Clone)]
pub struct Binary {
    pub left: Box<ParseNode<Expression>>,
    pub right: Box<ParseNode<Expression>>,
    pub operator: SymbolKeyword,
}
impl fmt::Debug for Binary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(binary: {:?} {:?} {:?})",
            self.operator, self.left.data, self.right.data
        )
    }
}
#[derive(Clone)]
pub enum SymbolKeyword {
    Symbol(Symbol),
    Keyword(Keyword),
}
impl fmt::Debug for SymbolKeyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbol(s) => s.fmt(f),
            Self::Keyword(k) => k.fmt(f),
        }
    }
}
#[derive(Clone)]
pub struct Unary {
    pub operand: Box<ParseNode<Expression>>,
    pub operator: Symbol,
}
impl fmt::Debug for Unary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(unary {:?} {:?})", self.operator, self.operand.data)
    }
}
#[derive(Clone, Copy)]
pub enum Number {
    Int(u64),
    Real(f64),
}
#[derive(Clone, Copy)]
pub enum IdentKeyword {
    Ident(StringName),
    Keyword(Keyword),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
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
    And,
    Or,
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
    LeftArrow,
    RightArrow,
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
            "<-" => Symbol::LeftArrow,
            "->" => Symbol::RightArrow,
            _ => return Err(()),
        })
    }
}
pub const MAX_SYMBOL_LENGTH: usize = 5;
