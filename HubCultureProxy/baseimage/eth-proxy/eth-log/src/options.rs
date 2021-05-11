use ethrpc::types::U256;
use ethrpc::Url;
use config::{ConfigFile,Config};
use error::Error;
use serde::de::DeserializeOwned;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use toml;



#[derive(Debug,Clone,StructOpt)]
pub struct RunOptions {
    /// Path to configuration file
    #[structopt(name = "config-path",long="config-file",default_value="config.toml")]
    pub config_path: String,
    /// Path to template directory
    #[structopt(name = "template-path",long="template-dir",default_value="templates")]
    pub template_dir: String,
    /// Address of ethereum node
    #[structopt(name = "node-url",long="node-addr",default_value="ws://127.0.0.1:8546")]
    pub node_addr: Url,
    /// Block number to start stream from
    #[structopt(name = "block-number",long="start-block",default_value="0x0")]
    pub start_block: U256, 
    /// Number of blocks to lag by
    #[structopt(name = "block-count",long="lag-by",default_value="3")]
    pub lag_by: u8,
}


impl RunOptions {

    pub fn load_config(&self) -> Result<Config,Error> {
        let config_file: ConfigFile = load_toml(&self.config_path)?;
        let config = Config::try_from(config_file)?;
        Ok(config)
    }

    pub fn load_templates(&self) -> Result<HashMap<String,String>,Error> {
        load_dir(&self.template_dir).collect()
    }
}



fn load_toml<T>(path: impl AsRef<Path>) -> Result<T,Error> where T: DeserializeOwned {
    let path = path.as_ref();
    if path.is_file() {
        let buf = fs::read_to_string(path)?;
        let parsed = toml::from_str(&buf).map_err(|e| {
            let msg = format!("Failed to parse config: {}",e);
            Error::message(msg)
        })?;
        Ok(parsed)
    } else {
        let msg = format!("unable to locate file `{}`",path.display());
        Err(Error::message(msg))
    }
}



fn load_dir(dir: impl AsRef<Path>) -> impl Iterator<Item=Result<(String,String),Error>> {
    WalkBuilder::new(dir.as_ref()).git_global(false)
        .git_ignore(false).git_exclude(false)
        .add_custom_ignore_filename(".ignore").build()
        .filter_map(|rslt| rslt.ok().filter(|entry| entry.path().is_file()))
        .map(move |entry| {
            let filedata = fs::read_to_string(entry.path())?;
            let filename = entry.path().strip_prefix(dir.as_ref()).map_err(|e| {
                Error::message(e.to_string())
            })?;
            let filename = filename.to_str().ok_or_else(|| {
                Error::message("non-utf8 filename")
            })?;
            Ok((filename.to_owned(),filedata))
        })
}
