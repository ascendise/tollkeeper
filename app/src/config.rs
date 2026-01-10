use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub struct Config {
    server: Server,
    gates: Vec<Gate>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Server {
    base_api_url: url::Url,
}
impl Server {
    pub fn new(base_api_url: url::Url) -> Self {
        Self { base_api_url }
    }
    pub fn base_api_url(&self) -> &url::Url {
        &self.base_api_url
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Gate {
    id: String,
    destination: url::Url,
    orders: Vec<Ref<Order>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Ref<T> {
    Value(T),
    Reference(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Order {
    id: String,
    descriptions: Vec<Ref<Description>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Description {
    id: String,
    kind: String,
    config: HashMap<String, String>,
}
