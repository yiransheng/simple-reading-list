use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use actix_rt::System;
use actix_web::client::{Client, SendRequestError};
use actix_web::error::ResponseError;
use actix_web::http::header::CONTENT_TYPE;
use derive_more::*;
use futures::future::{lazy, Future, Stream};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "create-toshi-index")]
struct Opt {
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    config: PathBuf,

    #[structopt(name = "INDEX_FILE", parse(from_os_str))]
    index: PathBuf,
}

#[derive(Debug, Display)]
pub enum CreateIndexError {
    #[display(fmt = "Bad Toshi config.toml: {}", _0)]
    BadToshiConfig(String),

    #[display(fmt = "Request Failed: {}", _0)]
    RequestError(SendRequestError),

    #[display(fmt = "Payload Error: {}", _0)]
    ResponseError(Box<dyn ResponseError>),
}

impl Error for CreateIndexError {}

fn get_toshi_host(opt: &Opt) -> Result<String, Box<dyn Error>> {
    use toml::Value;

    let mut conf = File::open(&opt.config)?;
    let mut contents = String::new();
    conf.read_to_string(&mut contents)?;

    let conf: Value = toml::de::from_str(contents.as_ref())?;

    match conf {
        Value::Table(ref table) => {
            let host = table.get("host").ok_or_else(|| {
                CreateIndexError::BadToshiConfig("Missing host".to_owned())
            })?;
            let port = table.get("port").ok_or_else(|| {
                CreateIndexError::BadToshiConfig("Missing port".to_owned())
            })?;
            let host = match *host {
                Value::String(ref s) => Ok(s),
                _ => {
                    Err(CreateIndexError::BadToshiConfig("Bad host".to_owned()))
                }
            }?;
            let port = match *port {
                Value::Integer(ref p) if *p > 0 => Ok(p),
                _ => {
                    Err(CreateIndexError::BadToshiConfig("Bad port".to_owned()))
                }
            }?;

            let uri = format!("{}:{}", host, port);

            Ok(uri)
        }
        _ => Err(CreateIndexError::BadToshiConfig(
            "Invalid config.toml".to_owned(),
        )
        .into()),
    }
}

fn get_toshi_index(opt: &Opt) -> Result<String, Box<dyn Error>> {
    let mut conf = File::open(&opt.index)?;
    let mut contents = String::new();
    conf.read_to_string(&mut contents)?;

    Ok(contents)
}

fn create_index(
    toshi_host: &str,
    payload: String,
) -> Result<(), CreateIndexError> {
    System::new("create_index").block_on(lazy(|| {
        let client = Client::default();
        let uri = format!("http://{}/bookmarks/_create", toshi_host);
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
    let host = get_toshi_host(&opt)?;
    let payload = get_toshi_index(&opt)?;

    create_index(&host, payload)?;

    Ok(())
}
