use super::*;

/// Return this error when [Suspect] is required to pay [Toll]
#[derive(Debug, PartialEq, Eq)]
pub struct AccessDeniedError {
    toll: Box<Toll>,
}

impl AccessDeniedError {
    pub fn new(toll: Box<Toll>) -> Self {
        Self { toll }
    }

    pub fn toll(&self) -> &Toll {
        &self.toll
    }
}

impl Error for AccessDeniedError {}
impl Display for AccessDeniedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Access denied; Pay the toll and acquire a visa to enter")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentDeniedError {
    InvalidPayment(InvalidPaymentError),
    MismatchedSuspect(MismatchedSuspectError),
}

impl Error for PaymentDeniedError {}
impl Display for PaymentDeniedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPayment(e) => e.fmt(f),
            Self::MismatchedSuspect(e) => e.fmt(f),
        }
    }
}
impl From<InvalidPaymentError> for PaymentDeniedError {
    fn from(value: InvalidPaymentError) -> Self {
        PaymentDeniedError::InvalidPayment(value)
    }
}
impl From<MismatchedSuspectError> for PaymentDeniedError {
    fn from(value: MismatchedSuspectError) -> Self {
        PaymentDeniedError::MismatchedSuspect(value)
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

/// Return this error when [Payment] was issued for different [Suspect]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MismatchedSuspectError {
    expected: Box<Suspect>,
    new_toll: Box<Toll>,
}

impl MismatchedSuspectError {
    pub fn new(expected: Box<Suspect>, new_toll: Box<Toll>) -> Self {
        Self { expected, new_toll }
    }

    pub fn expected(&self) -> &Suspect {
        &self.expected
    }

    pub fn actual(&self) -> &Suspect {
        &self.new_toll.recipient
    }

    pub fn new_toll(&self) -> &Toll {
        &self.new_toll
    }
}

impl Error for MismatchedSuspectError {}
impl Display for MismatchedSuspectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "'{}' tried to pay toll for {}!",
            self.actual().identifier(),
            self.expected().identifier(),
        )
    }
}

/// Return this error when there was a problem during a [Suspect] passing a [Gate].
///
/// E.g. a [Destination] with no matching [Gate]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    MissingGate(MissingGateError),
    MissingOrder(MissingOrderError),
}

impl Error for GatewayError {}
impl Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingGate(e) => e.fmt(f),
            Self::MissingOrder(e) => e.fmt(f),
        }
    }
}
impl From<MissingGateError> for GatewayError {
    fn from(value: MissingGateError) -> Self {
        Self::MissingGate(value)
    }
}
impl From<MissingOrderError> for GatewayError {
    fn from(value: MissingOrderError) -> Self {
        Self::MissingOrder(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingGateError {
    expected_gate_id: String,
}

impl MissingGateError {
    pub fn new(expected_gate_id: impl Into<String>) -> Self {
        Self {
            expected_gate_id: expected_gate_id.into(),
        }
    }

    pub fn gate_id(&self) -> &str {
        &self.expected_gate_id
    }
}
impl Error for MissingGateError {}
impl Display for MissingGateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gate {} does not exist!", &self.gate_id())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingOrderError {
    gate_id: String,
    expected_order_id: String,
}

impl MissingOrderError {
    pub fn new(gate_id: impl Into<String>, expected_order_id: impl Into<String>) -> Self {
        Self {
            gate_id: gate_id.into(),
            expected_order_id: expected_order_id.into(),
        }
    }

    pub fn gate_id(&self) -> &str {
        &self.gate_id
    }

    pub fn order_id(&self) -> &str {
        &self.expected_order_id
    }
}

impl Error for MissingOrderError {}
impl Display for MissingOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Gate '{}' does not contain order '{}'",
            &self.gate_id(),
            &self.order_id()
        )
    }
}

/// Return this error when there are problems during creation of the [Tollkeeper] or
/// it's subentities caused by wrong init arguments
#[derive(Debug, Eq, Clone)]
pub struct ConfigError {
    key: String,
    description: String,
}

impl ConfigError {
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }

    /// Property that caused the error
    pub fn key(&self) -> &str {
        &self.key
    }

    /// User-friendly message describing what is wrong with the configuration
    /// Not part of equality comparison
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl Error for ConfigError {}
impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to load config value '{}': '{}'",
            &self.key, &self.description
        )
    }
}

impl PartialEq for ConfigError {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
