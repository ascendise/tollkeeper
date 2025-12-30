use std::sync::Arc;

use tollkeeper::{
    signatures::{InMemorySecretKeyProvider, Signed},
    Declaration,
};

use crate::{
    payment::{Payment, PaymentError, PaymentService, PaymentServiceImpl},
    proxy::{Recipient, Toll},
};

fn setup(password: String, recipient: Recipient) -> (Toll, PaymentServiceImpl) {
    let destination =
        tollkeeper::descriptions::Destination::new("http://example.ascendise.ch", 80, "/");
    let declaration = FakeTollDeclaration::new(password);
    let orders = vec![tollkeeper::Order::with_id(
        "order",
        vec![Box::new(StubDescription)],
        tollkeeper::AccessPolicy::Blacklist,
        Box::new(declaration.clone()),
    )];
    let gates = vec![tollkeeper::Gate::with_id("gate", destination, orders).unwrap()];
    let secret_key_provider = InMemorySecretKeyProvider::new(b"Secret key".into());
    let secret_key_provider = Box::new(secret_key_provider);
    let tollkeeper = tollkeeper::Tollkeeper::new(gates, secret_key_provider).unwrap();
    let toll = declaration.declare(
        recipient.into(),
        tollkeeper::declarations::OrderIdentifier::new("gate", "order"),
    );
    let toll = Signed::sign(toll, b"Secret key");
    (toll.into(), PaymentServiceImpl::new(Arc::new(tollkeeper)))
}

fn setup_unsigned_toll(
    password: String,
    recipient: Recipient,
) -> (tollkeeper::declarations::Toll, PaymentServiceImpl) {
    let destination =
        tollkeeper::descriptions::Destination::new("http://example.ascendise.ch", 80, "/");
    let declaration = FakeTollDeclaration::new(password);
    let orders = vec![tollkeeper::Order::new(
        vec![Box::new(StubDescription)],
        tollkeeper::AccessPolicy::Blacklist,
        Box::new(declaration.clone()),
    )];
    let order_id = orders[0].id().to_string();
    let gates = vec![tollkeeper::Gate::new(destination, orders).unwrap()];
    let gate_id = gates[0].id().to_string();
    let secret_key_provider = InMemorySecretKeyProvider::new(b"Secret key".into());
    let secret_key_provider = Box::new(secret_key_provider);
    let tollkeeper = tollkeeper::Tollkeeper::new(gates, secret_key_provider).unwrap();
    let toll = declaration.declare(
        recipient.into(),
        tollkeeper::declarations::OrderIdentifier::new(gate_id, order_id),
    );
    (toll, PaymentServiceImpl::new(Arc::new(tollkeeper)))
}

#[test]
pub fn pay_toll_should_return_visa_when_providing_correct_payment() {
    // Arrange
    let recipient = Recipient::new("192.106.12.13", "UnitTest", "example.ascendise.ch:80/hello");
    let (toll_to_pay, sut) = setup("secret".into(), recipient.clone());
    // Act
    let payment = Payment::new(toll_to_pay, "secret".into());
    let payment_result = sut.pay_toll(recipient, payment);
    // Assert
    assert!(payment_result.is_ok(), "Valid payment rejected!");
    let visa: tollkeeper::signatures::Signed<tollkeeper::declarations::Visa> =
        payment_result.unwrap().into();

    assert!(
        visa.verify(b"Secret key").is_ok(),
        "Visa has invalid signature!"
    );
}

#[test]
pub fn pay_toll_should_return_error_for_wrong_payment() {
    // Arrange
    let recipient = Recipient::new("192.106.12.13", "UnitTest", "example.ascendise.ch:80/hello");
    let (toll_to_pay, sut) = setup("secret".into(), recipient.clone());
    // Act
    let payment = Payment::new(toll_to_pay.clone(), "not-the-secret".into());
    let payment_result = sut.pay_toll(recipient, payment);
    // Assert
    let expected_err = PaymentError::ChallengeFailed(toll_to_pay, "not-the-secret".into());
    let expected_err = Box::new(expected_err);
    assert_eq!(Err(expected_err), payment_result);
}

