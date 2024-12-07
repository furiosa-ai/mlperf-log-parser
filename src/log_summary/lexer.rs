use log::{info, trace, warn};
use logos::{Logos, SpannedIter};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    #[default]
    InvalidToken,
}

pub fn remove_last_char(s: &str) -> String {
    s[..s.len() - 1].to_string()
}

#[derive(Clone, Debug, Logos, PartialEq)]
#[logos(error = LexicalError)]
pub enum Token {
    #[regex(r"={3,}(\r\n|\n)", |lex| remove_last_char(lex.slice()), priority = 4)]
    SectionSeparatorLine(String),

    #[regex(r"[^\n]+(\r\n|\n)", |lex| remove_last_char(lex.slice()), priority = 2)]
    Line(String),

    #[regex(r"(\r\n|\n)+", |lex| lex.slice().to_string(), priority = 1)]
    EndOfSection(String),

    EOF,
}

pub struct Lexer<'input> {
    eof_encountered: bool,
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        let token_stream = Token::lexer(input).spanned();
        Lexer {
            eof_encountered: false,
            token_stream,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eof_encountered {
            return None;
        }
        if let Some((token, span)) = self.token_stream.next() {
            let token = token.ok()?;
            trace!(
                "TOKEN: {:?}, start: {}, end: {}",
                token,
                span.start,
                span.end
            );
            Some(Ok((span.start, token, span.end)))
        } else {
            trace!("TOKEN EOF");
            self.eof_encountered = true;
            // EOF 토큰을 수동으로 추가
            Some(Ok((0, Token::EOF, 0)))
        }
    }
}
