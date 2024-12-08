use log::debug;
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

#[derive(PartialEq, Debug, Clone)]
pub struct SourceLocation {
    pub source: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Logos, PartialEq)]
#[logos(error = LexicalError)]
#[logos(extras = usize)] // 줄 번호만 추적
pub enum Token {
    #[regex(r"={3,}(\r\n|\n)", |lex| {
        lex.extras += 1;
        let location = SourceLocation {
            source: remove_last_char(lex.slice()),
            line: lex.extras,
            column: lex.span().start,
        };
        location
    }, priority = 4)]
    SectionSeparatorLine(SourceLocation),

    #[regex(r"[^\n]+(\r\n|\n)", |lex| {
        lex.extras += 1;
        let location = SourceLocation {
            source: remove_last_char(lex.slice()),
            line: lex.extras,
            column: lex.span().start,
        };
        location
    }, priority = 2)]
    Line(SourceLocation),

    #[regex(r"(\r\n|\n)+", |lex| {
        let newlines = lex.slice().matches('\n').count();
        lex.extras += newlines;
        let location = SourceLocation {
            source: remove_last_char(lex.slice()),
            line: lex.extras,
            column: lex.span().start,
        };
        location
    }, priority = 1)]
    EndOfSection(SourceLocation),

    EOF,
}

pub struct Lexer<'input> {
    eof_encountered: bool,
    line_number: usize,
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        // extras를 0으로 초기화하여 lexer 생성
        // extras: 현재 줄 번호
        let mut lexer = Token::lexer(input);
        lexer.extras = Default::default();
        let token_stream = lexer.spanned();

        Lexer {
            eof_encountered: false,
            line_number: 0,
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
            self.line_number = self.token_stream.extras;
            let token = token.ok()?;
            debug!(
                "TOKEN: {:?}, start: {}, end: {}, line: {}",
                token,
                span.start,
                span.end,
                self.line_number, // 현재 줄 번호 출력
            );
            Some(Ok((span.start, token, span.end)))
        } else {
            debug!("TOKEN EOF");
            self.eof_encountered = true;
            Some(Ok((0, Token::EOF, 0)))
        }
    }
}