#[test]
pub fn pay_toll_should_return_error_for_mismatched_recipient() {
    // Arrange
    let toll_recipient =
        Recipient::new("192.106.12.13", "UnitTest", "example.ascendise.ch:80/hello");
    let (toll_to_pay, sut) = setup("secret".into(), toll_recipient.clone());
    // Act
    let payment = Payment::new(toll_to_pay.clone(), "not-the-secret".into());
    let different_recipient =
        Recipient::new("85.120.13.37", "UnitTest", "example.ascendise.ch/hello");
    let payment_result = sut.pay_toll(different_recipient.clone(), payment);
    // Assert
    let mut challenge = tollkeeper::declarations::Challenge::new();
    challenge.insert("hello".into(), "world".into());
    let expected_new_toll = tollkeeper::declarations::Toll::new(
        different_recipient.clone().into(),
        tollkeeper::declarations::OrderIdentifier::new("gate", "order"),
        challenge,
    );
    let expected_new_toll = Signed::sign(expected_new_toll, b"Secret key");
    let expected_new_toll: Toll = expected_new_toll.into();
    let expected_err = PaymentError::MismatchedRecipient(toll_recipient, expected_new_toll);
    let expected_err = Box::new(expected_err);
    assert_eq!(Err(expected_err), payment_result);
}

#[test]
pub fn pay_toll_should_return_error_for_forged_payment() {
    // Arrange
    let recipient = Recipient::new("192.106.12.13", "UnitTest", "example.ascendise.ch:80/hello");
    let (unsigned_toll, sut) = setup_unsigned_toll("secret".into(), recipient.clone());
    let forged_toll = Signed::sign(unsigned_toll, b"forged-key");
    let forged_toll: Toll = forged_toll.into();
    // Act
    let payment = Payment::new(forged_toll.clone(), "not-the-secret".into());
    let payment_result = sut.pay_toll(recipient, payment);
    // Assert
    let expected_err = PaymentError::InvalidSignature;
    let expected_err = Box::new(expected_err);
    assert_eq!(Err(expected_err), payment_result);
}

#[test]
pub fn pay_toll_should_return_error_for_unknown_order_id() {
    // Arrange
    let recipient = Recipient::new("192.106.12.13", "UnitTest", "example.ascendise.ch:80/hello");
    let (_, sut) = setup("secret".into(), recipient.clone());
    let toll_to_pay = tollkeeper::declarations::Toll::new(
        recipient.clone().into(),
        tollkeeper::declarations::OrderIdentifier::new("?", "!"), // Simulating an old toll with stale order id
        tollkeeper::declarations::Challenge::new(),
    );
    let toll_to_pay = Signed::sign(toll_to_pay, b"Secret key");
    let toll_to_pay: Toll = toll_to_pay.into();
    // Act
    let payment = Payment::new(toll_to_pay.clone(), "not-the-secret".into());
    let payment_result = sut.pay_toll(recipient.clone(), payment);
    // Assert
    let expected_err = PaymentError::GatewayError;
    let expected_err = Box::new(expected_err);
    assert_eq!(Err(expected_err), payment_result);
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
        suspect: tollkeeper::descriptions::Suspect,
        order_id: tollkeeper::declarations::OrderIdentifier,
    ) -> tollkeeper::declarations::Toll {
        let mut challenge = tollkeeper::declarations::Challenge::new();
        challenge.insert("hello".into(), "world".into());
        tollkeeper::declarations::Toll::new(suspect, order_id, challenge)
    }

    fn pay(
        &self,
        payment: tollkeeper::declarations::Payment,
        suspect: &tollkeeper::descriptions::Suspect,
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
    fn matches(&self, _: &tollkeeper::descriptions::Suspect) -> bool {
        true
    }
}
