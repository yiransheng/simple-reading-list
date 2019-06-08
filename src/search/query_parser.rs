use super::query::*;
use crate::error::ServiceError;

#[derive(Debug, Copy, Clone)]
pub struct ParseError {}

impl Into<ServiceError> for ParseError {
    fn into(self) -> ServiceError {
        ServiceError::BadRequest(String::new())
    }
}

pub struct QueryParser<'a> {
    query_str: &'a str,
}

impl<'a> QueryParser<'a> {
    pub fn new(query_str: &'a str) -> Self {
        Self { query_str }
    }
    pub fn parse(self) -> Result<Query, ParseError> {
        let q = BoolQueryBuilder::new()
            .should(self.query_str.split(" ").map(|term| {
                FuzzyQueryBuilder::new()
                    .with_field("title".to_owned())
                    .with_term(term.to_owned())
                    .build()
            }))
            .should(self.query_str.split(" ").map(|term| {
                FuzzyQueryBuilder::new()
                    .with_field("body".to_owned())
                    .with_term(term.to_owned())
                    .build()
            }))
            .should(self.query_str.split(" ").map(|term| {
                ExactQueryBuilder::new()
                    .with_field("tags".to_owned())
                    .with_term(term.to_owned())
                    .build()
            }))
            .build();

        Ok(q)
    }
}
