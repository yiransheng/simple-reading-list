use super::query::*;
use crate::error::ServiceError;
use logos::{Lexer, Logos};

#[derive(Debug, Copy, Clone)]
pub struct ParseError {}

impl Into<ServiceError> for ParseError {
    fn into(self) -> ServiceError {
        ServiceError::BadRequest(String::new())
    }
}

#[derive(Logos, Copy, Clone, Debug, PartialEq, Eq)]
enum Token {
    #[end]
    End,

    #[error]
    Error,

    #[token = "not:"]
    Inverse,

    #[token = "'"]
    QuoteSingle,

    #[token = "\""]
    QuoteDouble,

    #[regex = r#"[^\s'"]+"#]
    Word,
}

pub struct QueryParser<'a> {
    lexer: Lexer<Token, &'a str>,
    inverse: bool,
}

impl<'a> QueryParser<'a> {
    pub fn new(query_str: &'a str) -> Self {
        eprintln!("Q: {}", query_str);
        Self {
            lexer: Token::lexer(query_str),
            inverse: false,
        }
    }
    pub fn parse(mut self) -> Result<Query, ParseError> {
        let query_builder = self.do_parse(BoolQueryBuilder::new())?;

        Ok(query_builder.build())
    }
}

#[derive(Clone)]
enum OneOrMore<T> {
    Empty,
    One(T),
    More(Vec<T>),
}

impl<'a> QueryParser<'a> {
    fn do_parse(
        &mut self,
        builder: BoolQueryBuilder,
    ) -> Result<BoolQueryBuilder, ParseError> {
        let mut query_builder = builder;
        loop {
            eprintln!(
                "Next token: {:?}, {}",
                self.lexer.token,
                self.lexer.slice()
            );
            match self.lexer.token {
                Token::Inverse => {
                    self.inverse = true;
                    self.lexer.advance();
                }
                Token::Word => {
                    let term = self.lexer.slice();
                    let adder = if self.inverse {
                        BoolQueryBuilder::must_not_one
                    } else {
                        BoolQueryBuilder::should_one
                    };
                    query_builder = adder(
                        query_builder,
                        FuzzyQueryBuilder::new()
                            .with_field("title".to_owned())
                            .with_term(term.to_owned())
                            .build(),
                    );
                    query_builder = adder(
                        query_builder,
                        FuzzyQueryBuilder::new()
                            .with_field("body".to_owned())
                            .with_term(term.to_owned())
                            .build(),
                    );
                    query_builder = adder(
                        query_builder,
                        ExactQueryBuilder::new()
                            .with_field("tags".to_owned())
                            .with_term(term.to_owned())
                            .build(),
                    );

                    self.inverse = false;
                    self.lexer.advance();
                }
                Token::QuoteSingle | Token::QuoteDouble => {
                    query_builder = self.quote(query_builder)?;
                }
                Token::End => break,
                Token::Error => return Err(ParseError {}),
            }
        }

        Ok(query_builder)
    }
    fn inside_quote(&mut self) -> Result<OneOrMore<&'a str>, ParseError> {
        let mut terms: Vec<_> = vec![];
        loop {
            match self.lexer.token {
                Token::Inverse => {
                    terms.push("not:");
                    self.lexer.advance();
                }
                Token::Word => {
                    terms.push(self.lexer.slice());
                    self.lexer.advance();
                }
                Token::End | Token::QuoteSingle | Token::QuoteDouble => break,
                Token::Error => return Err(ParseError {}),
            }
        }

        if terms.is_empty() {
            Ok(OneOrMore::Empty)
        } else if terms.len() == 1 {
            Ok(OneOrMore::One(terms.pop().unwrap()))
        } else {
            Ok(OneOrMore::More(terms))
        }
    }
    fn quote(
        &mut self,
        builder: BoolQueryBuilder,
    ) -> Result<BoolQueryBuilder, ParseError> {
        let mut query_builder = builder;
        let quote = self.lexer.token;
        self.lexer.advance();

        let terms = self.inside_quote()?;
        self.assert_token(quote)
            .or_else(|_| self.assert_token(Token::End))?;

        let fields = ["tags", "title", "body"];
        match terms {
            OneOrMore::Empty => {}
            OneOrMore::One(term) => {
                let queries = (&fields[..]).into_iter().map(|field| {
                    ExactQueryBuilder::new()
                        .with_field(field.to_string())
                        .with_term(term.to_owned())
                        .build()
                });
                if self.inverse {
                    query_builder = query_builder.must_not(queries);
                    self.inverse = false;
                } else {
                    query_builder = query_builder.should(queries);
                }
            }
            OneOrMore::More(ref terms) => {
                let queries = (&fields[..]).into_iter().map(|field| {
                    PhraseQueryBuilder::new()
                        .with_field(field.to_string())
                        .with_terms(terms.iter().map(|term| term.to_string()))
                        .build()
                });
                if self.inverse {
                    query_builder = query_builder.must_not(queries);
                    self.inverse = false;
                } else {
                    query_builder = query_builder.should(queries);
                }
            }
        }

        Ok(query_builder)
    }

    fn assert_token(&mut self, token: Token) -> Result<(), ParseError> {
        if token == self.lexer.token {
            self.lexer.advance();
            Ok(())
        } else {
            Err(ParseError {})
        }
    }
}
