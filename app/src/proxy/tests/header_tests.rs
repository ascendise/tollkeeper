use pretty_assertions::assert_eq;
use tollkeeper::signatures::Base64;

use crate::{
    data_formats::{AsHttpHeader, FromHttpHeader},
    payment::Visa,
    proxy::{OrderId, Recipient},
};

#[test]
pub fn serializing_visa_should_return_x_keeper_token() {
    // Arrange
    let visa = Visa::new(
        OrderId {
            gate_id: "gate".into(),
            order_id: "order".into(),
        },
        Recipient {
            client_ip: "1.2.3.4".into(),
            user_agent: "Netscape".into(),
            destination: "http://example.com/".into(),
        },
        Base64::encode(&[1, 2, 3, 4, 5]),
    );
    // Act
    let (key, value) = visa.as_http_header();
    // Assert
    assert_eq!("X-Keeper-Token", key);
    assert_eq!("eyJpcCI6IjEuMi4zLjQiLCJ1YSI6Ik5ldHNjYXBlIiwiZGVzdCI6Imh0dHA6Ly9leGFtcGxlLmNvbS8iLCJvcmRlcl9pZCI6ImdhdGUjb3JkZXIifQ==.AQIDBAU=", value);
}

#[test]
pub fn deserializing_x_keeper_token_should_return_visa() {
    // Arrange
    let token = "eyJkZXN0IjoiaHR0cDovL2V4YW1wbGUuY29tLyIsImlwIjoiMS4yLjMuNCIsIm9yZGVyX2lkIjoiZ2F0ZSNvcmRlciIsInVhIjoiTmV0c2NhcGUifQ==.AQIDBAU=";
    // Act
    let visa = Visa::from_http_header(token);
    // Assert
    let expected = Visa::new(
        OrderId {
            gate_id: "gate".into(),
            order_id: "order".into(),
        },
        Recipient {
            client_ip: "1.2.3.4".into(),
            user_agent: "Netscape".into(),
            destination: "http://example.com/".into(),
        },
        Base64::encode(&[1, 2, 3, 4, 5]),
    );
    assert_eq!(Ok(expected), visa);
}
