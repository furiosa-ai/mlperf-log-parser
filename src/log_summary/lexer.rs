use logos::{Logos, SpannedIter};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    #[default]
    InvalidToken,
}

#[derive(Clone, Debug, Logos, PartialEq)]
#[logos(error = LexicalError)]
pub enum Token {
    #[regex(r"(\r\n|\n)")]
    SingleNewLine,

    #[regex(r"={3,}", |lex| lex.slice().to_string(), priority = 4)]
    SectionSeparatorLine(String),

    //#[regex(r"[^=\n]{1,3}[^\n]*", |lex| lex.slice().to_string(), priority = 2)]
    //LineWithoutSeparator(String),
    #[regex(r"[^\n]+", |lex| lex.slice().to_string(), priority = 2)]
    Line(String),

    EOF,
}

pub struct Lexer<'input> {
    eof_encountered: bool,
    line_counter: usize,
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        let token_stream = Token::lexer(input).spanned();
        Lexer {
            eof_encountered: false,
            line_counter: 0,
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
            println!(
                "TOKEN: {:?}, start: {}, end: {}",
                token, span.start, span.end
            );
            if token == Token::SingleNewLine {
                self.line_counter += 1;
            }
            Some(Ok((span.start, token, span.end)))
        } else {
            println!("TOKEN EOF");
            self.eof_encountered = true;
            // EOF 토큰을 수동으로 추가
            Some(Ok((0, Token::EOF, 0)))
        }
    }
}
