#[cfg(test)]
mod tests;

use crate::location::Loc;
use crate::utils::*;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Enum representing lexeme types
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TokenKind {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Question,
    Colon,
    Semicolon,

    // One or two character tokens
    Minus,
    MinusEqual,
    MinusMinus,
    Plus,
    PlusEqual,
    PlusPlus,
    Slash,
    SlashEqual,
    Star,
    StarEqual,
    Percent,
    PercentEqual,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    Str,
    Integer,
    Float,

    // Keywords
    And,
    Break,
    Class,
    Continue,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

#[derive(PartialEq, Debug)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Str(String),
}

#[derive(PartialEq, Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub literal: Option<Literal>,
    pub loc: Loc,
}

pub struct Scanner<'a> {
    input: &'a str,
    chars: std::str::Chars<'a>,
    start: usize,
    current: usize,
    start_loc: Loc,
    current_loc: Loc,
}

#[derive(Debug, PartialEq, Fail)]
pub enum ScanningError {
    UnrecognizedCharacter(char, Loc),
    UnterminatedString(Loc),
    InvalidNumber(String, Loc),
    UnterminatedBlockComment(Loc),
    Multiple(Vec<ScanningError>),
}

type TokenRes<'a> = Result<Token<'a>, ScanningError>;
type ScanningRes<'a> = Result<Vec<Token<'a>>, ScanningError>;

impl Literal {
    fn string(string: &str) -> Self {
        Literal::Str(String::from(string))
    }
}

