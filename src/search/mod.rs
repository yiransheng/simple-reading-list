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
use crate::models::{BookmarkDoc, SearchResults};

#[derive(Deserialize)]
pub struct Search {
    pub q: String,
}

pub struct SearchClient {
    rest_client: Client,
    insert_doc_endpoint: uri::Uri,
    query_doc_endpoint: uri::Uri,
}

impl SearchClient {
    pub fn new() -> Self {
        let toshi_host = std::env::var("TOSHI_HOST")
            .unwrap_or_else(|_| "localhost:8000".to_owned());

        SearchClient {
            rest_client: Client::default(),
            insert_doc_endpoint: insert_doc_endpoint(toshi_host.as_ref()),
            query_doc_endpoint: query_doc_endpoint(toshi_host.as_ref()),
        }
    }

    pub fn insert_doc(
        &self,
        doc: BookmarkDoc,
    ) -> impl Future<Item = (), Error = ServiceError> {
        #[derive(Serialize)]
        struct InsertPayload<D> {
            options: InsertOptions,
            doc: D,
        }
        #[derive(Serialize)]
        struct InsertOptions {
            commit: bool,
        }
        impl<D> InsertPayload<D> {
            fn new(doc: D) -> Self {
                Self {
                    options: InsertOptions { commit: true },
                    doc,
                }
            }
        }
        self.rest_client
            .put(&self.insert_doc_endpoint)
            .header(CONTENT_TYPE, "application/json")
            .send_json(&InsertPayload::new(doc))
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
    ) -> impl Future<Item = SearchResults, Error = Error> {
        eprintln!("Query: {}", q);
        let q = QueryParser::new(q).parse().unwrap();
        eprintln!("{}", serde_json::to_string_pretty(&q).unwrap());
        self.rest_client
            .put(&self.query_doc_endpoint)
            .header(CONTENT_TYPE, "application/json")
            .send_json(&json!({
                "query": &q,
                "limit": 25,
            }))
            .from_err()
            .and_then(|mut resp| {
                eprintln!("Query: {:?}", resp);
                // toshi response does not have correct Content-Type header
                // so cannot use .json() here
                resp.body().from_err().and_then(|body| {
                    serde_json::from_slice(&body).map_err(Error::from)
                })
            })
    }
}

fn insert_doc_endpoint(toshi_host: &str) -> uri::Uri {
    uri::Builder::new()
        .scheme("http")
        .authority(toshi_host)
        // LOCAL Toshi workaround:...
        .path_and_query("/bookmarks/_add")
        .build()
        .expect("Invalid endpoint")
}

fn query_doc_endpoint(toshi_host: &str) -> uri::Uri {
    uri::Builder::new()
        .scheme("http")
        .authority(toshi_host)
        .path_and_query("/bookmarks")
        .build()
        .expect("Invalid endpoint")
}
