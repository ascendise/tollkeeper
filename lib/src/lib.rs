pub use declarations::Declaration;
pub mod declarations;
pub use descriptions::Description;
pub mod descriptions;
pub mod err;
pub mod signatures;
pub mod util;

#[cfg(test)]
mod tests;

use std::{error::Error, fmt::Display};

use declarations::*;
use descriptions::*;
use err::*;
use signatures::Signed;
use uuid::Uuid;

use signatures::SecretKeyProvider;

/// Guards actions against spam by requiring a PoW [challenge](Toll) to be solved before proceeding.
pub struct Tollkeeper {
    gates: Vec<Gate>,
    secret_key_provider: Box<dyn SecretKeyProvider + Send + Sync>,
}

impl Tollkeeper {
    pub fn new(
        gates: Vec<Gate>,
        secret_key_provider: Box<dyn SecretKeyProvider + Send + Sync>,
    ) -> Result<Self, ConfigError> {
        if gates.is_empty() {
            Err(ConfigError::new(
                String::from("gates"),
                String::from("No gates defined. Tollkeeper has nothing to protect!"),
            ))
        } else {
            Ok(Self {
                gates,
                secret_key_provider,
            })
        }
    }

    fn find_matching_gate(&self, suspect: &Suspect) -> Result<&Gate, AccessError> {
        let access_destination = suspect.destination().clone();
        match self
            .gates
            .iter()
            .find(|g| g.destination().contains(&access_destination))
        {
            Some(g) => Ok(g),
            None => Err(AccessError::DestinationNotFound(Box::new(
                access_destination,
            ))),
        }
    }

    /// Checks if [Suspect] [matches description](Description::matches) and has to [pay a toll](Toll)
    /// before proceeding with it's action.
    ///
    /// Returns [Option::None] and calls ```on_access``` if [Suspect] is permitted, or [Toll]
    /// to be paid before being able to try again.
    pub fn check_access(
        &self,
        suspect: &Suspect,
        visa: &Option<Signed<Visa>>,
    ) -> Result<(), AccessError> {
        let _span = tracing::info_span!("[Tollkeeper(access_control)]").entered();
        let gate = self.find_matching_gate(suspect)?;
        let secret_key = self.secret_key_provider.read_secret_key();
        let result = gate.pass(suspect, visa, secret_key);
        match result {
            Some(toll) => {
                let secret_key = self.secret_key_provider.read_secret_key();
                let toll = Signed::sign(toll, secret_key);
                Err(AccessError::AccessDeniedError(Box::new(toll)))
            }
            None => Ok(()),
        }
    }

    /// Pay the [Toll] for a [Gate] [Order]. Changing priorities in orders may require you to get a
    /// new [Visa], if the new [Order] is higher ordered than the [Order] the [Visa] was bought for
    ///
    /// Returns new [Toll] if [Payment] is invalid
    /// Returns a [PaymentDeniedError] if there was a problem processing the [Payment]
    /// Returns a [PaymentDeniedError::GatewayError] if gate/order is unknown/removed
    pub fn pay_toll(
        &self,
        suspect: &Suspect,
        payment: SignedPayment,
    ) -> Result<Signed<Visa>, PaymentDeniedError> {
        let _span = tracing::info_span!("[Tollkeeper(payment)]").entered();
        let secret_key = self.secret_key_provider.read_secret_key();
        let payment = payment.verify(secret_key)?;
        let toll = payment.toll();
        let order_id = toll.order_id();
        let gate = Self::find_gate_by_id(&self.gates, order_id)?;
        let order = Self::find_order_by_id(&gate.orders, order_id)?;
        if suspect != toll.recipient() {
            let new_toll = order
                .toll_declaration
                .declare(suspect.clone(), OrderIdentifier::new(&gate.id, &order.id));
            let new_toll = Signed::sign(new_toll, secret_key);
            let error =
                MismatchedSuspectError::new(Box::new(toll.recipient().clone()), Box::new(new_toll));
            let error = PaymentDeniedError::MismatchedSuspect(error);
            Err(error)
        } else {
            match order.toll_declaration.pay(payment.clone(), suspect) {
                Ok(visa) => Ok(Signed::sign(visa, secret_key)),
                Err(err) => Err(PaymentDeniedError::InvalidPayment(err.into(secret_key))),
            }
        }
    }

