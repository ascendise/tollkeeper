use indexmap::IndexMap;
use pretty_assertions::assert_eq;
use tollkeeper::AccessPolicy;

use crate::config::{
    Api, Config, Declaration, Description, Gate, HashcashDeclaration, Order, Ref,
    SecretKeyProvider, Server, StubDescription,
};

#[test]
pub fn config_should_be_deserializable_from_toml() {
    // Arrange
    let toml = r#"
secret_key_provider = { InMemory = "verysecretkey" }

[server]
proxy_port = 9000
# api_port = use default 

[api]
base_url = "http://localhost:9100/"
    
[gates]

[gates.local_proxy_gate]
destination = "http://localhost/"
orders = ["hash_cash_order"]

[gates.ext_proxy_gate]
destination = "http://example.com/"
orders = ["hash_cash_order"]

[orders.hash_cash_order]
descriptions = [{Stub = {is_match = true}}]
access_policy = "Blacklist"
toll_declaration = { Hashcash = { expiry = "1h", difficulty = 4}}
"#;
    // Act
    let config: Config = toml::from_str(toml).unwrap();
    // Assert
    let api = Api {
        base_url: url("http://localhost:9100/"),
    };
    let mut gates = IndexMap::new();
    gates.insert(
        "ext_proxy_gate".into(),
        Gate {
            destination: url("http://example.com:80/"),
            orders: vec![Ref::Id("hash_cash_order".into())],
        },
    );
    gates.insert(
        "local_proxy_gate".into(),
        Gate {
            destination: url("http://localhost:80/"),
            orders: vec![Ref::Id("hash_cash_order".into())],
        },
    );
    let description = Description::Stub(StubDescription { is_match: true });
    let mut orders = IndexMap::new();
    orders.insert(
        "hash_cash_order".to_string(),
        Order {
            descriptions: vec![Ref::Value(description)],
            access_policy: AccessPolicy::Blacklist,
            toll_declaration: Declaration::Hashcash(HashcashDeclaration {
                difficulty: 4,
                expiry: "1h".into(),
            }),
        },
    );
    let server = Server {
        proxy_port: Some(9000),
        api_port: None,
    };
    let expected_config = Config {
        server: Some(server),
        api,
        secret_key_provider: SecretKeyProvider::InMemory("verysecretkey".into()),
        gates,
        orders: Some(orders),
        descriptions: None,
    };
    assert_eq!(expected_config, config);
}

fn url(s: &str) -> url::Url {
    url::Url::parse(s).unwrap()
}

#[test]
pub fn create_tollkeeper_should_create_a_new_tollkeeper_instance_with_given_config() {
    // Arrange
    let api = Api {
        base_url: url("http://localhost:9100/"),
    };

    let mut gates = IndexMap::new();
    gates.insert(
        "ext_proxy_gate".into(),
        Gate {
            destination: url("http://example.com:80/"),
            orders: vec![Ref::Id("hash_cash_order".into())],
        },
    );
    gates.insert(
        "local_proxy_gate".into(),
        Gate {
            destination: url("http://localhost:80/"),
            orders: vec![Ref::Id("hash_cash_order".into())],
        },
    );
    let description = Description::Stub(StubDescription { is_match: true });
    let mut orders = IndexMap::new();
    orders.insert(
        "hash_cash_order".to_string(),
        Order {
            descriptions: vec![Ref::Value(description)],
            access_policy: AccessPolicy::Blacklist,
            toll_declaration: Declaration::Hashcash(HashcashDeclaration {
                difficulty: 4,
                expiry: "10h".into(),
            }),
        },
    );
    let secret_key_provider = SecretKeyProvider::InMemory("verysecretkey".into());
    let server = Server {
        proxy_port: Some(9000),
        api_port: Some(9100),
    };
    let config = Config {
        server: Some(server),
        api,
        secret_key_provider,
        gates,
        orders: Some(orders),
        descriptions: None,
    };
    // Act
    let tollkeeper = config.create_tollkeeper();
    //Assert
    assert!(
        tollkeeper.is_some(),
        "Failed to create a Tollkeeper from given config!"
    );
}
