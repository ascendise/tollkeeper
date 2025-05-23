use std::collections::HashMap;

use crate::tollkeeper::{declarations::InvalidPaymentError, *};

pub struct StubDescription {
    matches: bool,
}

impl StubDescription {
    pub fn new(matches: bool) -> Self {
        Self { matches }
    }
}

impl Description for StubDescription {
    fn matches(&self, _: &Suspect) -> bool {
        self.matches
    }
}

pub struct StubDeclaration {
    accept_payment: bool,
}

impl StubDeclaration {
    pub fn new() -> Self {
        Self {
            accept_payment: false,
        }
    }
    pub fn new_payment_stub() -> Self {
        Self {
            accept_payment: true,
        }
    }
}

impl Declaration for StubDeclaration {
    fn declare(&self, suspect: Suspect, order_id: OrderIdentifier) -> Toll {
        Toll::new(suspect, order_id, HashMap::new())
    }

    fn pay(&mut self, payment: Payment, suspect: &Suspect) -> Result<Visa, InvalidPaymentError> {
        if self.accept_payment {
            let visa = Visa::new(payment.order_id().clone(), suspect.clone());
            Result::Ok(visa)
        } else {
            let new_toll = self.declare(suspect.clone(), payment.order_id().clone());
            let error = InvalidPaymentError::new(Box::new(payment.clone()), Box::new(new_toll));
            Result::Err(error)
        }
    }
}

pub struct SpyRequest {
    accessed: bool,
}

impl SpyRequest {
    pub fn new() -> Self {
        Self { accessed: false }
    }

    pub fn access(&mut self) {
        self.accessed = true;
    }

    pub fn accessed(&self) -> bool {
        self.accessed
    }
}
