use structopt::StructOpt;
use std::net::IpAddr;
use std::path::PathBuf;
use native_tls::Identity;
use error::Error;
use std::fs;

#[derive(Debug,Clone,StructOpt)]
pub struct ServerOptions {
    /// Password for PKCS-12 archive
    #[structopt(long = "tls-cert-pass",name = "tls-cert-pass",default_value = "password")]
    certpass: String,
    /// Path to PKCS-12 archive file
    #[structopt(long = "tls-cert-file",name = "tls-cert-file",default_value = "certificate.p12",parse(from_os_str))]
    certfile: PathBuf,
    /// Indicates whether to use tls
    #[structopt(long = "use-tls")]
    pub use_tls: bool,
    /// Path to whitelist file
    #[structopt(long = "ip-whitelist",name = "ip-whitelist",parse(from_os_str))]
    whitelist: Option<PathBuf>,
}


impl ServerOptions {

    pub fn from_args() -> Self { <Self as StructOpt>::from_args() }

    pub fn load_identity(&self) -> Result<Identity,Error> {
        if self.certfile.is_file() {
            let archive = fs::read(&self.certfile)?;
            let identity = Identity::from_pkcs12(&archive,&self.certpass).map_err(|err| {
                error!("Failed to decrypt PKCS-12 archive: {:?}",err);
                Error::message("Failed to decrypt PKCS-12 archive (see logs for details)")
            })?;
            Ok(identity)
        } else {
            let msg = format!("Unable to locate archive file (`{}`)",self.certfile.display());
            Err(Error::message(msg))
        }
    }

    pub fn load_whitelist(&self) -> Result<Option<Vec<IpAddr>>,Error> {
        if let Some(ref path) = self.whitelist {
            if path.is_file() {
                let raw = fs::read_to_string(path)?;
                let parse = raw.split_whitespace()
                    .map(|s| s.parse())
                    .collect::<Result<_,_>>();
                match parse {
                    Ok(addrs) => Ok(Some(addrs)),
                    Err(err) => {
                        let msg = format!("Failed to parse whitelist ({})",err);
                        Err(Error::message(msg))
                    },
                }
            } else {
                let msg = format!("Unable to locate whitelist file (`{}`)",path.display());
                Err(Error::message(msg))
            }
        } else {
            Ok(Default::default())
        }
    }
}
