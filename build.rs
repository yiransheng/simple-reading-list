#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AssetConfig {
    src: String,
    target: String,
}

fn main() {
    // project folder
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let assets_settings = root.join("assets/assets.json");

    asset_macros(assets_settings).expect("build failed");
}

fn asset_macros<P: AsRef<Path>>(path: P) -> Result<(), Box<Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let assets: HashMap<String, AssetConfig> = serde_json::from_reader(reader)?;

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("asset_macro.rs");
    let mut f = File::create(&dest_path).unwrap();

    write!(
        f,
        "
    macro_rules! asset {{
      ($url: expr) => {{
          match $url {{
    "
    )?;
    for (url, config) in assets.iter() {
        writeln!(f, "    \"{}\" => \"{}\",", url, &config.target)?;
    }
    write!(
        f,
        "
          _ => $url,
        }}
      }}
    }}
    "
    )?;

    Ok(())
}
