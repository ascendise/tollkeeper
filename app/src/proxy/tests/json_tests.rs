use serde_json::json;
use tollkeeper::declarations::Challenge;

use crate::{
    data_formats::AsHalJson,
    proxy::{OrderId, Recipient, Toll},
};

#[test]
pub fn serializing_toll_should_return_expected_json() {
    // Arrange
    let mut challenge = Challenge::new();
    challenge.insert(
        "question".into(),
        "Why does the chicken cross the road?".into(),
    );
    let toll = Toll {
        recipient: Recipient {
            client_ip: "1.2.3.4".into(),
            user_agent: "Netscape".into(),
            destination: "http://example.com/bot-secured-endpoint".into(),
        },
        order_id: OrderId {
            gate_id: "gate".into(),
            order_id: "order".into(),
        },
        challenge,
        signature: "do-not-edit".into(),
    };
    // Act
    let base_url = url::Url::parse("http://tollkeeper.com").unwrap();
    let toll_json = toll.as_hal_json(&base_url);
    // Assert
    let expected_json = json!({
        "toll": {
            "recipient": {
                "client_ip": "1.2.3.4",
                "user_agent": "Netscape",
                "destination": "http://example.com/bot-secured-endpoint",
            },
            "order_id": "gate#order",
            "challenge": {
                "question": "Why does the chicken cross the road?"
            },
            "signature": "do-not-edit",
        },
        "_links": {
            "pay": "http://tollkeeper.com/api/pay/"
        }
    });
    assert_eq!(expected_json, toll_json);
}
