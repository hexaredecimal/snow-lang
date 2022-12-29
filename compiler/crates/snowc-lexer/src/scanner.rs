use super::{LexerDebug, Span, Token};
use std::{iter::Peekable, str::Chars};
type Stream<'a> = Peekable<Chars<'a>>;

#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    stream: Stream<'a>,
    char_stream: Vec<char>,
    span: Span,
    current: Option<char>,
    previous: Option<char>,
    debug_lexer: LexerDebug,
}

impl<'a> Scanner<'a> {
    pub fn new(src: &'a str, debug_lexer: LexerDebug) -> Self {
        Self {
            stream: src.clone().chars().peekable(),
            char_stream: src.chars().collect(),
            span: Span::default(),
            current: None,
            previous: None,
            debug_lexer,
        }
    }

    fn next_char2(&mut self) -> char {
        let idx = self.span.end;
        self.span.shift_right();
        self.char_stream[idx]
    }

    fn previous() -> char {
        todo!()
    }
    fn advance(&mut self) {
        match self.current {
            Some(c) => {
                self.span.end += c.to_string().as_bytes().len();
            }
            None => {
                let Some(c) = self.previous else {
                    self.span.end += 1;
                    return;
                };
                self.span.end += c.to_string().as_bytes().len();
            }
        }
    }

    fn matched(&mut self, c: char) -> bool {
        self.peek_char() == c
    }

    fn next_char(&mut self) -> Option<char> {
        self.previous = self.current;
        self.current = self.stream.next();
        self.advance();
        self.current
    }

    fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        match (self.current, self.stream.next_if(func)) {
            (Some(current), next @ Some(_)) => {
                self.previous = Some(current);
                self.current = next;
                self.advance();
            }
            (_, next @ Some(_)) => self.current = next,
            _ => return None,
        }
        self.current
    }

    fn peek_char(&mut self) -> char {
        *self.stream.peek().unwrap_or(&'\0')
    }

    fn reset_span(&mut self) {
        self.span.start = self.span.end;
    }

    fn span(&mut self) -> Span {
        let span = self.span.clone();
        self.reset_span();
        span
    }

    fn number(&mut self) -> Token {
        let mut number = self.current.unwrap().to_string();
        while let Some(ch) =
            self.next_if(|c| c.is_ascii_digit() || c == &'_' || c == &'.')
        {
            number.push(ch);
        }
        let span = self.span();
        if number.contains('.') {
            Token::Float(number, span)
        } else {
            Token::Int(number, span)
        }
    }

    fn id(&mut self) -> Token {
        let mut ident = self.current.unwrap().to_string();
        while let Some(ch) = self.next_if(|c| c.is_ascii_alphanumeric() || c == &'_') {
            ident.push(ch);
        }
        let span = self.span();
        Token::lookup(&ident.clone()).map_or(Token::Id(ident.into(), span), |i| {
            Token::KeyWord(i.into(), span)
        })
    }

    fn line_comment(&mut self) -> Option<Token> {
        while let Some(_) = self.next_if(|c| c != &'\n') {}
        self.next()
    }

    fn string(&mut self) -> Option<Token> {
        let mut string = String::new();
        while let Some(ch) = self.next_if(|c| c != &'"') {
            string.push(ch);
        }
        self.next_char();
        Some(Token::String(string, self.span()))
    }

    fn chr(&mut self) -> Option<Token> {
        let mut c = String::new();
        while let Some(ch) = self.next_if(|c| c != &'\'') {
            c.push(ch);
        }
        self.next_char();
        Some(Token::Char(c, self.span()))
    }

    fn op_token(&mut self, op: &str) -> Token {
        for _ in 0..op.chars().count().saturating_sub(1) {
            self.next_char();
        }
        Token::Op(op.into(), self.span())
    }

    fn err(&mut self, c: char) -> Option<Token> {
        Some(Token::Error(c.to_string(), self.span()))
    }

    fn debug_token(&mut self, token: Option<Token>) -> Token {
        let token = token.unwrap_or(Token::Eof(self.span()));
        if let LexerDebug::On = self.debug_lexer {
            let kind = token.value();
            let span = token.span();
            eprintln!("{kind:?} @ {span:?}");
        }
        token
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        let Some(ch) = self.next_char() else {
            return Some(self.debug_token(None));
        };
        let token = match ch {
            num if num.is_ascii_digit() => Some(self.number()),
            ident if ident.is_ascii_alphabetic() => Some(self.id()),
            '-' if self.matched('-') => self.line_comment(),
            '-' if self.matched('>') => Some(self.op_token("->")),
            '=' if self.matched('>') => Some(self.op_token("=>")),
            '<' if self.matched('-') => Some(self.op_token("<-")),
            '<' if self.matched('=') => Some(self.op_token("<=")),
            '>' if self.matched('=') => Some(self.op_token(">=")),
            '=' if self.matched('=') => Some(self.op_token("==")),
            '!' if self.matched('=') => Some(self.op_token("!=")),
            ':' if self.matched(':') => Some(self.op_token("::")),
            '|' if self.matched('>') => Some(self.op_token("|>")),
            '"' => self.string(),
            '\'' => self.chr(),
            '\\' => Some(self.op_token("\\")),
            '|' => Some(self.op_token("|")),
            '!' => Some(self.op_token("!")),
            '<' => Some(self.op_token("<")),
            '>' => Some(self.op_token(">")),
            '+' => Some(self.op_token("+")),
            '-' => Some(self.op_token("-")),
            '=' => Some(self.op_token("=")),
            '*' => Some(self.op_token("*")),
            '/' => Some(self.op_token("/")),
            ':' => Some(self.op_token(":")),
            ';' => Some(self.op_token(";")),
            ',' => Some(self.op_token(",")),
            '(' => Some(self.op_token("(")),
            ')' => Some(self.op_token(")")),
            '[' => Some(self.op_token("[")),
            ']' => Some(self.op_token("]")),
            '{' => Some(self.op_token("{")),
            '}' => Some(self.op_token("}")),
            'λ' => Some(self.op_token("λ")),
            ' ' | '\n' => {
                self.reset_span();
                self.next()
            }
            c => self.err(c),
        };
        Some(self.debug_token(token))
    }
}