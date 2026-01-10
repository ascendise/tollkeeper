use indexmap::IndexMap;
use pretty_assertions::assert_eq;

use crate::config::{Config, Description, Gate, Order, Ref, Server};

#[test]
pub fn config_should_be_deserializable_from_toml() {
    // Arrange
    let toml = r#"
[server]
base_api_url = "http://localhost:9100/"
    
[gates]

[gates.json_api_gate]
destination = "wtfismyip.com:80/json"
orders = ["hash_cash_order"]

[gates.local_proxy_gate]
destination = "localhost:80/"
orders = ["hash_cash_order"]

[gates.ext_proxy_gate]
destination = "example.com:80/"
orders = ["hash_cash_order"]

[orders.hash_cash_order]
descriptions = [{kind = "Stub", config = {is_match = "true"}}]
access_policy = "Blacklist"
toll_declaration = "Hashcash"
"#;
    // Act
    let config: Config = toml::from_str(toml).unwrap();
    // Assert
    let expected_server_config = Server::new(url("http://localhost:9100/"));
    let mut expected_gates = IndexMap::new();
    expected_gates.insert(
        "ext_proxy_gate".into(),
        Gate::new(
            url("example.com:80/"),
            vec![Ref::Id("hash_cash_order".into())],
        ),
    );
    expected_gates.insert(
        "json_api_gate".to_string(),
        Gate::new(
            url("wtfismyip.com:80/json"),
            vec![Ref::Id("hash_cash_order".into())],
        ),
    );
    expected_gates.insert(
        "local_proxy_gate".into(),
        Gate::new(
            url("localhost:80/"),
            vec![Ref::Id("hash_cash_order".into())],
        ),
    );
    let mut desc_config = IndexMap::new();
    desc_config.insert("is_match".to_string(), "true".to_string());
    let expected_description = Description::new("Stub".into(), desc_config);
    let mut expected_orders = IndexMap::new();
    expected_orders.insert(
        "hash_cash_order".to_string(),
        Order::new(vec![Ref::Value(expected_description)]),
    );
    let expected_config = Config::new(expected_server_config, expected_gates, expected_orders);
    assert_eq!(expected_config, config);
}

fn url(s: &str) -> url::Url {
    url::Url::parse(s).unwrap()
}
