//! Minimal s-expression reader. Atoms are any run of non-space, non-paren
//! characters; `;` starts a line comment.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SExp {
    Atom(String),
    List(Vec<SExp>),
}

impl SExp {
    pub fn as_atom(&self) -> Option<&str> {
        match self {
            SExp::Atom(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<&[SExp]> {
        match self {
            SExp::List(v) => Some(v),
            _ => None,
        }
    }
}

impl fmt::Display for SExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SExp::Atom(s) => write!(f, "{s}"),
            SExp::List(v) => {
                write!(f, "(")?;
                for (i, x) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{x}")?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.msg)
    }
}

impl std::error::Error for ParseError {}

/// Parse all top-level forms in `src`.
pub fn parse_all(src: &str) -> Result<Vec<SExp>, ParseError> {
    let mut p = Parser { chars: src.chars().collect(), pos: 0, line: 1 };
    let mut out = Vec::new();
    loop {
        p.skip_ws();
        if p.pos >= p.chars.len() {
            return Ok(out);
        }
        out.push(p.parse_one()?);
    }
}

pub fn parse_one(src: &str) -> Result<SExp, ParseError> {
    let mut v = parse_all(src)?;
    if v.len() != 1 {
        return Err(ParseError { line: 1, msg: format!("expected one form, found {}", v.len()) });
    }
    Ok(v.pop().unwrap())
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl Parser {
    fn skip_ws(&mut self) {
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c == ';' {
                while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                    self.pos += 1;
                }
            } else if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                }
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_one(&mut self) -> Result<SExp, ParseError> {
        self.skip_ws();
        let start_line = self.line;
        match self.chars.get(self.pos) {
            None => Err(ParseError { line: self.line, msg: "unexpected end of input".into() }),
            Some(')') => Err(ParseError { line: self.line, msg: "unexpected ')'".into() }),
            Some('(') => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    match self.chars.get(self.pos) {
                        None => {
                            return Err(ParseError { line: start_line, msg: "unclosed '('".into() });
                        }
                        Some(')') => {
                            self.pos += 1;
                            return Ok(SExp::List(items));
                        }
                        _ => items.push(self.parse_one()?),
                    }
                }
            }
            Some(_) => {
                let s = self.pos;
                while self.pos < self.chars.len() {
                    let c = self.chars[self.pos];
                    if c.is_whitespace() || c == '(' || c == ')' || c == ';' {
                        break;
                    }
                    self.pos += 1;
                }
                Ok(SExp::Atom(self.chars[s..self.pos].iter().collect()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_and_comments() {
        let v = parse_all("; hi\n(a (b c) d) ; tail\n(e)").unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].to_string(), "(a (b c) d)");
        assert_eq!(v[1].to_string(), "(e)");
    }

    #[test]
    fn reports_errors() {
        assert!(parse_all("(a (b)").is_err());
        assert!(parse_all(")").is_err());
    }
}
