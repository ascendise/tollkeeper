use crate::tollkeeper::*;

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
    toll: Toll,
    accept_payment: bool,
}

impl StubDeclaration {
    pub fn new(toll: Toll) -> Self {
        Self {
            toll,
            accept_payment: false,
        }
    }
    pub fn new_payment_stub(toll: Toll, accept_payment: bool) -> Self {
        Self {
            toll,
            accept_payment,
        }
    }
}

impl Declaration for StubDeclaration {
    fn declare(&self) -> Toll {
        self.toll.clone()
    }
    fn pay(&self, payment: &Payment, suspect: &Suspect) -> Result<Visa, Toll> {
        if self.accept_payment {
            let visa = Visa::new(&payment.gate_id, &payment.order_id, suspect.clone());
            Result::Ok(visa)
        } else {
            Result::Err(self.declare())
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
