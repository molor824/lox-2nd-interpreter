use std::fmt;
use std::fmt::Write;
use std::ops::Range;

use crate::source::SourceIter;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    pub range: Range<usize>,
    pub error: ErrorType,
}
impl Error {
    pub fn new(range: Range<usize>, error: ErrorType) -> Self {
        Self { range, error }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

fn highlight_line(
    source: &str,
    line_range: Range<usize>,
    error_range: Range<usize>,
    line_number: usize,
    message: &mut impl Write,
) -> fmt::Result {
    // checking for collision, if there isn't then there's no need to do anything
    if usize::max(error_range.start, line_range.start)
        >= usize::min(error_range.end, line_range.end)
    {
        return Ok(());
    }

    let line_number_string = format!(" {:3} | ", line_number + 1);
    writeln!(
        message,
        "{}{}",
        line_number_string,
        &source[line_range.clone()]
    )?;

    // print spaces to compensate for the line number string
    for _ in 0..line_number_string.len() {
        write!(message, " ")?;
    }

    // print spaces until it reaches the highlight
    let highlight_start = usize::max(line_range.start, error_range.start);
    for _ in line_range.start..highlight_start {
        write!(message, " ")?;
    }

    // print '^' until it reaches error range end or line range end
    for _ in highlight_start..(usize::min(line_range.end, error_range.end)) {
        write!(message, "^")?;
    }

    // finally newline
    writeln!(message)
}
impl DisplayError for Error {
    fn display(&self, source: &str, message: &mut impl Write) -> fmt::Result {
        let mut iter = SourceIter::from(source);
        let mut column = 0;
        let mut line = 0;
        let mut line_start = 0;
        let mut line_end = 0; // index past the last iterated character (for the last line ending without newline)

        for (i, ch) in iter.by_ref() {
            line_end = i + ch.len_utf8();

            if ch == '\n' {
                // reached end of line, highlight the line
                highlight_line(source, line_start..i, self.range.clone(), line, message)?;

                // update line, column and line_start
                line += 1;
                column = 0;
                line_start = i + 1; // '\n' will always have length 1
            } else {
                column += 1;
            }
            // check if it just reached the range
            if i == self.range.start {
                // print the error location
                write!(
                    message,
                    "Error at line: {}, column: {}\n{}\n",
                    line + 1,
                    column,
                    self.error
                )?;
            }
        }

        // this is the last line, highlight it if range exists
        highlight_line(
            source,
            line_start..line_end,
            self.range.clone(),
            line,
            message,
        )
    }
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
    ExpectedFuncName,
    ExpectedSemicolon,
    ExpectedColon,
    ExpectedAssign,
    ExpectedBlock,
    ExpectedFuncBlock,
    ExpectedEOF,

    IncompleteString,
    IncompleteCharCode,
    IncompleteEscape,
    IncompleteChar,

    InvalidEscape,
    InvalidCharCode,

    EmptyChar,
    TooManyChars,

    UnexpectedSymbol,

    ExtraDots,

    UnderscoreVariable,
}
impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::IntOverflow => write!(f, "Integer overflow"),
            ErrorType::UnexpectedSymbol => write!(f, "Unexpected symbol"),

            ErrorType::ExpectedLParen => write!(f, "Expected a '('"),
            ErrorType::ExpectedRParen => write!(f, "Expected a ')'"),
            ErrorType::ExpectedLCurly => write!(f, "Expected a '{{'"),
            ErrorType::ExpectedRCurly => write!(f, "Expected a '}}'"),
            ErrorType::ExpectedLSquare => write!(f, "Expected a '['"),
            ErrorType::ExpectedRSquare => write!(f, "Expected a ']'"),
            ErrorType::ExpectedSemicolon => write!(f, "Expected a ';'"),
            ErrorType::ExpectedColon => write!(f, "Expected a ':'"),
            ErrorType::ExpectedAssign => write!(f, "Expected a '='"),
            ErrorType::ExpectedNumber => write!(f, "Expected a number"),
            ErrorType::ExpectedInteger => write!(f, "Expected an integer"),
            ErrorType::ExpectedIdent => write!(f, "Expected an identifier"),
            ErrorType::ExpectedFuncName => write!(f, "Expected a function name"),
            ErrorType::ExpectedExpr => write!(f, "Expected expression"),
            ErrorType::ExpectedVarName => write!(f, "Expected variable name"),
            ErrorType::ExpectedEOF => write!(f, "Expected end of file"),
            ErrorType::ExpectedBlock => write!(f, "Expected a block of statements"),
            ErrorType::ExpectedFuncBlock => {
                write!(f, "Expected a block or expression ('-> [expr]')")
            }

            ErrorType::IncompleteString => write!(f, "Incomplete string"),
            ErrorType::IncompleteEscape => write!(f, "Incomplete escape sequence"),
            ErrorType::IncompleteChar => write!(f, "Incomplete character literal"),
            ErrorType::IncompleteCharCode => {
                write!(f, "Incomplete character code. Must have 2 digits")
            }
            ErrorType::InvalidEscape => write!(f, "Invalid escape sequence"),
            ErrorType::InvalidCharCode => write!(f, "Invalid character code"),
            ErrorType::EmptyChar => write!(f, "Empty character literal. Must have 1 character"),
            ErrorType::TooManyChars => write!(
                f,
                "Too many characters in character literal. Must have 1 character"
            ),
            ErrorType::ExtraDots => {
                write!(f, "Cannot have multiple '...' symbol in array unpacking")
            }
            ErrorType::UnderscoreVariable => {
                write!(f, "Cannot read from '_'. You can only assign to it")
            }
        }
    }
}

pub trait DisplayError: fmt::Display {
    fn display(&self, source: &str, message: &mut impl Write) -> fmt::Result;
}
