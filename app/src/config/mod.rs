use indexmap::IndexMap;
use serde::Deserialize;

#[cfg(test)]
mod tests;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Config {
    server: Server,
    gates: IndexMap<String, Gate>,
    orders: IndexMap<String, Order>,
}

impl Config {
    pub fn new(
        server: Server,
        gates: IndexMap<String, Gate>,
        orders: IndexMap<String, Order>,
    ) -> Self {
        Self {
            server,
            gates,
            orders,
        }
    }

    pub fn server(&self) -> &Server {
        &self.server
    }

    pub fn gates(&self) -> &IndexMap<String, Gate> {
        &self.gates
    }

    pub fn orders(&self) -> &IndexMap<String, Order> {
        &self.orders
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
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

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Gate {
    destination: url::Url,
    orders: Vec<Ref<Order>>,
}

impl Gate {
    pub fn new(destination: url::Url, orders: Vec<Ref<Order>>) -> Self {
        Self {
            destination,
            orders,
        }
    }

    pub fn destination(&self) -> &str {
        self.destination.as_ref()
    }

    pub fn orders(&self) -> &[Ref<Order>] {
        &self.orders
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Ref<T> {
    Value(T),
    Id(String),
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Order {
    descriptions: Vec<Ref<Description>>,
}

impl Order {
    pub fn new(descriptions: Vec<Ref<Description>>) -> Self {
        Self { descriptions }
    }

    pub fn descriptions(&self) -> &[Ref<Description>] {
        &self.descriptions
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Description {
    kind: String,
    config: IndexMap<String, String>,
}

impl Description {
    pub fn new(kind: String, config: IndexMap<String, String>) -> Self {
        Self { kind, config }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn config(&self) -> &IndexMap<String, String> {
        &self.config
    }
}
