use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Query {
    Boolean { bool: BoolQuery },
    Exact(ExactTerm),
    Fuzzy(FuzzyQuery),
    Phrase(PhraseQuery),
}
impl Query {
    pub fn is_empty(&self) -> bool {
        match self {
            Query::Boolean { bool: b } => b.is_empty(),
            _ => unreachable!(
                "Expect this check to only be called on Bool query"
            ),
        }
    }
}

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct ExactTerm {
    term: KeyValue<String>,
}

impl ExactTerm {
    pub fn new(term: KeyValue<String>) -> Self {
        Self { term }
    }
}

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct BoolQuery {
    #[serde(default = "Vec::new")]
    must: Vec<Query>,
    #[serde(default = "Vec::new")]
    must_not: Vec<Query>,
    #[serde(default = "Vec::new")]
    should: Vec<Query>,
    minimum_should_match: Option<u64>,
    boost: Option<f64>,
}

impl BoolQuery {
    pub fn is_empty(&self) -> bool {
        self.must.is_empty()
            && self.must_not.is_empty()
            && self.should.is_empty()
    }
}

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct FuzzyQuery {
    fuzzy: KeyValue<FuzzyTerm>,
}

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct FuzzyTerm {
    value: String,
    #[serde(default)]
    distance: u8,
    #[serde(default)]
    transposition: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PhraseQuery {
    phrase: KeyValue<TermPair>,
}
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TermPair {
    terms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offsets: Option<Vec<usize>>,
}

impl FuzzyTerm {
    pub fn new(value: String, distance: u8, transposition: bool) -> Self {
        Self {
            value,
            distance,
            transposition,
        }
    }
}

impl FuzzyQuery {
    pub fn new(fuzzy: KeyValue<FuzzyTerm>) -> Self {
        Self { fuzzy }
    }
}

impl TermPair {
    pub fn new(terms: Vec<String>, offsets: Option<Vec<usize>>) -> Self {
        TermPair { terms, offsets }
    }
}

impl PhraseQuery {
    pub fn new(phrase: KeyValue<TermPair>) -> Self {
        PhraseQuery { phrase }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyValue<T> {
    pub field: String,
    pub value: T,
}

impl<T> Serialize for KeyValue<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut m = serializer.serialize_map(Some(1))?;
        m.serialize_entry(&self.field, &self.value)?;
        m.end()
    }
}

pub struct BoolQueryBuilder {
    __must: Vec<Query>,
    __must_not: Vec<Query>,
    __should: Vec<Query>,
    __minimum_should_match: Option<u64>,
    __boost: Option<f64>,
}
impl BoolQueryBuilder {
    pub fn new() -> Self {
        Self {
            __must: vec![],
            __must_not: vec![],
            __should: vec![],
            __minimum_should_match: None,
            __boost: None,
        }
    }
    pub fn build(self) -> Query {
        Query::Boolean {
            bool: BoolQuery {
                must: self.__must,
                must_not: self.__must_not,
                should: self.__should,
                minimum_should_match: self.__minimum_should_match,
                boost: self.__boost,
            },
        }
    }
    pub fn must(mut self, q: Query) -> Self {
        self.__must.push(q);
        self
    }
    pub fn must_not(mut self, q: Query) -> Self {
        self.__must_not.push(q);
        self
    }
    pub fn should(mut self, q: Query) -> Self {
        self.__should.push(q);
        self
    }
    pub fn minimum_should_match(mut self, s: u64) -> Self {
        self.__minimum_should_match = Some(s);
        self
    }
    pub fn boost(mut self, b: f64) -> Self {
        self.__boost = Some(b);
        self
    }
}

pub struct ExactQueryBuilder {
    field: Option<String>,
    term: Option<String>,
    // TODO: offsets
}
impl ExactQueryBuilder {
    pub fn new() -> Self {
        Self {
            field: None,
            term: None,
        }
    }
    pub fn build(self) -> Query {
        Query::Exact(ExactTerm::new(KeyValue {
            field: self.field.unwrap(),
            value: self.term.unwrap(),
        }))
    }
    pub fn with_field(mut self, f: String) -> Self {
        self.field = Some(f);
        self
    }
    pub fn with_term(mut self, term: String) -> Self {
        self.term = Some(term);
        self
    }
}

pub struct PhraseQueryBuilder {
    field: Option<String>,
    terms: Vec<String>,
    // TODO: offsets
}
impl PhraseQueryBuilder {
    pub fn new() -> Self {
        Self {
            field: None,
            terms: vec![],
        }
    }
    pub fn build(self) -> Query {
        Query::Phrase(PhraseQuery::new(KeyValue {
            field: self.field.unwrap(),
            value: TermPair::new(self.terms, None),
        }))
    }
    pub fn with_field(mut self, f: String) -> Self {
        self.field = Some(f);
        self
    }
    pub fn with_terms<I>(mut self, terms: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.terms.extend(terms.into_iter());
        self
    }
    pub fn with_term(mut self, term: String) -> Self {
        self.terms.push(term);
        self
    }
}

pub struct FuzzyQueryBuilder {
    field: Option<String>,
    value: Option<String>,
    distance: u8,
    transposition: bool,
}

impl FuzzyQueryBuilder {
    pub fn new() -> Self {
        Self {
            field: None,
            value: None,
            distance: 0,
            transposition: false,
        }
    }
    pub fn build(self) -> Query {
        Query::Fuzzy(FuzzyQuery::new(KeyValue {
            field: self.field.unwrap(),
            value: FuzzyTerm::new(
                self.value.unwrap(),
                self.distance,
                self.transposition,
            ),
        }))
    }
    pub fn with_field(mut self, f: String) -> Self {
        self.field = Some(f);
        self
    }
    pub fn with_term(mut self, v: String) -> Self {
        self.value = Some(v);
        self
    }
    pub fn with_distance(mut self, d: u8) -> Self {
        self.distance = d;
        self
    }
    pub fn with_transposition(mut self, t: bool) -> Self {
        self.transposition = t;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn test_ser_fuzzy() {
        let q = FuzzyQueryBuilder::new()
            .with_field("test_text".to_owned())
            .with_term("document".to_owned())
            .build();

        let s = serde_json::to_string(&q).unwrap();
        let js_value: Value = serde_json::from_str(s.as_ref()).unwrap();

        assert_eq!(
            js_value,
            json!({
                "fuzzy": {
                    "test_text": {
                        "value": "document",
                        "distance": 0,
                        "transposition": false
                    }
                }
            })
        );
    }
}
