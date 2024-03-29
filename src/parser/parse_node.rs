use once_cell::sync::Lazy;
use std::{collections::HashMap, fmt, ops::Range};

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
    pub fn start_mut(&mut self) -> &mut usize {
        &mut self.range.start
    }
    pub fn end_mut(&mut self) -> &mut usize {
        &mut self.range.end
    }
    pub fn convert<T2>(self, conversion: impl FnOnce(T) -> T2) -> ParseNode<T2> {
        ParseNode::new(self.range, conversion(self.data))
    }
    pub fn replace<T2>(self, value: T2) -> ParseNode<T2> {
        ParseNode::new(self.range, value)
    }
}

#[derive(Debug, Clone)]
pub enum AnyNode {
    None,
    Bool(bool),
    Int(u64),
    Real(f64),
    String(String),
    Char(char),
    Tuple(Vec<ParseNode<AnyNode>>),
    Ident(StringName),
    Binary {
        left: Box<ParseNode<AnyNode>>,
        right: Box<ParseNode<AnyNode>>,
        operator: Symbol,
    },
    Unary {
        operand: Box<ParseNode<AnyNode>>,
        operator: Symbol,
    },
    Suffix {
        node: Box<ParseNode<AnyNode>>,
        suffix: Box<SuffixType>,
    },
    Grouping(Box<ParseNode<AnyNode>>),
    VarDecl {
        name: StringName,
        value: Option<Box<ParseNode<AnyNode>>>,
    },
}
impl From<Suffix> for AnyNode {
    fn from(suffix: Suffix) -> Self {
        Self::Suffix {
            suffix: Box::new(suffix.suffix),
            node: Box::new(suffix.node),
        }
    }
}
impl From<Binary> for AnyNode {
    fn from(value: Binary) -> Self {
        Self::Binary {
            left: Box::new(value.left),
            right: Box::new(value.right),
            operator: value.operator,
        }
    }
}
impl From<Unary> for AnyNode {
    fn from(value: Unary) -> Self {
        Self::Unary {
            operand: Box::new(value.operand),
            operator: value.operator,
        }
    }
}
impl From<VarDecl> for AnyNode {
    fn from(value: VarDecl) -> Self {
        Self::VarDecl {
            name: value.name,
            value: value.value.map(Box::new),
        }
    }
}
impl From<Number> for AnyNode {
    fn from(value: Number) -> Self {
        match value {
            Number::Int(i) => AnyNode::Int(i),
            Number::Real(r) => AnyNode::Real(r),
        }
    }
}
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: StringName,
    pub value: Option<ParseNode<AnyNode>>,
}
#[derive(Debug, Clone)]
pub enum SuffixType {
    Call(Vec<ParseNode<AnyNode>>),
    Index(ParseNode<AnyNode>),
    Property(StringName),
}
#[derive(Debug, Clone)]
pub struct Suffix {
    pub node: ParseNode<AnyNode>,
    pub suffix: SuffixType,
}
#[derive(Debug, Clone)]
pub struct Binary {
    pub left: ParseNode<AnyNode>,
    pub right: ParseNode<AnyNode>,
    pub operator: Symbol,
}
#[derive(Debug, Clone)]
pub struct Unary {
    pub operand: ParseNode<AnyNode>,
    pub operator: Symbol,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    None,
    True,
    False,
    Var,
    Func,
}
pub static STR_TO_KEYWORD: Lazy<HashMap<&str, Keyword>> = Lazy::new(|| {
    [
        ("none", Keyword::None),
        ("true", Keyword::True),
        ("false", Keyword::False),
        ("var", Keyword::Var),
        ("func", Keyword::Func),
    ]
    .into()
});
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
    Assign,
    Comma,
    Semicolon,
    Colon,
}
pub static STR_TO_SYMBOL: Lazy<HashMap<&str, Symbol>> = Lazy::new(|| {
    [
        ("+", Symbol::Add),
        ("-", Symbol::Sub),
        ("*", Symbol::Mul),
        ("/", Symbol::Div),
        ("%", Symbol::Mod),
        ("**", Symbol::Pow),
        ("!", Symbol::Not),
        ("&", Symbol::And),
        ("|", Symbol::Or),
        ("^", Symbol::Xor),
        ("==", Symbol::Eq),
        ("!=", Symbol::NotEq),
        (">", Symbol::Greater),
        ("<", Symbol::Less),
        (">=", Symbol::GreaterEq),
        ("<=", Symbol::LessEq),
        ("(", Symbol::LParenthesis),
        (")", Symbol::RParenthesis),
        ("[", Symbol::LSquareBracket),
        ("]", Symbol::RSquareBracket),
        ("{", Symbol::LCurlyBracket),
        ("}", Symbol::RCurlyBracket),
        (".", Symbol::Dot),
        ("=", Symbol::Assign),
        (",", Symbol::Comma),
        (";", Symbol::Semicolon),
        (":", Symbol::Colon),
    ]
    .into()
});
pub static MAX_SYMBOL_LENGTH: Lazy<usize> = Lazy::new(|| {
    STR_TO_SYMBOL
        .keys()
        .map(|s| s.len())
        .max()
        .expect("symbol map has no symbol keys!")
});
