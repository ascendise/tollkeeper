pub mod err;

#[cfg(test)]
mod tests;

use std::{collections::HashMap, error::Error, fmt::Display};

use err::*;
use uuid::Uuid;

/// Guards actions against spam by requiring a PoW [challenge](Toll) to be solved before proceeding.
pub trait Tollkeeper {
    /// Checks if [Suspect] [matches description](Description::matches) and has to [pay a toll](Toll) before proceeding with it's
    /// action.
    ///
    /// Returns [Option::None] and calls ```on_access``` if [Suspect] is permitted, or [Toll]
    /// to be paid before being able to try again.
    fn guarded_access<T>(
        &self,
        suspect: &Suspect,
        visa: &Option<Visa>,
        request: &mut T,
        on_access: impl Fn(&mut T),
    ) -> Option<Toll>;

    /// Pay the [Toll] for a [Gate] [Order]. Changing priorities in orders may require you to get a
    /// new [Visa], if the new [Order] is higher ordered than the [Order] the [Visa] was bought for
    ///
    /// Returns new [Toll] if [Payment] is invalid
    /// Returns a [GatewayError] if there was a problem processing the [Payment]
    fn buy_visa(
        &self,
        suspect: &Suspect,
        payment: &Payment,
    ) -> Result<Result<Visa, Toll>, GatewayError>;
}

/// Default implementation of the [Tollkeeper].
pub struct TollkeeperImpl {
    gates: Vec<Gate>,
}

impl TollkeeperImpl {
    pub fn new(gates: Vec<Gate>) -> Result<Self, ConfigError> {
        if gates.is_empty() {
            Result::Err(ConfigError::new(
                String::from("gates"),
                String::from("No gates defined. Tollkeeper has nothing to protect!"),
            ))
        } else {
            Result::Ok(Self { gates })
        }
    }

    fn find_gate(&self, suspect: &Suspect) -> Option<&Gate> {
        self.gates
            .iter()
            .find(|g| g.destination == suspect.destination)
            .or(Option::None)
    }
}

/// Sends [Suspect] through matching [Gate] and  requests a [Toll] if necessary
impl Tollkeeper for TollkeeperImpl {
    fn guarded_access<T>(
        &self,
        suspect: &Suspect,
        visa: &Option<Visa>,
        request: &mut T,
        on_access: impl Fn(&mut T),
    ) -> Option<Toll> {
        let gate = match self.find_gate(suspect) {
            Option::Some(g) => g,
            Option::None => return Option::None, //TODO: Communicate somehow that no gate was found!
        };
        let result = gate.pass(suspect, visa);
        match result {
            Option::Some(g) => Option::Some(g),
            Option::None => {
                on_access(request);
                Option::None
            }
        }
    }

    fn buy_visa(
        &self,
        suspect: &Suspect,
        payment: &Payment,
    ) -> Result<Result<Visa, Toll>, GatewayError> {
        let gate = self
            .gates
            .iter()
            .find(|g| g.id == payment.order_id().gate_id)
            .ok_or(MissingGateError::new(&payment.order_id().gate_id))?;
        let order = gate
            .orders
            .iter()
            .find(|o| o.id == payment.order_id().order_id)
            .ok_or(MissingOrderError::new(
                &payment.order_id().gate_id,
                &payment.order_id().order_id,
            ))?;
        if suspect != &payment.toll.recipient {
            Result::Ok(Result::Err(order.toll_declaration.declare(
                suspect.clone(),
                OrderIdentifier::new(&gate.id, &order.id),
            )))
        } else {
            Result::Ok(order.toll_declaration.pay(payment, suspect))
        }
    }
}

/// Defines the target machine and which [suspects](Suspect) are allowed or not
pub struct Gate {
    id: String,
    destination: Destination,
    orders: Vec<Order>,
}

impl Gate {
    pub fn new(destination: Destination, orders: Vec<Order>) -> Result<Self, ConfigError> {
        if orders.is_empty() {
            Result::Err(ConfigError::new(
                "orders",
                "You need to define at least one order for the gate!",
            ))
        } else {
            Result::Ok(Self {
                id: Uuid::new_v4().to_string(),
                destination,
                orders,
            })
        }
    }

    /// Target machine destination
    pub fn destination(&self) -> &Destination {
        &self.destination
    }

    /// Defines which [suspects](Suspect) to look out for and how to proceed with them. Priority is
    /// based on order, meaning the first [Order] that explicitly [grants](AccessPolicy::Whitelist) or [denies](AccessPolicy::Blacklist) access will be
    /// executed.
    pub fn orders(&self) -> &Vec<Order> {
        &self.orders
    }

    /// Examine [Suspect] and check if it has to pay a [Toll]
    fn pass(&self, suspect: &Suspect, visa: &Option<Visa>) -> Option<Toll> {
        for order in &self.orders {
            let exam = order.examine(suspect, visa, &self.id);
            if exam.access_granted {
                return Option::None;
            }
            if exam.toll.is_some() {
                return exam.toll;
            }
        }
        Option::None
    }
}

/// Defines if [gates](Gate) suspects are allowed or denied on matching [Description]
/// [AccessPolicy::Whitelist]
#[derive(Debug, PartialEq, Eq)]
pub enum AccessPolicy {
    Whitelist,
    Blacklist,
}

/// Defines conditional process for a [Gate]
pub struct Order {
    id: String,
    descriptions: Vec<Box<dyn Description>>,
    access_policy: AccessPolicy,
    toll_declaration: Box<dyn Declaration>,
}

impl Order {
    pub fn new(
        descriptions: Vec<Box<dyn Description>>,
        access_policy: AccessPolicy,
        toll_declaration: Box<dyn Declaration>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            descriptions,
            access_policy,
            toll_declaration,
        }
    }

    fn examine(&self, suspect: &Suspect, visa: &Option<Visa>, gate_id: &str) -> Examination {
        let matches_description = self.is_match(suspect);
        let require_toll = (matches_description && self.access_policy == AccessPolicy::Blacklist)
            || (!matches_description && self.access_policy == AccessPolicy::Whitelist);
        let toll = if require_toll && !self.has_valid_visa(suspect, visa) {
            Option::Some(self.toll_declaration.declare(
                suspect.clone(),
                OrderIdentifier::new(gate_id, self.id.clone()),
            ))
        } else {
            Option::None
        };
        let access_granted = toll.is_none() && matches_description;
        Examination::new(toll, access_granted)
    }

    fn is_match(&self, suspect: &Suspect) -> bool {
        self.descriptions.iter().any(|d| d.matches(suspect))
    }

    fn has_valid_visa(&self, suspect: &Suspect, visa: &Option<Visa>) -> bool {
        match visa {
            Option::Some(v) => v.order_id().order_id == self.id && &v.suspect == suspect,
            Option::None => false,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Examines [Suspect] for a defined condition like matching IP/User-Agent/...
pub trait Description {
    fn matches(&self, suspect: &Suspect) -> bool;
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
}

struct Examination {
    toll: Option<Toll>,
    access_granted: bool,
}

impl Examination {
    fn new(toll: Option<Toll>, access_granted: bool) -> Self {
        Self {
            toll,
            access_granted,
        }
    }
}

/// Creates and verifies [tolls](Toll)
pub trait Declaration {
    fn declare(&self, suspect: Suspect, order_id: OrderIdentifier) -> Toll;
    fn pay(&self, payment: &Payment, suspect: &Suspect) -> Result<Visa, Toll>;
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
