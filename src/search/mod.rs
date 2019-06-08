use actix_web::client::Client;
use actix_web::error::ResponseError;
use actix_web::http::{header::CONTENT_TYPE, uri};
use actix_web::web::Bytes;
use actix_web::Error;
use futures::future::{lazy, Future};
use serde_derive::*;
use serde_json::json;

mod index;
mod query;
mod query_parser;

use self::query_parser::QueryParser;
use crate::error::ServiceError;
use crate::models::BookmarkDoc;

#[derive(Deserialize)]
pub struct Search {
    pub q: String,
}

pub struct SearchClient {
    rest_client: Client,
    toshi_host: String,
}

impl SearchClient {
    pub fn new() -> Self {
        let toshi_host = std::env::var("TOSHI_HOST")
            .unwrap_or_else(|_| "localhost:8000".to_owned());

        SearchClient {
            rest_client: Client::default(),
            toshi_host,
        }
    }

    pub fn insert_doc(
        &self,
        doc: &BookmarkDoc,
    ) -> impl Future<Item = (), Error = ServiceError> {
        self.rest_client
            .put(self.insert_doc_endpoint())
            .header(CONTENT_TYPE, "application/json")
            .send_json(&json!({
                "options": { "commit": true },
                "document": doc
            }))
            .map_err(|_| ServiceError::InternalServerError)
            .and_then(|mut resp| {
                eprintln!("Insert: {:?}", resp);
                resp.body()
                    .map_err(|_| ServiceError::InternalServerError)
                    .map(|b| {
                        eprintln!("{:?}", b);
                    })
            })
    }

    pub fn query_docs(
        &self,
        q: &str,
    ) -> impl Future<Item = Bytes, Error = Error> {
        eprintln!("Query: {}", q);
        let q = QueryParser::new(q).parse().unwrap();
        eprintln!("{}", serde_json::to_string_pretty(&q).unwrap());
        self.rest_client
            .post(self.query_doc_endpoint())
            .header(CONTENT_TYPE, "application/json")
            .send_json(&json!({
                "query": &q,
                "limit": 25,
            }))
            .from_err()
            .and_then(|mut resp| {
                eprintln!("Query: {:?}", resp);
                resp.body().from_err()
            })
    }

    fn insert_doc_endpoint(&self) -> uri::Uri {
        uri::Builder::new()
            .scheme("http")
            .authority::<&str>(self.toshi_host.as_ref())
            // LOCAL Toshi workaround:...
            .path_and_query("/bookmarks/_add")
            .build()
            .expect("Invalid endpoint")
    }

    fn query_doc_endpoint(&self) -> uri::Uri {
        uri::Builder::new()
            .scheme("http")
            .authority::<&str>(self.toshi_host.as_ref())
            .path_and_query("/bookmarks")
            .build()
            .expect("Invalid endpoint")
    }
}
