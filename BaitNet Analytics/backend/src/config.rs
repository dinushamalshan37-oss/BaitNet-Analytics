use std::{env, error::Error, io};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub redis_ttl_seconds: u64,
    pub backend_port: u16,
    pub frontend_origin: String,
    pub virustotal_api_key: Option<String>,
    pub shodan_api_key: Option<String>,
    pub hibp_api_key: Option<String>,
    pub phishtank_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let database_url = required("DATABASE_URL")?;
        let redis_url = required("REDIS_URL")?;
        let redis_ttl_seconds = env::var("REDIS_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_owned())
            .parse::<u64>()?;
        if redis_ttl_seconds == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "REDIS_TTL_SECONDS must be greater than zero").into());
        }
        let backend_port = env::var("BACKEND_PORT")
            .unwrap_or_else(|_| "8080".to_owned())
            .parse::<u16>()?;
        if backend_port == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "BACKEND_PORT must be greater than zero").into());
        }
        Ok(Self {
            database_url,
            redis_url,
            redis_ttl_seconds,
            backend_port,
            frontend_origin: env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_owned()),
            virustotal_api_key: optional("VIRUSTOTAL_API_KEY"),
            shodan_api_key: optional("SHODAN_API_KEY"),
            hibp_api_key: optional("HIBP_API_KEY"),
            phishtank_api_key: optional("PHISHTANK_API_KEY"),
        })
    }
}

fn required(name: &str) -> Result<String, Box<dyn Error>> {
    env::var(name).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("Missing required environment variable {name}")).into())
}

fn optional(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}
