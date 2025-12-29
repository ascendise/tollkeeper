use serde::{de::Visitor, Deserialize, Serialize};

use crate::signatures::Base64;

impl Serialize for Base64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.data)
    }
}

impl<'de> Deserialize<'de> for Base64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(Base64Visitor)
    }
}

struct Base64Visitor;
impl<'de> Visitor<'de> for Base64Visitor {
    type Value = Base64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a base64-encoded value")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match Base64::from(String::from(v)) {
            Ok(v) => Ok(v),
            Err(_) => Err(E::custom(format!("not a base64 string: '{v}'"))),
        }
    }
}
