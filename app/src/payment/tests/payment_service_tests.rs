use std::{collections::HashMap, net};

use tollkeeper::{
    declarations::{Challenge, OrderIdentifier},
    descriptions,
    signatures::InMemorySecretKeyProvider,
    Declaration,
};

use crate::{
    payment::{Payment, PaymentService, PaymentServiceImpl},
    proxy::{OrderId, Recipient, Toll},
};

fn setup(password: String, recipient: Recipient) -> (Toll, PaymentServiceImpl) {
    let destination = descriptions::Destination::new("http://example.ascendise.ch", 80, "/");
    let declaration = FakeTollDeclaration::new(password);
    let orders = vec![tollkeeper::Order::new(
        vec![Box::new(StubDescription)],
        tollkeeper::AccessPolicy::Blacklist,
        Box::new(declaration.clone()),
    )];
    let order_id = orders[0].id().to_string();
    let gates = vec![tollkeeper::Gate::new(destination, orders).unwrap()];
    let gate_id = gates[0].id().to_string();
    let secret_key_provider = InMemorySecretKeyProvider::new("Secret key".into());
    let secret_key_provider = Box::new(secret_key_provider);
    let tollkeeper = tollkeeper::Tollkeeper::new(gates, secret_key_provider).unwrap();
    let toll = declaration.declare(recipient.into(), OrderIdentifier::new(gate_id, order_id));
    let toll = tollkeeper::signatures::Signed::sign(toll, b"Secret key");
    (toll.into(), PaymentServiceImpl::new(tollkeeper))
}

#[test]
pub fn pay_toll_should_return_visa_when_providing_correct_payment() {
    // Arrange
    let recipient = Recipient::new(
        "192.106.12.13",
        "UnitTest",
        "http://example.ascendise.ch/hello",
    );
    let (toll_to_pay, sut) = setup("secret".into(), recipient.clone());
    // Act
    let payment = Payment::new(toll_to_pay, "secret".into());
    let payment_result = sut.pay_toll(recipient, payment);
    // Assert
    assert!(payment_result.is_ok());
}

#[derive(Clone)]
struct FakeTollDeclaration {
    password: String,
}
impl FakeTollDeclaration {
    fn new(password: String) -> Self {
        Self { password }
    }
}
impl tollkeeper::Declaration for FakeTollDeclaration {
    fn declare(
        &self,
        suspect: descriptions::Suspect,
        order_id: tollkeeper::declarations::OrderIdentifier,
    ) -> tollkeeper::declarations::Toll {
        tollkeeper::declarations::Toll::new(suspect, order_id, Challenge::new())
    }

    fn pay(
        &mut self,
        payment: tollkeeper::declarations::Payment,
        suspect: &descriptions::Suspect,
    ) -> Result<tollkeeper::declarations::Visa, tollkeeper::declarations::PaymentError> {
        let order_id = payment.toll().order_id().clone();
        if payment.value() == self.password {
            let visa = tollkeeper::declarations::Visa::new(order_id.clone(), suspect.clone());
            Ok(visa)
        } else {
            let error = tollkeeper::declarations::PaymentError::new(
                Box::new(payment),
                Box::new(self.declare(suspect.clone(), order_id)),
            );
            Err(error)
        }
    }
}

struct StubDescription;
impl tollkeeper::Description for StubDescription {
    fn matches(&self, _: &descriptions::Suspect) -> bool {
        true
    }
}
