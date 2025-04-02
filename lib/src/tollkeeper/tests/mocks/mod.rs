use crate::tollkeeper::*;

pub struct SpySuspect {
    client_ip: String,
    user_agent: String,
    target_host: String,
    target_path: String,
    payment: Option<Payment>,
    accessed: bool,
}

impl SpySuspect {
    pub fn new(
        client_ip: impl Into<String>,
        user_agent: impl Into<String>,
        target_host: impl Into<String>,
        target_path: impl Into<String>,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            user_agent: user_agent.into(),
            target_host: target_host.into(),
            target_path: target_path.into(),
            payment: Option::None,
            accessed: false,
        }
    }

    pub fn with_payment(
        client_ip: impl Into<String>,
        user_agent: impl Into<String>,
        target_host: impl Into<String>,
        target_path: impl Into<String>,
        payment: Payment,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            user_agent: user_agent.into(),
            target_host: target_host.into(),
            target_path: target_path.into(),
            payment: Option::Some(payment),
            accessed: false,
        }
    }
}

impl SpySuspect {
    pub fn access(&mut self) {
        self.accessed = true;
    }
    pub fn is_accessed(&self) -> bool {
        self.accessed
    }
}

impl Suspect for SpySuspect {
    fn client_ip(&self) -> &str {
        &self.client_ip
    }
    fn user_agent(&self) -> &str {
        &self.user_agent
    }
    fn target_host(&self) -> &str {
        &self.target_host
    }
    fn target_path(&self) -> &str {
        &self.target_path
    }

    fn payment(&self) -> &Option<Payment> {
        &self.payment
    }
}

pub struct StubDescription {
    matches: bool,
}

impl StubDescription {
    pub fn new(matches: bool) -> Self {
        Self { matches }
    }
}

impl Description for StubDescription {
    fn matches(&self, _: &dyn Suspect) -> bool {
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
    fn pay(&self, _: &Payment) -> bool {
        self.accept_payment
    }
}
