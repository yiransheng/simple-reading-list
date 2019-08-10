use std::borrow::Cow;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use actix_rt::System;
use actix_web::client::{Client, SendRequestError};
use actix_web::error::ResponseError;
use actix_web::http::header::CONTENT_TYPE;
use actix_web::http::StatusCode;
use derive_more::*;
use dotenv::dotenv;
use futures::future::{lazy, Future};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "create-toshi-index")]
struct Opt {
    #[structopt(short = "t", long = "toshi-host")]
    toshi_host: Option<String>,

    #[structopt(short = "n", long = "name")]
    index_name: Option<String>,

    #[structopt(name = "INDEX_FILE", parse(from_os_str))]
    index: PathBuf,
}

#[derive(Debug, Display)]
pub enum CreateIndexError {
    #[display(fmt = "Missing TOSHI_URL")]
    MissingHostError,

    #[display(fmt = "Missing TOSHI_INDEX")]
    MissingNameError,

    #[display(fmt = "Request Failed: {}", _0)]
    RequestError(SendRequestError),

    #[display(fmt = "Payload Error: {}", _0)]
    ResponseError(Box<dyn ResponseError>),
}

impl Error for CreateIndexError {}

fn get_toshi_index(opt: &Opt) -> Result<String, Box<dyn Error>> {
    let mut conf = File::open(&opt.index)?;
    let mut contents = String::new();
    conf.read_to_string(&mut contents)?;

    Ok(contents)
}

fn create_index(
    toshi_host: &str,
    index_name: &str,
    payload: String,
) -> Result<(), CreateIndexError> {
    let toshi_host = toshi_host.trim();
    let index_name = index_name.trim();

    let get_uri = format!("http://{}/{}", toshi_host, index_name);
    let put_uri = format!("http://{}/{}/_create", toshi_host, index_name);

    System::new("create_index").block_on(lazy(|| {
        let client = Client::default();
        println!("Endpoint: {}", &put_uri);

        client
            .get(&get_uri)
            .send()
            .map_err(CreateIndexError::RequestError)
            .and_then(
                |resp| -> Box<Future<Item = (), Error = CreateIndexError>> {
                    if resp.status() == StatusCode::NOT_FOUND {
                        let client = Client::default();
                        Box::new(
                            client
                                .put(&put_uri)
                                .header(CONTENT_TYPE, "application/json")
                                .send_body(payload)
                                .map_err(CreateIndexError::RequestError)
                                .and_then(|mut resp| {
                                    println!("{:?}", resp);
                                    resp.body()
                                        .map_err(|err| {
                                            CreateIndexError::ResponseError(
                                                Box::new(err),
                                            )
                                        })
                                        .map(|_| ())
                                }),
                        )
                    } else {
                        Box::new(futures::future::ok(()))
                    }
                },
            )
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let opt = Opt::from_args();
    let payload = get_toshi_index(&opt)?;
    let host: Cow<String> = std::env::var("TOSHI_URL")
        .map(Cow::Owned)
        .map_err(|_| CreateIndexError::MissingHostError)
        .or_else(|_| {
            opt.toshi_host
                .as_ref()
                .map(|s| Cow::Borrowed(s))
                .ok_or_else(|| CreateIndexError::MissingHostError)
        })?;

    let name: Cow<String> = std::env::var("TOSHI_INDEX")
        .map(Cow::Owned)
        .map_err(|_| CreateIndexError::MissingNameError)
        .or_else(|_| {
            opt.index_name
                .as_ref()
                .map(|s| Cow::Borrowed(s))
                .ok_or_else(|| CreateIndexError::MissingNameError)
        })?;

    create_index(&host, &name, payload)?;

    Ok(())
}
