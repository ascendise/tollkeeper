pub mod hashcash;

use std::{collections::HashMap, error::Error, fmt::Display};

/// Creates and verifies [tolls](Toll)
pub trait Declaration {
    fn declare(&self, suspect: Suspect, order_id: OrderIdentifier) -> Toll;
    fn pay(&mut self, payment: Payment, suspect: &Suspect) -> Result<Visa, InvalidPaymentError>;
}

/// Information about the source trying to access the resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suspect {
    client_ip: String,
    user_agent: String,
    destination: Destination,
}

impl Suspect {
    pub fn new(
        client_ip: impl Into<String>,
        user_agent: impl Into<String>,
        destination: Destination,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            user_agent: user_agent.into(),
            destination,
        }
    }

    pub fn client_ip(&self) -> &str {
        &self.client_ip
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn destination(&self) -> &Destination {
        &self.destination
    }

    /// Full 'name' of suspect
    pub fn identifier(&self) -> String {
        format!("({})[{}]", self.user_agent, self.client_ip)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Destination {
    base_url: String,
    port: u16,
    path: String,
}

impl Destination {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            port: 80,
            path: String::from("/"),
        }
    }

    pub fn new_with_details(
        base_url: impl Into<String>,
        port: u16,
        path: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            port,
            path: path.into(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

/// A Proof-of-Work challenge to be solved before being granted access
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Toll {
    recipient: Suspect,
    order_id: OrderIdentifier,
    challenge: HashMap<String, String>,
}

impl Toll {
    pub fn new(
        recipient: Suspect,
        order_id: OrderIdentifier,
        challenge: HashMap<String, String>,
    ) -> Self {
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
    pub fn challenge(&self) -> &HashMap<String, String> {
        &self.challenge
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

    pub fn order_id(&self) -> &OrderIdentifier {
        &self.toll.order_id
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Represents an access token for an [Order]
#[derive(Debug, PartialEq, Eq)]
pub struct Visa {
    order_id: OrderIdentifier,
    suspect: Suspect,
}

impl Visa {
    pub fn new(order_id: OrderIdentifier, suspect: Suspect) -> Self {
        Self { order_id, suspect }
    }

    /// [Order] the [Visa] was issued for
    pub fn order_id(&self) -> &OrderIdentifier {
        &self.order_id
    }

    /// [Suspect] the [Visa] was issued for
    pub fn suspect(&self) -> &Suspect {
        &self.suspect
    }
}

/// Return this error when [Payment::value()] is invalid
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidPaymentError {
    payment: Box<Payment>,
    new_toll: Box<Toll>,
}

impl InvalidPaymentError {
    pub fn new(payment: Box<Payment>, new_toll: Box<Toll>) -> Self {
        Self { payment, new_toll }
    }

    pub fn payment(&self) -> &Payment {
        &self.payment
    }

    pub fn new_toll(&self) -> &Toll {
        &self.new_toll
    }
}

impl Error for InvalidPaymentError {}
impl Display for InvalidPaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Value '{}' does not match criteria! A new toll was issued",
            self.payment.value()
        )
    }
}
