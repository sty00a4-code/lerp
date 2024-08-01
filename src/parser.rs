use std::{
    fmt::{Debug, Display},
    iter::Peekable,
    num::{ParseFloatError, ParseIntError},
    str::Chars,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Expr(Vec<Located<Self>>),
    Word(String),
    Int(i32),
    Float(f32),
    String(String),
}
impl Display for Located<SExpr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sexpr = &self.value;
        match sexpr {
            SExpr::Expr(sexprs) => write!(
                f,
                "({})",
                sexprs
                    .iter()
                    .map(|sexpr| sexpr.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            SExpr::Word(word) => write!(f, "{word}"),
            SExpr::Int(int) => write!(f, "{int:?}"),
            SExpr::Float(float) => write!(f, "{float:?}"),
            SExpr::String(string) => write!(f, "{string:?}"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub ln: usize,
    pub col: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Located<T>
where
    T: Debug + Clone,
{
    pub value: T,
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub pos: Position,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    Unexpected(char),
    Unclosed(char),
    UnclosedString,
    ParseFloatError(ParseFloatError),
    ParseIntError(ParseIntError),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.pos.ln + 1, self.pos.col + 1, self.kind)
    }
}
impl Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorKind::Unexpected(c) => write!(f, "unexpected {c:?}"),
            ParseErrorKind::Unclosed(c) => write!(f, "unclosed {c:?}"),
            ParseErrorKind::UnclosedString => write!(f, "unclosed string"),
            ParseErrorKind::ParseFloatError(err) => write!(f, "error while parsing float: {err}"),
            ParseErrorKind::ParseIntError(err) => write!(f, "error while parsing int: {err}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer<'s> {
    pub text: Peekable<Chars<'s>>,
    pub ln: usize,
    pub col: usize,
}
impl<'s> From<&'s str> for Lexer<'s> {
    fn from(value: &'s str) -> Self {
        Self {
            text: value.chars().peekable(),
            ln: 0,
            col: 0,
        }
    }
}
impl<'s> Lexer<'s> {
    pub const SYMBOLS: &'static [char] = &['(', ')', '"'];
    pub fn next(&mut self) -> Option<char> {
        let c = self.text.next()?;
        if c == '\n' {
            self.ln += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Some(c)
    }
    pub fn peek(&mut self) -> Option<&char> {
        self.text.peek()
    }
    pub fn pos(&mut self) -> Position {
        Position {
            ln: self.ln,
            col: self.col,
        }
    }
    pub fn parse_next(&mut self) -> Result<Option<Located<SExpr>>, ParseError> {
        while let Some(c) = self.peek() {
            if !c.is_ascii_whitespace() {
                break;
            }
            self.next();
        }
        let pos = self.pos();
        let Some(c) = self.next() else {
            return Ok(None);
        };
        match c {
            '(' => {
                let mut exprs = vec![];
                while let Some(c) = self.peek() {
                    if c == &')' {
                        break;
                    }
                    let Some(sexpr) = self.parse_next()? else {
                        return Err(ParseError {
                            kind: ParseErrorKind::Unclosed('('),
                            pos,
                        });
                    };
                    exprs.push(sexpr);
                    while let Some(c) = self.peek() {
                        if !c.is_ascii_whitespace() {
                            break;
                        }
                        self.next();
                    }
                }
                if self.next() != Some(')') {
                    return Err(ParseError {
                        kind: ParseErrorKind::Unclosed('('),
                        pos,
                    });
                }
                Ok(Some(Located {
                    value: SExpr::Expr(exprs),
                    pos,
                }))
            }
            '"' => {
                let mut string = String::new();
                while let Some(c) = self.peek() {
                    if c == &'"' {
                        break;
                    }
                    let c = self.next().unwrap();
                    string.push(c);
                }
                let Some(c) = self.next() else {
                    return Err(ParseError {
                        kind: ParseErrorKind::UnclosedString,
                        pos: self.pos(),
                    });
                };
                if c != '"' {
                    return Err(ParseError {
                        kind: ParseErrorKind::UnclosedString,
                        pos: self.pos(),
                    });
                }
                Ok(Some(Located {
                    value: SExpr::String(string),
                    pos,
                }))
            }
            c if c.is_ascii_digit() => {
                let mut number = String::from(c);
                while let Some(c) = self.peek() {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    let c = self.next().unwrap();
                    number.push(c);
                }
                if self.peek() == Some(&'.') {
                    let c = self.next().unwrap();
                    number.push(c);
                    while let Some(c) = self.peek() {
                        if !c.is_ascii_digit() {
                            break;
                        }
                        let c = self.next().unwrap();
                        number.push(c);
                    }
                    Ok(Some(Located {
                        value: SExpr::Float(number.parse().map_err(|err| ParseError {
                            kind: ParseErrorKind::ParseFloatError(err),
                            pos,
                        })?),
                        pos,
                    }))
                } else {
                    Ok(Some(Located {
                        value: SExpr::Int(number.parse().map_err(|err| ParseError {
                            kind: ParseErrorKind::ParseIntError(err),
                            pos,
                        })?),
                        pos,
                    }))
                }
            }
            c if !Self::SYMBOLS.contains(&c) => {
                let mut word = String::from(c);
                while let Some(c) = self.peek() {
                    if c.is_ascii_whitespace() || Self::SYMBOLS.contains(c) {
                        break;
                    }
                    let c = self.next().unwrap();
                    word.push(c);
                }
                Ok(Some(Located {
                    value: SExpr::Word(word),
                    pos,
                }))
            }
            c => Err(ParseError {
                kind: ParseErrorKind::Unexpected(c),
                pos,
            }),
        }
    }
    pub fn parse(&mut self) -> Result<Vec<Located<SExpr>>, ParseError> {
        let mut exprs = vec![];
        while let Some(expr) = self.parse_next()? {
            exprs.push(expr);
        }
        Ok(exprs)
    }
}

pub fn parse(code: &str) -> Result<Vec<Located<SExpr>>, ParseError> {
    Lexer::from(code).parse()
}
