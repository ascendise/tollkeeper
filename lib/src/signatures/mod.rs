use std::{error::Error, fmt::Display};

use base64::{prelude::BASE64_STANDARD, Engine};
use hmac::Mac;

#[cfg(test)]
mod tests;

/// A wrapper to make values tamper-proof against third parties
/// by creating a signature of a byte repesentation of the wrapped value
///
/// Uses HMAC-SHA256 for signing
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Signed<T: AsBytes> {
    value: T,
    signature: Signature,
}

impl<T: AsBytes> Signed<T> {
    /// Creates a [Signed] with a user-specified signature that may be invalid
    pub fn new(value: T, signature: Vec<u8>) -> Self {
        let signature = Signature(signature);
        Self { value, signature }
    }

    /// Create a new [Signed] using a secret key
    pub fn sign(value: T, secret_key: &[u8]) -> Self {
        let bytes = value.as_bytes();
        let signature = Signature::sign(bytes, secret_key);
        Self { value, signature }
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Checks the siganture given the secret_key and either returns the wrapped value
    /// or an error in case the signature is invalid/forged
    pub fn verify(&self, secret_key: &[u8]) -> Result<&T, InvalidSignatureError> {
        if !self.signature.is_valid(&self.value, secret_key) {
            Err(InvalidSignatureError)
        } else {
            Ok(&self.value)
        }
    }

    /// Returns the signed value as tuple containing the `signature` and `value`
    /// This allows access to the wrapped object without the `signature` having to be valid
    ///
    /// Using [Self::deconstruct] is only advised if used in a context, where the wrapped value
    /// does not have to be trusted. E.g. serialization
    pub fn deconstruct(&self) -> (&Signature, &T) {
        (&self.signature, &self.value)
    }
}

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

/// Used to create a copy of the value as binary
pub trait AsBytes {
    /// Returns a binary representation of the struct
    fn as_bytes(&self) -> Vec<u8>;
}
impl<T> AsBytes for T
where
    T: AsRef<[u8]>,
{
    fn as_bytes(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }
}

/// Returned when trying to access a [Signed] with invalid signature
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidSignatureError;
impl Error for InvalidSignatureError {}
impl Display for InvalidSignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signature does not match with key!")
    }
}

/// HMAC-SHA256 signature
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Signature(Vec<u8>);
impl Signature {
    fn sign(value: impl AsBytes, secret_key: &[u8]) -> Self {
        let signature = Self::create_signature(&value.as_bytes(), secret_key);
        Self(signature)
    }

    /// Check if signature is valid. Requires the original object
    pub fn is_valid(&self, value: &impl AsBytes, secret_key: &[u8]) -> bool {
        let value = value.as_bytes();
        let expected_signature = Self::create_signature(&value, secret_key);
        expected_signature == self.0
    }

    fn create_signature(value: &[u8], key: &[u8]) -> Vec<u8> {
        let mut hmac = HmacSha256::new_from_slice(key).expect("Invalid key length for signing");
        hmac.update(value);

        hmac.finalize().into_bytes().to_vec()
    }

    pub fn raw(&self) -> &[u8] {
        &self.0
    }

    pub fn base64(&self) -> String {
        BASE64_STANDARD.encode(&self.0)
    }
}

/// Provides access to a secret key through a key ring/file/...
pub trait SecretKeyProvider {
    fn read_secret_key(&self) -> &[u8];
}
/// Provides secret key from memory. Not advised for production use :)
pub struct InMemorySecretKeyProvider(Vec<u8>);
impl InMemorySecretKeyProvider {
    pub fn new(key: Vec<u8>) -> Self {
        Self(key)
    }
}
impl SecretKeyProvider for InMemorySecretKeyProvider {
    fn read_secret_key(&self) -> &[u8] {
        &self.0
    }
}
