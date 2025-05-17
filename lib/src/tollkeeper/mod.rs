pub mod declarations;
pub mod descriptions;
pub mod err;
pub mod util;

#[cfg(test)]
mod tests;

use std::{error::Error, fmt::Display};

use declarations::*;
use descriptions::*;
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
        &mut self,
        suspect: &Suspect,
        payment: Payment,
    ) -> Result<Result<Visa, PaymentDeniedError>, GatewayError>;
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
            .find(|g| g.destination() == suspect.destination())
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
        let gate = self.find_gate(suspect)?;
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
        &mut self,
        suspect: &Suspect,
        payment: Payment,
    ) -> Result<Result<Visa, PaymentDeniedError>, GatewayError> {
        let gate = self
            .gates
            .iter_mut()
            .find(|g| g.id == payment.order_id().gate_id())
            .ok_or(MissingGateError::new(payment.order_id().gate_id()))?;
        let order = gate
            .orders
            .iter_mut()
            .find(|o| o.id == payment.order_id().order_id())
            .ok_or(MissingOrderError::new(
                payment.order_id().gate_id(),
                payment.order_id().order_id(),
            ))?;
        if suspect != payment.toll().recipient() {
            let new_toll = order
                .toll_declaration
                .declare(suspect.clone(), OrderIdentifier::new(&gate.id, &order.id));
            let error = MismatchedSuspectError::new(Box::new(suspect.clone()), Box::new(new_toll));
            let error = PaymentDeniedError::MismatchedSuspect(error);
            Result::Ok(Result::Err(error))
        } else {
            let payment_result = match order.toll_declaration.pay(payment.clone(), suspect) {
                Result::Ok(visa) => Result::Ok(visa),
                Result::Err(err) => Result::Err(PaymentDeniedError::InvalidPayment(err)),
            };
            Result::Ok(payment_result)
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
            Option::Some(v) => v.order_id().order_id() == self.id && v.suspect() == suspect,
            Option::None => false,
        }
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