impl<'a> Token<'a> {
    fn eof(loc: Loc) -> Self {
        Token {
            kind: TokenKind::EOF,
            lexeme: "EOF",
            literal: None,
            loc,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let data = match &self.literal {
            Some(Literal::Str(string)) => format!("{}, \"{}\"", self.lexeme, string),
            Some(Literal::Integer(integer)) => format!("{}, {}", self.lexeme, integer),
            Some(Literal::Float(float)) => format!("{}, {}", self.lexeme, float),
            None => String::from(self.lexeme),
        };

        write!(f, "<{:?}, {}, {}>", self.kind, data, self.loc)
    }
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars(),
            start: 0,
            current: 0,
            start_loc: Loc::new(0, 0),
            current_loc: Loc::new(0, 0),
        }
    }

    pub fn scan_tokens(&mut self) -> ScanningRes<'a> {
        let mut tokens: Vec<Token<'a>> = vec![];
        let mut errors: Vec<ScanningError> = vec![];
        let mut token_res = self.scan_token();

        loop {
            match token_res {
                Ok(Some(token)) => {
                    tokens.push(token);
                }
                Ok(None) => {
                    if self.is_at_end() {
                        break;
                    }
                }
                Err(error) => {
                    errors.push(error);
                }
            }
            token_res = self.scan_token();
        }

        match errors.len() {
            0 => {
                tokens.push(Token::eof(self.current_loc));
                Ok(tokens)
            }
            1 => Err(errors.pop().unwrap()),
            _ => Err(ScanningError::Multiple(errors)),
        }
    }

    fn scan_token(&mut self) -> Result<Option<Token<'a>>, ScanningError> {
        use TokenKind::*;
        self.skip_whitespace();
        self.start = self.current;
        self.start_loc = self.current_loc;

        if self.is_at_end() {
            return Ok(None);
        }

        let token = match self.advance().unwrap() {
            '(' => self.create_token(LeftParen),
            ')' => self.create_token(RightParen),
            '{' => self.create_token(LeftBrace),
            '}' => self.create_token(RightBrace),
            '[' => self.create_token(LeftBracket),
            ']' => self.create_token(RightBracket),
            ',' => self.create_token(Comma),
            '.' => self.create_token(Dot),
            '?' => self.create_token(Question),
            ':' => self.create_token(Colon),
            ';' => self.create_token(Semicolon),
            '-' => {
                let kind = if self.matches('=') {
                    MinusEqual
                } else if self.matches('-') {
                    MinusMinus
                } else {
                    Minus
                };
                self.create_token(kind)
            }
            '+' => {
                let kind = if self.matches('=') {
                    PlusEqual
                } else if self.matches('+') {
                    PlusPlus
                } else {
                    Plus
                };
                self.create_token(kind)
            }
            '*' => {
                let kind = if self.matches('=') { StarEqual } else { Star };
                self.create_token(kind)
            }
            '%' => {
                let kind = if self.matches('=') {
                    PercentEqual
                } else {
                    Percent
                };
                self.create_token(kind)
            }
            '!' => {
                let kind = if self.matches('=') { BangEqual } else { Bang };
                self.create_token(kind)
            }
            '=' => {
                let kind = if self.matches('=') { EqualEqual } else { Equal };
                self.create_token(kind)
            }
            '<' => {
                let kind = if self.matches('=') { LessEqual } else { Less };
                self.create_token(kind)
            }
            '>' => {
                let kind = if self.matches('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.create_token(kind)
            }
            '/' => {
                if self.matches('/') {
                    self.advance_while(|ch| ch != '\n');
                    return Ok(None);
                } else if self.matches('*') {
                    self.skip_block_comment()?;
                    return Ok(None);
                } else {
                    let kind = if self.matches('=') { SlashEqual } else { Slash };
                    self.create_token(kind)
                }
            }
            '"' => self.recognize_string()?,
            '0'..='9' => self.recognize_number()?,
            ch if is_alpha(ch) => self.recognize_identifier()?,
            character => return Err(unrecognized_character(&self, character)),
        };

        Ok(Some(token))
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.peek() {
            if !character.is_ascii_whitespace() {
                break;
            }

            self.advance();

            if character == '\n' {
                self.current_loc.new_line();
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.next() {
            self.current += ch.len_utf8();
            self.current_loc.advance();
            Some(ch)
        } else {
            None
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.clone().nth(1)
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance_while<F>(&mut self, mut predicate: F)
    where
        F: FnMut(char) -> bool,
    {
        while let Some(ch) = self.peek() {
            if predicate(ch) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn get_lexeme(&self) -> &'a str {
        &self.input[self.start..self.current]
    }

    fn recognize_string(&mut self) -> TokenRes<'a> {
        while let Some(character) = self.peek() {
            match character {
                '"' => break,
                '\\' => {
                    self.advance();
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.current_loc.new_line();
                }
                _ => {
                    self.advance();
                }
            }
        }

        if self.is_at_end() {
            return Err(unterminated_string(&self));
        }

        self.advance();

        let value = unescape_string(&self.input[self.start + 1..self.current - 1]);
        let literal = Literal::string(&value);
        Ok(self.create_literal_token(TokenKind::Str, literal))
    }

    fn recognize_number(&mut self) -> TokenRes<'a> {
        self.advance_while(is_digit);

        let mut is_float = match (self.peek(), self.peek_next()) {
            (Some('.'), Some(ch)) if is_digit(ch) => {
                self.advance();
                self.advance_while(is_digit);
                true
            }
            _ => false,
        };

        if self.matches('e') || self.matches('E') {
            is_float = true;
            if !self.matches('+') {
                self.matches('-');
            };
            self.advance_while(is_digit);
        }

        let kind = if is_float {
            TokenKind::Float
        } else {
            TokenKind::Integer
        };

        let lexeme = self.get_lexeme();

        let literal = if is_float {
            Literal::Float(lexeme.parse().map_err(|_| invalid_number(&self, lexeme))?)
        } else {
            Literal::Integer(lexeme.parse().map_err(|_| invalid_number(&self, lexeme))?)
        };

        Ok(self.create_literal_token(kind, literal))
    }

    fn recognize_identifier(&mut self) -> TokenRes<'a> {
        self.advance_while(is_alphanumeric);

        let text = self.get_lexeme();
        let kind = keyword_to_kind(text).unwrap_or(TokenKind::Identifier);

        Ok(self.create_token(kind))
    }

    fn skip_block_comment(&mut self) -> Result<(), ScanningError> {
        let mut depth = 1usize;
        while let Some(ch) = self.advance() {
            match (ch, self.peek()) {
                ('/', Some('*')) => {
                    self.advance();
                    depth += 1;
                }
                ('*', Some('/')) => {
                    self.advance();
                    depth -= 1;

                    if depth == 0 {
                        break;
                    }
                }
                ('\n', _) => self.current_loc.new_line(),
                _ => (),
            }
        }

        if depth == 0 {
            Ok(())
        } else {
            Err(unterminated_block_comment(&self))
        }
    }

    fn create_token(&self, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            lexeme: self.get_lexeme(),
            literal: None,
            loc: self.start_loc,
        }
    }

    fn create_literal_token(&self, kind: TokenKind, literal: Literal) -> Token<'a> {
        Token {
            kind,
            lexeme: &self.get_lexeme(),
            literal: Some(literal),
            loc: self.start_loc,
        }
    }
}

fn unrecognized_character(scanner: &Scanner, character: char) -> ScanningError {
    ScanningError::UnrecognizedCharacter(character, scanner.start_loc)
}

fn unterminated_string(scanner: &Scanner) -> ScanningError {
    ScanningError::UnterminatedString(scanner.current_loc)
}

fn invalid_number(scanner: &Scanner, number: &str) -> ScanningError {
    ScanningError::InvalidNumber(String::from(number), scanner.start_loc)
}

fn unterminated_block_comment(scanner: &Scanner) -> ScanningError {
    ScanningError::UnterminatedBlockComment(scanner.current_loc)
}

fn keyword_to_kind(keyword: &str) -> Option<TokenKind> {
    use TokenKind::*;
    Some(match keyword {
        "and" => And,
        "break" => Break,
        "class" => Class,
        "continue" => Continue,
        "else" => Else,
        "false" => False,
        "for" => For,
        "fun" => Fun,
        "if" => If,
        "nil" => Nil,
        "or" => Or,
        "print" => Print,
        "return" => Return,
        "super" => Super,
        "this" => This,
        "true" => True,
        "var" => Var,
        "while" => While,
        _ => return None,
    })
}
