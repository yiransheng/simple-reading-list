use logos::{Lexer, Logos};

use super::query::*;
use crate::macros::*;

#[derive(Logos, Copy, Clone, Debug, PartialEq, Eq)]
enum Token {
    #[end]
    End,

    #[error]
    Error,

    #[token = "not:"]
    Inverse,

    #[token = "tag:"]
    Tag,

    #[token = "'"]
    QuoteSingle,

    #[token = "\""]
    QuoteDouble,

    #[regex = r#"[^\s'":]+"#]
    Word,
}

pub struct QueryParser<'a> {
    lexer: Lexer<Token, &'a str>,
}

impl<'a> QueryParser<'a> {
    pub fn new(query_str: &'a str) -> Self {
        Self {
            lexer: Token::lexer(query_str),
        }
    }
    pub fn parse(mut self) -> Query {
        self.parse_items(BoolQueryBuilder::new()).build()
    }
}

#[derive(Clone)]
enum OneOrMore<T> {
    Empty,
    One(T),
    More(Vec<T>),
}

// queries       = query +
// query         = match | phrase | Inverse match | Inverse phrase
// match         = Word | exact | (Tag ":" Word) | (Tag ":" exact)
// exact         = <quote> Word <quote>
// phrase        = <quote> Word {2,*} <quote>
impl<'a> QueryParser<'a> {
    fn parse_items(
        &mut self,
        mut builder: BoolQueryBuilder,
    ) -> BoolQueryBuilder {
        loop_panic_when_stuck!({
            let result = match self.lexer.token {
                Token::End | Token::Error => return builder,
                Token::Inverse => self.inverse_item(builder),
                _ => self.regular_item(builder),
            };
            match result {
                Ok(b) => {
                    builder = b;
                }
                Err(b) => return b,
            }
        })
    }

