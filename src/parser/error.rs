use std::fmt;
use std::fmt::Write;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub range: Range<usize>,
    pub error: ErrorType,
}
impl SyntaxError {
    pub fn new(range: Range<usize>, error: ErrorType) -> Self {
        Self { range, error }
    }
}
impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}
impl DisplayError for SyntaxError {
    fn display(&self, source: &str, message: &mut impl Write) -> fmt::Result {
        let (start, end) = cursor_locations(self.range.clone(), source).unwrap();
        let mut lines = source.lines().skip(start.0);

        write!(
            message,
            "Error at line: {}, column: {}\n{}\n\n",
            start.0 + 1,
            start.1,
            self
        )?;

        for i in start.0..=end.0 {
            let line = if let Some(s) = lines.next() {
                s
            } else {
                continue;
            };
            let line_num = format!("{:>3} | ", i + 1);

            writeln!(message, "{}{}", line_num, line)?;

            for _ in 0..line_num.len() {
                write!(message, " ")?;
            }

            let mut offset = 0;
            let mut end_offset = line.len();

            if i == start.0 {
                offset = start.1;
            }
            if i == end.0 {
                end_offset = end.1 + 1;
            }
            for i in 0..line.len() {
                write!(
                    message,
                    "{}",
                    if i < offset || i > end_offset {
                        ' '
                    } else {
                        '^'
                    }
                )?;
            }

            if i != end.0 {
                writeln!(message)?;
            }
        }

        Ok(())
    }
}
fn cursor_locations(range: Range<usize>, text: &str) -> Option<((usize, usize), (usize, usize))> {
    let mut start = (0, 0);

    for c in text.get(..range.start)?.chars() {
        start.1 += 1;
        if c == '\n' {
            start.0 += 1;
            start.1 = 0;
        }
    }

    let mut end = start;
    for c in text.get(range)?.chars() {
        end.1 += 1;
        if c == '\n' {
            end.0 += 1;
            end.1 = 0;
        }
    }

    Some((start, end))
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    IntOverflow,
    ExpectedLParen,
    ExpectedRParen,
    ExpectedLCurly,
    ExpectedRCurly,
    ExpectedLSquare,
    ExpectedRSquare,
    ExpectedNumber,
    ExpectedInteger,
    ExpectedIdent,
    ExpectedExpr,
    ExpectedVarName,
    ExpectedSeperator,
    IncompleteString,
    IncompleteCharCode,
    IncompleteEscape,
    IncompleteChar,
    InvalidEscape,
    InvalidCharCode,
    EmptyChar,
    TooManyChars,
    UnexpectedSymbol,
    TrailingComma,
}
impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::IntOverflow => write!(f, "Integer overflow"),
            ErrorType::ExpectedLParen => write!(f, "Expected a '('"),
            ErrorType::ExpectedRParen => write!(f, "Expected a ')'"),
            ErrorType::ExpectedLCurly => write!(f, "Expected a '{{'"),
            ErrorType::ExpectedRCurly => write!(f, "Expected a '}}'"),
            ErrorType::ExpectedLSquare => write!(f, "Expected a '['"),
            ErrorType::ExpectedRSquare => write!(f, "Expected a ']'"),
            ErrorType::ExpectedNumber => write!(f, "Expected a number"),
            ErrorType::ExpectedInteger => write!(f, "Expected an integer"),
            ErrorType::ExpectedIdent => write!(f, "Expected an identifier"),
            ErrorType::ExpectedSeperator => write!(f, "Expected a ';'"),
            ErrorType::UnexpectedSymbol => write!(f, "Unexpected symbol"),
            ErrorType::IncompleteString => write!(f, "Incomplete string"),
            ErrorType::InvalidEscape => write!(f, "Invalid escape sequence"),
            ErrorType::IncompleteCharCode => {
                write!(f, "Incomplete character code. Must have 2 digits")
            }
            ErrorType::IncompleteEscape => write!(f, "Incomplete escape sequence"),
            ErrorType::InvalidCharCode => write!(f, "Invalid character code"),
            ErrorType::IncompleteChar => write!(f, "Incomplete character literal"),
            ErrorType::EmptyChar => write!(f, "Empty character literal. Must have 1 character"),
            ErrorType::TooManyChars => write!(
                f,
                "Too many characters in character literal. Must have 1 character"
            ),
            ErrorType::ExpectedExpr => write!(f, "Expected expression"),
            ErrorType::TrailingComma => write!(f, "Trailing comma"),
            ErrorType::ExpectedVarName => write!(f, "Expected variable name or tuple and struct deconstructing"),
        }
    }
}

pub trait DisplayError: fmt::Display {
    fn display(&self, source: &str, message: &mut impl Write) -> fmt::Result;
}
