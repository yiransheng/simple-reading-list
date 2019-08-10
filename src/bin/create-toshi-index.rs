use std::borrow::Cow;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use actix_rt::System;
use actix_web::client::{Client, SendRequestError};
use actix_web::error::ResponseError;
use actix_web::http::header::CONTENT_TYPE;
use derive_more::*;
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
    #[display(fmt = "Missing TOSHI_HOST")]
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
    System::new("create_index").block_on(lazy(|| {
        let client = Client::default();
        let uri = format!("http://{}/{}/_create", toshi_host, index_name);
        println!("Endpoint: {}", uri);

        client
            .put(uri)
            .header(CONTENT_TYPE, "application/json")
            .send_body(payload)
            .map_err(CreateIndexError::RequestError)
            .and_then(|mut resp| {
                println!("{:?}", resp);
                resp.body()
                    .map_err(|err| {
                        CreateIndexError::ResponseError(Box::new(err))
                    })
                    .map(|b| {
                        println!(
                            "Response Body:\n {}",
                            ::std::str::from_utf8(&b).unwrap()
                        );
                    })
            })
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let payload = get_toshi_index(&opt)?;
    let host: Cow<String> = std::env::var("TOSHI_HOST")
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
