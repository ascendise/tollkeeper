pub mod hashcash;

use std::{collections::HashMap, error::Error, fmt::Display};

use crate::{
    err::InvalidPaymentError,
    signatures::{AsBytes, Signed},
};

use super::descriptions::Suspect;

/// Creates and verifies [tolls](Toll)
pub trait Declaration {
    fn declare(&self, suspect: Suspect, order_id: OrderIdentifier) -> Toll;
    fn pay(&mut self, payment: Payment, suspect: &Suspect) -> Result<Visa, PaymentError>;
}

pub type Challenge = HashMap<String, String>;

/// A Proof-of-Work challenge to be solved before being granted access
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Toll {
    recipient: Suspect,
    order_id: OrderIdentifier,
    challenge: Challenge,
}

impl Toll {
    pub fn new(recipient: Suspect, order_id: OrderIdentifier, challenge: Challenge) -> Self {
        Self {
            recipient,
            order_id,
            challenge,
        }
    }

    /// Who has to pay the toll
    pub fn recipient(&self) -> &Suspect {
        &self.recipient
    }

    /// Order the toll has to be paid for
    pub fn order_id(&self) -> &OrderIdentifier {
        &self.order_id
    }

    /// All values required to solve the challenge, like seed values, algorithms, etc.
    pub fn challenge(&self) -> &Challenge {
        &self.challenge
    }
}
impl AsBytes for Toll {
    fn as_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.append(&mut self.recipient.as_bytes());
        data.append(&mut self.order_id.as_bytes());
        for kv in &self.challenge {
            data.append(&mut AsBytes::as_bytes(kv.0));
            data.append(&mut AsBytes::as_bytes(kv.1));
        }
        data
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct OrderIdentifier {
    gate_id: String,
    order_id: String,
}
impl OrderIdentifier {
    pub fn new(gate_id: impl Into<String>, order_id: impl Into<String>) -> Self {
        Self {
            gate_id: gate_id.into(),
            order_id: order_id.into(),
        }
    }

    pub fn gate_id(&self) -> &str {
        &self.gate_id
    }

    pub fn order_id(&self) -> &str {
        &self.order_id
    }
}
impl AsBytes for OrderIdentifier {
    fn as_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.append(&mut AsBytes::as_bytes(&self.gate_id));
        data.append(&mut AsBytes::as_bytes(&self.order_id));
        data
    }
}

/// Solution for solved [challenge](Toll)
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Payment {
    toll: Toll,
    value: String,
}

impl Payment {
    /// Creates a payment containing the [challenge][Toll] to be solved and the calculated hash
    pub fn new(toll: Toll, value: impl Into<String>) -> Self {
        Self {
            toll,
            value: value.into(),
        }
    }

    pub fn toll(&self) -> &Toll {
        &self.toll
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Represents an access token for an [super::Order]
#[derive(Debug, PartialEq, Eq)]
pub struct Visa {
    order_id: OrderIdentifier,
    suspect: Suspect,
}
impl Visa {
    pub fn new(order_id: OrderIdentifier, suspect: Suspect) -> Self {
        Self { order_id, suspect }
    }

    /// [super::Order] the [Visa] was issued for
    pub fn order_id(&self) -> &OrderIdentifier {
        &self.order_id
    }

    /// [Suspect] the [Visa] was issued for
    pub fn suspect(&self) -> &Suspect {
        &self.suspect
    }
}
impl AsBytes for Visa {
    fn as_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.append(&mut self.order_id.as_bytes());
        data.append(&mut self.suspect.as_bytes());
        data
    }
}

/// Return this error when [Payment::value()] is invalid
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentError {
    payment: Box<Payment>,
    new_toll: Box<Toll>,
}

impl PaymentError {
    pub fn new(payment: Box<Payment>, new_toll: Box<Toll>) -> Self {
        Self { payment, new_toll }
    }

    pub fn payment(&self) -> &Payment {
        &self.payment
    }

    pub fn new_toll(&self) -> &Toll {
        &self.new_toll
    }

    pub fn into(self, secret_key: &[u8]) -> InvalidPaymentError {
        let toll = Signed::sign(*self.new_toll, secret_key);
        InvalidPaymentError::new(self.payment, Box::new(toll))
    }
}

impl Error for PaymentError {}
impl Display for PaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Value '{}' does not match criteria! A new toll was issued",
            self.payment.value()
        )
    }
}
