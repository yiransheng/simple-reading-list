use actix_web::client::Client;
use actix_web::http::{header::CONTENT_TYPE, uri, StatusCode};
use actix_web::Error;
use futures::future::Future;
use serde_derive::*;

mod index;
mod query;
mod query_parser;

use self::query_parser::QueryParser;
use crate::config::CONFIG;
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
        SearchClient {
            rest_client: Client::default(),
            insert_doc_endpoint: insert_doc_endpoint(&CONFIG.toshi_url),
            query_doc_endpoint: query_doc_endpoint(&CONFIG.toshi_url),
        }
    }

    pub fn insert_doc(
        &self,
        doc: BookmarkDoc,
    ) -> impl Future<Item = Result<(), ServiceError>, Error = Error> {
        #[derive(Serialize)]
        struct InsertPayload<D> {
            options: InsertOptions,
            document: D,
        }
        #[derive(Serialize)]
        struct InsertOptions {
            commit: bool,
        }
        impl<D> InsertPayload<D> {
            fn new(document: D) -> Self {
                Self {
                    options: InsertOptions { commit: true },
                    document,
                }
            }
        }
        self.rest_client
            .put(&self.insert_doc_endpoint)
            .header(CONTENT_TYPE, "application/json")
            .send_json(&InsertPayload::new(doc))
            .from_err()
            .map(|resp| {
                eprintln!("Insert: {:?}", resp);
                if resp.status() == StatusCode::CREATED {
                    Ok(())
                } else {
                    Err(ServiceError::InternalServerError)
                }
            })
    }

    pub fn query_docs(
        &self,
        q: &str,
    ) -> impl Future<Item = SearchResults, Error = Error> {
        #[derive(Serialize)]
        struct QueryPayload<Q> {
            query: Q,
            limit: u32,
        }

        eprintln!("Query: {}", q);
        let q = QueryParser::new(q).parse();
        eprintln!("{}", serde_json::to_string_pretty(&q).unwrap());
        self.rest_client
            .post(&self.query_doc_endpoint)
            .header(CONTENT_TYPE, "application/json")
            .send_json(&QueryPayload {
                query: q,
                limit: 25,
            })
            .from_err()
            .and_then(|mut resp| {
                eprintln!("Results: {:?}", resp);
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
        .path_and_query("/bookmarks")
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
