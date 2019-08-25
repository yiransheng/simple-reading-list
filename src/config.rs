use lazy_static::lazy_static;
use rand::rngs::OsRng;
use rand::RngCore;

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

pub struct Config {
    pub database_url: String,
    pub toshi_url: String,
    pub toshi_index: String,
    pub host_port: String,
    pub allowed_origin: String,
    pub jwt_secret: Vec<u8>,
}

impl Config {
    fn new() -> Self {
        Self {
            database_url: Self::from_env("DATABASE_URL"),
            toshi_url: Self::from_env("TOSHI_URL"),
            toshi_index: Self::from_env("TOSHI_INDEX"),
            allowed_origin: Self::from_env_or_else("ALLOWED_ORIGIN", || {
                "http://localhost:3000".to_owned()
            }),
            host_port: Self::from_env_or_else("HOST_PORT", || {
                "8080".to_owned()
            }),
            jwt_secret: Self::from_env_or_else("JWT_SECRET", || {
                let mut key = vec![0u8; 32];
                OsRng.fill_bytes(&mut key);

                key
            }),
        }
    }
    fn from_env(variable: &str) -> String {
        std::env::var(variable)
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| panic!("{} must be set", variable))
    }
    fn from_env_or_else<R, F>(variable: &str, f: F) -> R
    where
        F: FnOnce() -> R,
        R: From<String>,
    {
        std::env::var(variable)
            .ok()
            .filter(|s| !s.is_empty())
            .map(Into::into)
            .unwrap_or_else(f)
    }
}