    fn find_gate_by_id<'a>(
        gates: &'a [Gate],
        order_id: &OrderIdentifier,
    ) -> Result<&'a Gate, GatewayError> {
        let gate = gates
            .iter()
            .find(|g| g.id == order_id.gate_id())
            .ok_or(MissingGateError::new(order_id.gate_id()))?;
        Ok(gate)
    }

    fn find_order_by_id<'a>(
        orders: &'a [Order],
        order_id: &OrderIdentifier,
    ) -> Result<&'a Order, GatewayError> {
        let order =
            orders
                .iter()
                .find(|o| o.id == order_id.order_id())
                .ok_or(MissingOrderError::new(
                    order_id.gate_id(),
                    order_id.order_id(),
                ))?;
        Ok(order)
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
        let id = Uuid::new_v4().to_string();
        Self::with_id(id, destination, orders)
    }

    pub fn with_id(
        id: impl Into<String>,
        destination: Destination,
        orders: Vec<Order>,
    ) -> Result<Self, ConfigError> {
        if orders.is_empty() {
            Err(ConfigError::new(
                "orders",
                "You need to define at least one order for the gate!",
            ))
        } else {
            Ok(Self {
                id: id.into(),
                destination,
                orders,
            })
        }
    }

    /// Id of the gate
    pub fn id(&self) -> &str {
        &self.id
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
    fn pass(
        &self,
        suspect: &Suspect,
        visa: &Option<Signed<Visa>>,
        secret_key: &[u8],
    ) -> Option<Toll> {
        for order in &self.orders {
            let exam = order.examine(suspect, visa, secret_key, &self.id);
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AccessPolicy {
    Whitelist,
    Blacklist,
}

/// Defines conditional process for a [Gate]
pub struct Order {
    id: String,
    descriptions: Vec<Box<dyn Description + Send + Sync>>,
    access_policy: AccessPolicy,
    toll_declaration: Box<dyn Declaration + Send + Sync>,
}

impl Order {
    pub fn new(
        descriptions: Vec<Box<dyn Description + Send + Sync>>,
        access_policy: AccessPolicy,
        toll_declaration: Box<dyn Declaration + Send + Sync>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        Self::with_id(id, descriptions, access_policy, toll_declaration)
    }

    pub fn with_id(
        id: impl Into<String>,
        descriptions: Vec<Box<dyn Description + Send + Sync>>,
        access_policy: AccessPolicy,
        toll_declaration: Box<dyn Declaration + Send + Sync>,
    ) -> Self {
        Self {
            id: id.into(),
            descriptions,
            access_policy,
            toll_declaration,
        }
    }

    fn examine(
        &self,
        suspect: &Suspect,
        visa: &Option<Signed<Visa>>,
        secret_key: &[u8],
        gate_id: &str,
    ) -> Examination {
        let matches_description = self.is_match(suspect);
        let require_toll = (matches_description && self.access_policy == AccessPolicy::Blacklist)
            || (!matches_description && self.access_policy == AccessPolicy::Whitelist);
        let toll = if require_toll && !self.has_valid_visa(suspect, visa, secret_key) {
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

    fn has_valid_visa(
        &self,
        suspect: &Suspect,
        visa: &Option<Signed<Visa>>,
        secret_key: &[u8],
    ) -> bool {
        match visa {
            Option::Some(v) => match v.verify(secret_key) {
                Ok(v) => {
                    v.order_id().order_id() == self.id && Self::matches_visa(suspect, v.suspect())
                }
                Err(_) => false,
            },
            Option::None => false,
        }
    }

    fn matches_visa(suspect: &Suspect, visa_suspect: &Suspect) -> bool {
        visa_suspect.client_ip() == suspect.client_ip()
            && visa_suspect.user_agent() == suspect.user_agent()
            && visa_suspect.destination().contains(suspect.destination())
    }

    pub fn id(&self) -> &str {
        &self.id
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

pub struct SignedPayment {
    toll: Signed<Toll>,
    value: String,
}

impl SignedPayment {
    pub fn new(toll: Signed<Toll>, value: impl Into<String>) -> Self {
        Self {
            toll,
            value: value.into(),
        }
    }

    pub fn verify(&self, secret_key: &[u8]) -> Result<Payment, signatures::InvalidSignatureError> {
        let toll = self.toll.verify(secret_key)?;
        let payment = Payment::new(toll.clone(), self.value.clone());
        Ok(payment)
    }
}