    fn regular_item(
        &mut self,
        builder: BoolQueryBuilder,
    ) -> Result<BoolQueryBuilder, BoolQueryBuilder> {
        match self.lexer.token {
            Token::Tag => self.tag_item(false, builder),
            Token::Word | Token::QuoteSingle | Token::QuoteDouble => {
                self.match_item(false, builder)
            }
            Token::End | Token::Error => Err(builder),
            _ => {
                self.lexer.advance();
                Ok(builder)
            }
        }
    }
    fn inverse_item(
        &mut self,
        builder: BoolQueryBuilder,
    ) -> Result<BoolQueryBuilder, BoolQueryBuilder> {
        if let Err(_) = self.assert_token(Token::Inverse) {
            return Err(builder);
        }
        match self.lexer.token {
            Token::Tag => self.tag_item(true, builder),
            Token::Word | Token::QuoteSingle | Token::QuoteDouble => {
                self.match_item(true, builder)
            }
            Token::End => Ok(builder),
            Token::Error => Err(builder),
            _ => {
                self.lexer.advance();
                Ok(builder)
            }
        }
    }
    fn match_item(
        &mut self,
        inverse: bool,
        mut builder: BoolQueryBuilder,
    ) -> Result<BoolQueryBuilder, BoolQueryBuilder> {
        match self.lexer.token {
            Token::Word => {
                if inverse {
                    builder = builder
                        .must_not(
                            ExactQueryBuilder::new()
                                .with_field("body".to_owned())
                                .with_term(self.lexer.slice().to_owned())
                                .build(),
                        )
                        .must_not(
                            ExactQueryBuilder::new()
                                .with_field("title".to_owned())
                                .with_term(self.lexer.slice().to_owned())
                                .build(),
                        )
                        .must_not(
                            ExactQueryBuilder::new()
                                .with_field("tag".to_owned())
                                .with_term(self.lexer.slice().to_owned())
                                .build(),
                        );
                } else {
                    builder = builder
                        .should(
                            FuzzyQueryBuilder::new()
                                .with_field("title".to_owned())
                                .with_term(self.lexer.slice().to_owned())
                                .build(),
                        )
                        .should(
                            FuzzyQueryBuilder::new()
                                .with_field("body".to_owned())
                                .with_term(self.lexer.slice().to_owned())
                                .build(),
                        )
                        .should(
                            ExactQueryBuilder::new()
                                .with_field("tag".to_owned())
                                .with_term(self.lexer.slice().to_owned())
                                .build(),
                        )
                }
                self.lexer.advance();
            }
            Token::QuoteSingle | Token::QuoteDouble => {
                let quote_token = self.lexer.token;
                self.lexer.advance();
                let terms = match self.inside_quote(quote_token) {
                    Ok(terms) => terms,
                    Err(_) => return Err(builder),
                };
                match terms {
                    OneOrMore::One(term) => {
                        if inverse {
                            builder = builder
                                .must_not(
                                    ExactQueryBuilder::new()
                                        .with_field("body".to_owned())
                                        .with_term(term.to_string())
                                        .build(),
                                )
                                .must_not(
                                    ExactQueryBuilder::new()
                                        .with_field("title".to_owned())
                                        .with_term(term.to_string())
                                        .build(),
                                )
                                .must_not(
                                    ExactQueryBuilder::new()
                                        .with_field("tag".to_owned())
                                        .with_term(term.to_string())
                                        .build(),
                                )
                        } else {
                            builder = builder
                                .should(
                                    ExactQueryBuilder::new()
                                        .with_field("body".to_owned())
                                        .with_term(term.to_string())
                                        .build(),
                                )
                                .should(
                                    ExactQueryBuilder::new()
                                        .with_field("title".to_owned())
                                        .with_term(term.to_string())
                                        .build(),
                                )
                                .should(
                                    ExactQueryBuilder::new()
                                        .with_field("tag".to_owned())
                                        .with_term(term.to_string())
                                        .build(),
                                )
                        }
                    }
                    OneOrMore::More(ref terms) => {
                        if inverse {
                            builder = builder
                                .must_not(
                                    PhraseQueryBuilder::new()
                                        .with_field("title".to_owned())
                                        .with_terms(
                                            terms
                                                .into_iter()
                                                .map(|t| t.to_string()),
                                        )
                                        .build(),
                                )
                                .must_not(
                                    PhraseQueryBuilder::new()
                                        .with_field("body".to_owned())
                                        .with_terms(
                                            terms
                                                .into_iter()
                                                .map(|t| t.to_string()),
                                        )
                                        .build(),
                                );
                        } else {
                            builder = builder
                                .should(
                                    PhraseQueryBuilder::new()
                                        .with_field("title".to_owned())
                                        .with_terms(
                                            terms
                                                .into_iter()
                                                .map(|t| t.to_string()),
                                        )
                                        .build(),
                                )
                                .should(
                                    PhraseQueryBuilder::new()
                                        .with_field("body".to_owned())
                                        .with_terms(
                                            terms
                                                .into_iter()
                                                .map(|t| t.to_string()),
                                        )
                                        .build(),
                                );
                        }
                    }
                    _ => {}
                }
            }
            _ => return Err(builder),
        }

        Ok(builder)
    }
    fn tag_item(
        &mut self,
        inverse: bool,
        mut builder: BoolQueryBuilder,
    ) -> Result<BoolQueryBuilder, BoolQueryBuilder> {
        if let Err(_) = self.assert_token(Token::Tag) {
            return Err(builder);
        }

        match self.lexer.token {
            Token::Word => {
                if inverse {
                    builder = builder.must_not(
                        ExactQueryBuilder::new()
                            .with_field("tag".to_owned())
                            .with_term(self.lexer.slice().to_owned())
                            .build(),
                    )
                } else {
                    builder = builder.must(
                        ExactQueryBuilder::new()
                            .with_field("tag".to_owned())
                            .with_term(self.lexer.slice().to_owned())
                            .build(),
                    );
                }
                self.lexer.advance();
            }
            Token::QuoteSingle | Token::QuoteDouble => {
                let quote_token = self.lexer.token;
                self.lexer.advance();
                let terms = match self.inside_quote(quote_token) {
                    Ok(terms) => terms,
                    Err(_) => return Err(builder),
                };
                match terms {
                    OneOrMore::One(term) => {
                        if inverse {
                            builder = builder.must_not(
                                ExactQueryBuilder::new()
                                    .with_field("tag".to_owned())
                                    .with_term(term.to_string())
                                    .build(),
                            )
                        } else {
                            builder = builder.must(
                                ExactQueryBuilder::new()
                                    .with_field("tag".to_owned())
                                    .with_term(term.to_string())
                                    .build(),
                            );
                        }
                    }
                    _ => {}
                }
            }
            _ => return Err(builder),
        }

        Ok(builder)
    }

    fn inside_quote(
        &mut self,
        quote_token: Token,
    ) -> Result<OneOrMore<&'a str>, ()> {
        let mut terms: Vec<_> = vec![];
        loop_panic_when_stuck!({
            match self.lexer.token {
                Token::Inverse => {
                    terms.push("not:");
                    self.lexer.advance();
                }
                Token::Tag => {
                    terms.push("tag:");
                    self.lexer.advance();
                }
                Token::Word => {
                    terms.push(self.lexer.slice());
                    self.lexer.advance();
                }
                Token::QuoteDouble | Token::QuoteSingle => {
                    self.assert_token(quote_token)?;
                    break;
                }
                Token::End => break,
                Token::Error => return Err(()),
            }
        });

