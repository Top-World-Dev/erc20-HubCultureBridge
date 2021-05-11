use std::collections::HashSet;
use crypto::Address;


#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Contracts { 
    #[serde(default,skip_serializing_if = "HashSet::is_empty")]
    whitelist: HashSet<Address>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    default: Option<Address>,
}


impl Contracts {

    pub fn new(whitelist: HashSet<Address>) -> Self {
        let default = if whitelist.len() == 1 {
            whitelist.iter().next().map(|addr| *addr)
        } else {
            None
        };
        Self { whitelist, default }
    }

    pub fn is_empty(&self) -> bool { self.whitelist.is_empty() }

    pub fn contains(&self, addr: Address) -> bool { self.whitelist.contains(&addr) }

    pub fn get_default(&self) -> Option<Address> {
        if let Some(addr) = self.default {
            debug_assert!(self.is_allowed(addr));
            Some(addr)
        } else {
            None
        }
    }

    pub fn set_default(&mut self, addr: Address) -> Result<(),::Error> {
        if self.is_allowed(addr) {
            self.default = Some(addr);
            Ok(())
        } else {
            Err(::Error::message("cannot set default address (not whitelisted)"))
        }
    }

    pub fn is_allowed(&self, addr: Address) -> bool {
        self.is_empty() || self.contains(addr)
    }
}
