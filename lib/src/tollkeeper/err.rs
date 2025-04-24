use super::*;

/// Return this error when there was a problem during a [Suspect] passing a [Gate].
///
/// E.g. a [Destination] with no matching [Gate]
#[derive(Debug, PartialEq, Eq)]
pub struct GatewayError {
    gate_id: Option<String>,
    order_id: Option<String>,
    description: String,
}

impl GatewayError {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            gate_id: Option::None,
            order_id: Option::None,
            description: description.into(),
        }
    }

    pub fn failure_in_order(
        gate_id: impl Into<String>,
        order_id: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            gate_id: Option::Some(gate_id.into()),
            order_id: Option::Some(order_id.into()),
            description: description.into(),
        }
    }

    pub fn failure_in_gate(gate_id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            gate_id: Option::Some(gate_id.into()),
            order_id: Option::None,
            description: description.into(),
        }
    }
}

impl Error for GatewayError {}
impl Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let order_id = match &self.order_id {
            Option::Some(id) => id.clone(),
            Option::None => String::from("[UNKNOWN]"),
        };
        let gate_id = match &self.gate_id {
            Option::Some(id) => id.clone(),
            Option::None => String::from("[UNKNOWN]"),
        };
        if self.order_id.is_some() {
            write!(
                f,
                "A problem occured while trying to process order '{}': '{}'",
                &order_id, &self.description
            )
        } else if self.gate_id.is_some() {
            write!(
                f,
                "A problem occured while passing gate '{}': '{}'",
                &gate_id, &self.description
            )
        } else {
            write!(
                f,
                "A problem occured while evaluating access: '{}'",
                &self.description
            )
        }
    }
}

impl From<MissingGateError> for GatewayError {
    fn from(value: MissingGateError) -> Self {
        Self {
            gate_id: Option::Some(value.expected_gate_id),
            order_id: Option::None,
            description: String::from("Gate not found"),
        }
    }
}

impl From<MissingOrderError> for GatewayError {
    fn from(value: MissingOrderError) -> Self {
        Self {
            gate_id: Option::Some(value.gate_id),
            order_id: Option::Some(value.expected_order_id),
            description: String::from("Gate not found"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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