        if terms.is_empty() {
            Ok(OneOrMore::Empty)
        } else if terms.len() == 1 {
            Ok(OneOrMore::One(terms.pop().unwrap()))
        } else {
            Ok(OneOrMore::More(terms))
        }
    }

    fn assert_token(&mut self, token: Token) -> Result<(), ()> {
        if token == self.lexer.token {
            self.lexer.advance();
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn test_single_word() {
        let raw = "hello";
        let q = QueryParser::new(raw).parse();

        let s = serde_json::to_string_pretty(&q).unwrap();
        let js_value: Value = serde_json::from_str(s.as_ref()).unwrap();

        assert_eq!(
            js_value,
            json!({
              "bool": {
                "must": [],
                "must_not": [],
                "should": [
                  {
                    "fuzzy": {
                      "title": {
                        "value": "hello",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "fuzzy": {
                      "body": {
                        "value": "hello",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "term": {
                      "tag": "hello"
                    }
                  }
                ],
                "minimum_should_match": null,
                "boost": null
              }
            })
        );
    }

    #[test]
    fn test_a_few_words() {
        let raw = "hello world nice";
        let q = QueryParser::new(raw).parse();

        let s = serde_json::to_string_pretty(&q).unwrap();
        let js_value: Value = serde_json::from_str(s.as_ref()).unwrap();

        eprintln!("{}", s);

        assert_eq!(
            js_value,
            json!({
              "bool": {
                "must": [],
                "must_not": [],
                "should": [
                  {
                    "fuzzy": {
                      "title": {
                        "value": "hello",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "fuzzy": {
                      "body": {
                        "value": "hello",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "term": {
                      "tag": "hello"
                    }
                  },
                  {
                    "fuzzy": {
                      "title": {
                        "value": "world",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "fuzzy": {
                      "body": {
                        "value": "world",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "term": {
                      "tag": "world"
                    }
                  },
                  {
                    "fuzzy": {
                      "title": {
                        "value": "nice",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "fuzzy": {
                      "body": {
                        "value": "nice",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "term": {
                      "tag": "nice"
                    }
                  }
                ],
                "minimum_should_match": null,
                "boost": null
              }
            })
        );
    }

    #[test]
    fn test_quoted() {
        let raw = r#""two words""#;
        let q = QueryParser::new(raw).parse();

        let s = serde_json::to_string_pretty(&q).unwrap();
        let js_value: Value = serde_json::from_str(s.as_ref()).unwrap();

        eprintln!("{}", s);

        assert_eq!(
            js_value,
            json!({
              "bool": {
                "must": [],
                "must_not": [],
                "should": [
                  {
                    "phrase": {
                      "title": {
                        "terms": [
                          "two",
                          "words"
                        ]
                      }
                    }
                  },
                  {
                    "phrase": {
                      "body": {
                        "terms": [
                          "two",
                          "words"
                        ]
                      }
                    }
                  }
                ],
                "minimum_should_match": null,
                "boost": null
              }
            })
        );
    }

    #[test]
    fn test_tag_syntax() {
        let raw = r#"tag:github.com not:tag:"google.com""#;
        let q = QueryParser::new(raw).parse();

        let s = serde_json::to_string_pretty(&q).unwrap();
        let js_value: Value = serde_json::from_str(s.as_ref()).unwrap();

        eprintln!("{}", s);

        assert_eq!(
            js_value,
            json!({
              "bool": {
                "must": [
                  {
                    "term": {
                      "tag": "github.com"
                    }
                  }
                ],
                "must_not": [
                  {
                    "term": {
                      "tag": "google.com"
                    }
                  }
                ],
                "should": [],
                "minimum_should_match": null,
                "boost": null
              }
            })
        );
    }

    #[test]
    fn test_drops_tag_phrase() {
        let raw = r#"tag:"what up" inf"#;
        let q = QueryParser::new(raw).parse();

        let s = serde_json::to_string_pretty(&q).unwrap();
        let js_value: Value = serde_json::from_str(s.as_ref()).unwrap();

        eprintln!("{}", s);

        assert_eq!(
            js_value,
            json!({
              "bool": {
                "must": [],
                "must_not": [],
                "should": [
                  {
                    "fuzzy": {
                      "title": {
                        "value": "inf",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "fuzzy": {
                      "body": {
                        "value": "inf",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "term": {
                      "tag": "inf"
                    }
                  }
                ],
                "minimum_should_match": null,
                "boost": null
              }
            })
        );
    }

    #[test]
    fn test_mixing_up() {
        let raw = r#"not:'alright not: tag:t' tag:foo why"#;
        let q = QueryParser::new(raw).parse();

        let s = serde_json::to_string_pretty(&q).unwrap();
        let js_value: Value = serde_json::from_str(s.as_ref()).unwrap();

        eprintln!("{}", s);

        assert_eq!(
            js_value,
            json!({
              "bool": {
                "must": [
                  {
                    "term": {
                      "tag": "foo"
                    }
                  }
                ],
                "must_not": [
                  {
                    "phrase": {
                      "title": {
                        "terms": [
                          "alright",
                          "not:",
                          "tag:",
                          "t"
                        ]
                      }
                    }
                  },
                  {
                    "phrase": {
                      "body": {
                        "terms": [
                          "alright",
                          "not:",
                          "tag:",
                          "t"
                        ]
                      }
                    }
                  }
                ],
                "should": [
                  {
                    "fuzzy": {
                      "title": {
                        "value": "why",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "fuzzy": {
                      "body": {
                        "value": "why",
                        "distance": 0,
                        "transposition": false
                      }
                    }
                  },
                  {
                    "term": {
                      "tag": "why"
                    }
                  }
                ],
                "minimum_should_match": null,
                "boost": null
              }
            })
        );
    }
}
