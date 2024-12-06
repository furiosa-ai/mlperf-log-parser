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
}

pub struct Lexer<'input> {
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| {
            println!("{:?}", token);
            Ok((span.start, token?, span.end))
        })
    }
}
