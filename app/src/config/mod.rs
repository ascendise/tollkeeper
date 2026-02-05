use indexmap::IndexMap;
use serde::Deserialize;
use tollkeeper::signatures::InMemorySecretKeyProvider;

use crate::proxy;

#[cfg(test)]
mod tests;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub server: Option<Server>,
    pub api: Api,
    secret_key_provider: SecretKeyProvider,
    gates: IndexMap<String, Gate>,
    orders: Option<IndexMap<String, Order>>,
    descriptions: Option<IndexMap<String, Description>>,
}

impl Config {
    pub fn from_toml(toml: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml)
    }

    pub fn server(&self) -> Server {
        match &self.server {
            Some(s) => s.clone(),
            None => Server::default(),
        }
    }

    pub fn create_tollkeeper(&self) -> Option<tollkeeper::Tollkeeper> {
        let gates = self
            .gates
            .iter()
            .map(|(id, gate)| {
                gate.to_entity(
                    id.to_string(),
                    self.orders.as_ref().unwrap_or(&IndexMap::new()),
                    self.descriptions.as_ref().unwrap_or(&IndexMap::new()),
                )
                .unwrap()
            })
            .collect();
        let secret_key_provider = self.secret_key_provider.to_entity();
        let date_provider = Box::new(tollkeeper::util::DateTimeProviderImpl);
        let tollkeeper =
            tollkeeper::Tollkeeper::new(gates, secret_key_provider, date_provider).ok()?;
        Some(tollkeeper)
    }

    pub fn create_url_resolver(&self) -> proxy::UrlResolverImpl {
        let mappings: indexmap::IndexMap<url::Url, url::Url> = self
            .gates
            .iter()
            .map(|(_, g)| {
                let mut public_host = g.destination.clone();
                public_host.set_path("");
                let mut internal_host = g
                    .internal_destination
                    .clone()
                    .unwrap_or(public_host.clone());
                internal_host.set_path("");
                (public_host, internal_host)
            })
            .collect();
        proxy::UrlResolverImpl::new(mappings)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Server {
    pub proxy_port: Option<usize>,
    pub api_port: Option<usize>,
}
impl Server {
    pub const PROXY_PORT_DEFAULT: usize = 8000;
    pub const API_PORT_DEFAULT: usize = 8080;

    pub fn proxy_port(&self) -> usize {
        self.proxy_port.unwrap_or(Self::PROXY_PORT_DEFAULT)
    }

    pub fn api_port(&self) -> usize {
        self.api_port.unwrap_or(Self::API_PORT_DEFAULT)
    }
}
impl Default for Server {
    fn default() -> Self {
        Self {
            proxy_port: Some(Self::PROXY_PORT_DEFAULT),
            api_port: Some(Self::API_PORT_DEFAULT),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Api {
    pub base_url: url::Url,
    pub real_ip_header: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
enum SecretKeyProvider {
    InMemory(String),
}
impl SecretKeyProvider {
    fn to_entity(&self) -> Box<dyn tollkeeper::signatures::SecretKeyProvider + Send + Sync> {
        let provider = match self {
            SecretKeyProvider::InMemory(key) => {
                InMemorySecretKeyProvider::new(key.clone().into_bytes())
            }
        };
        Box::new(provider)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
struct Gate {
    destination: url::Url,
    internal_destination: Option<url::Url>,
    orders: Vec<Ref<Order>>,
}

impl Gate {
    fn to_entity(
        &self,
        id: String,
        orders: &IndexMap<String, Order>,
        descriptions: &IndexMap<String, Description>,
    ) -> Option<tollkeeper::Gate> {
        let orders: Vec<tollkeeper::Order> = self
            .orders
            .iter()
            .map(|o| -> Option<tollkeeper::Order> {
                o.read_value(orders)?
                    .to_entity(o.id().map(|s| s.to_string()), descriptions)
            })
            .map(|o| o.unwrap())
            .collect();
        let destination = tollkeeper::descriptions::Destination::new(
            self.destination.host().unwrap().to_string(),
            self.destination.port().unwrap_or(80),
            self.destination.path(),
        );
        let gate = tollkeeper::Gate::with_id(id, destination, orders).unwrap();
        Some(gate)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
enum Ref<T> {
    Value(T),
    Id(String),
}
impl<T> Ref<T>
where
    T: Clone,
{
    /// Reads the value from the ref by either returning the enclosed [Ref::Value]
    /// or reading the value from the provided entities.
    fn read_value(&self, entities: &IndexMap<String, T>) -> Option<T> {
        match self {
            Ref::Value(v) => Some(v.clone()),
            Ref::Id(id) => {
                let entity = entities.get(id)?.clone();
                Some(entity)
            }
        }
    }

    fn id(&self) -> Option<&String> {
        match self {
            Ref::Value(_) => None,
            Ref::Id(id) => Some(id),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
struct Order {
    descriptions: Vec<Ref<Description>>,
    access_policy: tollkeeper::AccessPolicy,
    toll_declaration: Declaration,
}

impl Order {
    fn to_entity(
        &self,
        id: Option<String>,
        descriptions: &IndexMap<String, Description>,
    ) -> Option<tollkeeper::Order> {
        let descriptions = self
            .descriptions
            .iter()
            .map(|d| d.read_value(descriptions)?.to_entity())
            .map(|o| o.unwrap())
            .collect();
        let order = match id {
            Some(id) => tollkeeper::Order::with_id(
                id,
                descriptions,
                self.access_policy,
                self.toll_declaration.to_entity(),
            ),
            None => tollkeeper::Order::new(
                descriptions,
                self.access_policy,
                self.toll_declaration.to_entity(),
            ),
        };
        Some(order)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
enum Description {
    Stub(StubDescription),
    Regex(RegexDescription),
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
struct StubDescription {
    is_match: bool,
}
impl Description {
    fn to_entity(&self) -> Option<Box<dyn tollkeeper::Description + Send + Sync>> {
        let description: Box<dyn tollkeeper::Description + Send + Sync> = match self {
            Description::Stub(cfg) => {
                let description = crate::StubDescription {
                    is_match: cfg.is_match,
                };
                Box::new(description)
            }
            Description::Regex(cfg) => {
                let key = cfg.key.clone();
                let regex = &cfg.regex;
                let negative_lookahead = cfg.negate.unwrap_or(false);
                let description = tollkeeper::descriptions::regex::RegexDescription::new(
                    key,
                    regex,
                    negative_lookahead,
                );
                Box::new(description.unwrap())
            }
        };
        Some(description)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
struct RegexDescription {
    key: String,
    regex: String,
    negate: Option<bool>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
enum Declaration {
    Hashcash(HashcashDeclaration),
}
impl Declaration {
    fn to_entity(&self) -> Box<dyn tollkeeper::Declaration + Send + Sync> {
        let declaration = match self {
            Declaration::Hashcash(hashcash) => hashcash.to_entity(),
        };
        Box::new(declaration)
    }
}
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
struct HashcashDeclaration {
    difficulty: u8,
    expiry: String,
    #[serde(default)]
    double_spent_db: DoubleSpentDatabase,
}
impl HashcashDeclaration {
    fn to_entity(&self) -> tollkeeper::declarations::hashcash::HashcashDeclaration {
        let date_provider = tollkeeper::util::DateTimeProviderImpl;
        let double_spent_db = self.double_spent_db.to_entity();
        tollkeeper::declarations::hashcash::HashcashDeclaration::new(
            self.difficulty,
            self.expiry(),
            Box::new(date_provider),
            Box::new(double_spent_db),
        )
    }

    fn expiry(&self) -> chrono::Duration {
        let end = self.expiry.len() - 1;
        let time = &self.expiry[0..end];
        let format = self.expiry.chars().last().unwrap();
        let time = time.parse::<i64>().unwrap();
        match format {
            's' => chrono::Duration::seconds(time),
            'm' => chrono::Duration::minutes(time),
            'h' => chrono::Duration::hours(time),
            'd' => chrono::Duration::days(time),
            _ => panic!("Unexpected time format: {format}"),
        }
    }
}
#[derive(Deserialize, Debug, Default, PartialEq, Eq, Clone)]
#[serde(default)]
struct DoubleSpentDatabase {
    stamp_limit: Option<usize>,
}
impl DoubleSpentDatabase {
    fn to_entity(&self) -> tollkeeper::declarations::hashcash::DoubleSpentDatabaseImpl {
        tollkeeper::declarations::hashcash::DoubleSpentDatabaseImpl::new(self.stamp_limit)
    }
}
