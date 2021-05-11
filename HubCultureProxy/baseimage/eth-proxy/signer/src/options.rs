use structopt::StructOpt;
use config::ConfigFile;
use crypto::{Address,Secret};
use error::Error;
use rand;
use toml;
use std::path::Path;
use std::{env,fs};


#[derive(Debug,Clone,StructOpt)]
pub struct SignerOptions {
    /// Allow raw transaction signing
    #[structopt(long = "allow-raw-txns")]
    pub allow_raw_txns: bool,
    /// Allow contract-creation transactions
    #[structopt(long = "allow-contract-creation")]
    pub allow_contract_creation: bool,
    /// Default target for function-calls
    #[structopt(name = "default-contract", long = "default-contract")]
    pub default_contract: Option<Address>,
    /// Path to optional config file
    #[structopt(name = "signer-config", long = "signer-config")]
    config_file: Option<String>,
    /// Raw hex-encoded secret key
    #[structopt(name = "secret-key", long = "secret-key")]
    secret_key: Option<Secret>,
    /// Path to file containing secret key
    #[structopt(name = "secret-file", long = "secret-file")]
    key_file: Option<String>,
    /// Path to file containing secret key (lazily generated)
    #[structopt(name = "secret-cache", long = "secret-cache")]
    key_cache: Option<String>,
    /// Name of env var containing secret key
    #[structopt(name = "secret-var", long = "secret-var")]
    key_var: Option<String>,
}


impl SignerOptions {

    pub fn from_args() -> Self { StructOpt::from_args() }

    pub fn load_config(&self) -> Result<ConfigFile,Error> {
        if let Some(path) = self.config_file.as_ref() {
            let buf = fs::read_to_string(path)?;
            let config = toml::from_str(&buf)?;
            Ok(config)
        } else {
            Ok(Default::default())
        }
    }

    pub fn load_secret(&self) -> Result<Secret,Error> {
        if let Some(secret) = self.secret_key.as_ref() {
            Ok(*secret)
        } else if let Some(path) = self.key_file.as_ref() {
            let buf = fs::read_to_string(path)?;
            let secret = buf.trim().parse()?;
            Ok(secret)
        } else if let Some(path) = self.key_cache.as_ref() {
            if Path::new(path).is_file() {
                let buf = fs::read_to_string(path)?;
                let secret = buf.trim().parse()?;
                Ok(secret)
            } else {
                info!("Initializing key cache: {}",path);
                let secret = rand::random();
                let buf = format!("{:?}\n",secret);
                fs::write(path,buf)?;
                Ok(secret)
            }
        } else if let Some(var) = self.key_var.as_ref() {
            let buf = env::var(var).map_err(|e| {
                Error::message(e.to_string())
            })?;
            let secret = buf.trim().parse()?;
            Ok(secret)
        } else {
            warn!("Using random ephemoral key (none specified)");
            let secret = rand::random();
            Ok(secret)
        }
    }
}
