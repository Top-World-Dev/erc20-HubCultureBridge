//! Incoming connection policies
//!
//! ## Example
//!
//! ```
//! extern crate proxy;
//!
//! use proxy::policy::{
//!     Policy,
//!     Filter,
//! };
//!
//! # fn main() {
//!
//! let whitelist = [
//!     "127.0.0.1",
//!     "10.0.0.1",
//! ];
//!
//! let filter = Filter::whitelist(
//!     whitelist.iter().map(|addr| addr.parse().unwrap())
//! );
//!
//! assert_eq!(filter.policy("127.0.0.1".parse().unwrap()),Policy::Allow);
//!
//! assert_eq!(filter.policy("192.168.0.1".parse().unwrap()),Policy::Deny);
//!
//! # }
//!
//! ```
use std::collections::HashMap;
use std::net::IpAddr;


/// Describes the policy to be applied to an incoming connection
///
#[derive(Debug,Copy,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Policy {
    Allow,
    Deny,
}


// TODO: Support ip ranges/CIDR blocks.

/// Policy filter (maps ip addresses to policies)
///
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Filter {
    defined: HashMap<IpAddr,Policy>,
    default: Policy,
}


impl Default for Filter {

    fn default() -> Self { Self::new(Policy::Deny) }
}


impl Filter {

    /// Build new filter with specified default policy
    ///
    pub fn new(default: Policy) -> Self {
        let defined = Default::default();
        Self { defined, default }
    }

    /// Get policy for specified ip address
    ///
    /// If no specific policy is defined, the default policy is returned.
    pub fn policy(&self, addr: IpAddr) -> Policy {
        match self.defined.get(&addr) {
            Some(policy) => *policy,
            None => self.default,
        }
    }

    /// Set default policy
    ///
    pub fn default(&mut self, policy: Policy) -> &mut Self {
        self.default = policy; self
    }


    /// Add explicit allowance
    ///
    pub fn allow(&mut self, addr: IpAddr) -> &mut Self {
        self.defined.insert(addr,Policy::Allow); self
    }

    /// Add explicit denial
    ///
    pub fn deny(&mut self, addr: IpAddr) -> &mut Self {
        self.defined.insert(addr,Policy::Deny); self
    }

    /// Initialize a whitelist style filter
    ///
    pub fn whitelist(addrs: impl IntoIterator<Item=IpAddr>) -> Self {
        let defined = addrs.into_iter()
            .map(|addr| (addr,Policy::Allow))
            .collect();
        let default = Policy::Deny;
        Self { defined, default }
    }

    /// Initialize a blacklist style filter
    ///
    pub fn blacklist(addrs: impl IntoIterator<Item=IpAddr>) -> Self {
        let defined = addrs.into_iter()
            .map(|addr| (addr,Policy::Deny))
            .collect();
        let default = Policy::Allow;
        Self { defined, default }
    }
}


