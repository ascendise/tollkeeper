use std::{collections::HashMap, fmt::Display};

use crate::signatures::AsBytes;

pub mod regex;

/// Examines [Suspect] for a defined condition like matching IP/User-Agent/...
pub trait Description {
    fn matches(&self, suspect: &Suspect) -> bool;
}

/// Information about the source trying to access the resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suspect {
    client_ip: String,
    user_agent: String,
    destination: Destination,
}
impl Suspect {
    pub fn new(
        client_ip: impl Into<String>,
        user_agent: impl Into<String>,
        destination: Destination,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            user_agent: user_agent.into(),
            destination,
        }
    }

    pub fn client_ip(&self) -> &str {
        &self.client_ip
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn destination(&self) -> &Destination {
        &self.destination
    }

    /// Full 'name' of suspect
    pub fn identifier(&self) -> String {
        format!("({})[{}]", self.user_agent, self.client_ip)
    }
}
impl From<&Suspect> for HashMap<String, String> {
    fn from(val: &Suspect) -> Self {
        let mut map = HashMap::new();
        map.insert("user_agent".into(), val.user_agent.clone());
        map.insert("client_ip".into(), val.client_ip.clone());
        map.insert("destination".into(), val.destination.to_string());
        map
    }
}
impl AsBytes for Suspect {
    fn as_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.append(&mut AsBytes::as_bytes(&self.client_ip));
        data.append(&mut AsBytes::as_bytes(&self.user_agent));
        data.append(&mut self.destination().as_bytes());
        data
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Destination {
    base_url: String,
    port: u16,
    path: String,
}
impl Destination {
    pub fn new_base(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            port: 80,
            path: String::from("/"),
        }
    }

    pub fn new(base_url: impl Into<String>, port: u16, path: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            port,
            path: path.into(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}{}", self.base_url, self.port, self.path)
    }
}
impl AsBytes for Destination {
    fn as_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.append(&mut AsBytes::as_bytes(&self.base_url));
        data.append(&mut AsBytes::as_bytes(&self.port.to_be_bytes()));
        data.append(&mut AsBytes::as_bytes(&self.path));
        data
    }
}
