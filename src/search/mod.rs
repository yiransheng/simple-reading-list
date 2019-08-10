use actix_web::client::Client;
use actix_web::http::{header::CONTENT_TYPE, uri, StatusCode};
use actix_web::Error;
use futures::future::Future;
use serde_derive::*;

mod index;
mod query;
mod query_parser;

pub use self::query::Query;
pub use self::query_parser::QueryParser;
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
        let client = SearchClient {
            rest_client: Client::default(),
            insert_doc_endpoint: insert_doc_endpoint(),
            query_doc_endpoint: query_doc_endpoint(),
        };
        log::info!("Created toshi client");
        log::info!("  toshi: {}", client.insert_doc_endpoint);
        log::info!("  toshi: {}", client.query_doc_endpoint);

        client
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
                if resp.status() == StatusCode::CREATED {
                    Ok(())
                } else {
                    Err(ServiceError::InternalServerError)
                }
            })
    }

    pub fn query_docs(
        &self,
        q: Query,
    ) -> impl Future<Item = SearchResults, Error = Error> {
        #[derive(Serialize)]
        struct QueryPayload<Q> {
            query: Q,
            limit: u32,
        }

        self.rest_client
            .post(&self.query_doc_endpoint)
            .header(CONTENT_TYPE, "application/json")
            .send_json(&QueryPayload {
                query: q,
                limit: 25,
            })
            .from_err()
            .and_then(|mut resp| {
                resp.body().from_err().and_then(|body| {
                    if body.is_empty() {
                        Ok(SearchResults::default())
                    } else {
                        serde_json::from_slice(&body).map_err(Error::from)
                    }
                })
            })
    }
}

fn insert_doc_endpoint() -> uri::Uri {
    let index_path = format!("/{}", CONFIG.toshi_index);

    uri::Builder::new()
        .scheme("http")
        .authority(CONFIG.toshi_url.as_str().trim())
        .path_and_query(index_path.as_str().trim())
        .build()
        .expect("Invalid endpoint")
}

fn query_doc_endpoint() -> uri::Uri {
    let index_path = format!("/{}", CONFIG.toshi_index);

    uri::Builder::new()
        .scheme("http")
        .authority(CONFIG.toshi_url.as_str().trim())
        .path_and_query(index_path.as_str().trim())
        .build()
        .expect("Invalid endpoint")
}
