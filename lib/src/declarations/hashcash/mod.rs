#[cfg(test)]
mod tests;

use std::{collections::HashSet, mem::size_of, str::FromStr, sync::Mutex};

use chrono::TimeZone;
use sha1::Digest;

use crate::util::DateTimeProvider;

use super::*;

/// [Declaration] for Hashcash-style [challenges](Toll)
///
/// See <http://hashcash.org> for more information
pub struct HashcashDeclaration {
    difficulty: u8,
    expiry: chrono::Duration,
    date_provider: Box<dyn DateTimeProvider + Send + Sync>,
    double_spent_db: Box<dyn DoubleSpentDatabase + Send + Sync>,
}
impl Declaration for HashcashDeclaration {
    fn declare(&self, suspect: Suspect, order_id: OrderIdentifier) -> Toll {
        let challenge = self.generate_challenge(&suspect);
        Toll::new(suspect, order_id, challenge)
    }

    fn pay(&self, payment: Payment, suspect: &Suspect) -> Result<Visa, PaymentError> {
        let error =
            |decl: &HashcashDeclaration, p: Payment| decl.invalid_payment_error(suspect.clone(), p);
        let stamp = payment.value();
        if self.double_spent_db.is_spent(stamp) {
            return error(self, payment);
        }
        let stamp = match Stamp::from_str(stamp) {
            Ok(s) => s,
            Err(_) => return error(self, payment),
        };
        let expiry_date = self.date_provider.now() - self.expiry;
        let is_expired = stamp.date().0 < expiry_date || stamp.date().0 > self.date_provider.now();
        if !is_expired && stamp.is_valid() {
            let order_id = payment.toll.order_id().clone();
            let visa = Visa::new(order_id, suspect.clone());
            match self.double_spent_db.insert(payment.value().into()) {
                Ok(()) => Ok(visa),
                Err(_) => error(self, payment),
            }
        } else {
            error(self, payment)
        }
    }
}
impl HashcashDeclaration {
    pub fn new(
        difficulty: u8,
        expiry: chrono::Duration,
        date_provider: Box<dyn DateTimeProvider + Send + Sync>,
        double_spent_db: Box<dyn DoubleSpentDatabase + Send + Sync>,
    ) -> Self {
        Self {
            difficulty,
            expiry,
            date_provider,
            double_spent_db,
        }
    }

    fn generate_challenge(&self, suspect: &Suspect) -> Challenge {
        let mut challenge = Challenge::new();
        challenge.insert("ver".into(), "1".into());
        challenge.insert("bits".into(), self.difficulty.to_string());
        challenge.insert("width".into(), Timestamp::width().to_string());
        let dest = suspect.destination();
        challenge.insert(
            "resource".into(),
            format!("{}({}){}", dest.base_url(), dest.port(), dest.path()),
        );
        challenge.insert("ext".into(), format!("suspect.ip={}", suspect.client_ip()));
        challenge
    }

    fn invalid_payment_error(
        &self,
        suspect: Suspect,
        payment: Payment,
    ) -> Result<Visa, PaymentError> {
        let order_id = payment.toll.order_id().clone();
        let toll = self.declare(suspect, order_id);
        let error = PaymentError::new(Box::new(payment), Box::new(toll));
        Err(error)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Stamp {
    ver: u8,
    bits: u8,
    date: Timestamp,
    resource: String,
    ext: Extension,
    rand: String,
    counter: String,
}
impl Stamp {
    fn new(
        ver: u8,
        bits: u8,
        date: Timestamp,
        resource: impl Into<String>,
        ext: Extension,
        rand: impl Into<String>,
        counter: impl Into<String>,
    ) -> Self {
        Self {
            ver,
            bits,
            date,
            resource: resource.into(),
            ext,
            rand: rand.into(),
            counter: counter.into(),
        }
    }

    /// Returns ```true``` if hash has correct amount of zero bits
    pub fn is_valid(&self) -> bool {
        let mut sha1 = sha1::Sha1::new();
        sha1.update(self.to_string().into_bytes());
        let result = sha1.finalize();
        let mut required_bits = self.bits;
        for byte in result {
            let zeros = byte.leading_zeros() as u8;
            if zeros >= required_bits {
                return true;
            }
            required_bits = required_bits.saturating_sub(zeros);
            if zeros as usize != size_of::<u8>() {
                break;
            }
        }
        required_bits == 0
    }

    /// Stamp format version. Currently 1 is expected
    /// Fuck me if there is a new stamp format
    pub fn ver(&self) -> u8 {
        self.ver
    }

    /// Number of zero bits the hash has to start with.
    pub fn bits(&self) -> u8 {
        self.bits
    }

    /// ```YYMMDDhhmmss``` format date the stamp was created at. Used for expiry check
    pub fn date(&self) -> &Timestamp {
        &self.date
    }

    /// Resource the stamp is minted for
    pub fn resource(&self) -> &str {
        &self.resource
    }

    /// Extension field used by tollkeeper
    pub fn ext(&self) -> &Extension {
        &self.ext
    }

    /// string of random characters to avoid preimage attacks
    pub fn rand(&self) -> &str {
        &self.rand
    }

    /// string generated by user to get the required amount of bits
    pub fn counter(&self) -> &str {
        &self.counter
    }

    fn create_stamp(values: Vec<&str>) -> Result<Stamp, ()> {
        let stamp = Stamp::new(
            Self::parse_ver(values[0])?,
            Self::parse_bits(values[1])?,
            Self::parse_date(values[2])?,
            Self::parse_resource(values[3])?,
            Self::parse_ext(values[4])?,
            Self::parse_rand(values[5])?,
            Self::parse_counter(values[6])?,
        );
        Ok(stamp)
    }

    fn parse_ver(values: &str) -> Result<u8, ()> {
        if values == "1" {
            Ok(1)
        } else {
            Err(())
        }
    }

    fn parse_bits(values: &str) -> Result<u8, ()> {
        match values.parse() {
            Ok(b) => Ok(b),
            _ => Err(()),
        }
    }

    fn parse_date(values: &str) -> Result<Timestamp, ()> {
        if values.len() != 12 {
            return Err(());
        }
        let result = |r: Result<i32, std::num::ParseIntError>| match r {
            Ok(v) => Ok(v),
            Err(_) => Err(()),
        };
        let year: i32 = result(values[0..2].parse())?;
        let result = |r: Result<u32, std::num::ParseIntError>| match r {
            Ok(v) => Ok(v),
            Err(_) => Err(()),
        };
        let year = 2000 + year;
        let month: u32 = result(values[2..4].parse())?;
        let day: u32 = result(values[4..6].parse())?;
        let hour: u32 = result(values[6..8].parse())?;
        let minute: u32 = result(values[8..10].parse())?;
        let second: u32 = result(values[10..12].parse())?;
        let time = chrono::Utc
            .with_ymd_and_hms(year, month, day, hour, minute, second)
            .unwrap()
            .to_utc();
        let time = Timestamp(time);
        Ok(time)
    }

    fn parse_resource(values: &str) -> Result<String, ()> {
        Ok(values.into())
    }

    fn parse_ext(values: &str) -> Result<Extension, ()> {
        let kv_pairs = values.split(';');
        let kv_pairs: Vec<Vec<&str>> = kv_pairs.map(|kv| kv.split('=').collect()).collect();
        let mut ext = Vec::<(String, String)>::new();
        for kv in kv_pairs {
            if kv.len() != 2 {
                return Err(());
            }
            ext.push((kv[0].into(), kv[1].into()));
        }
        let ext = Extension(ext);
        Ok(ext)
    }

    fn parse_rand(values: &str) -> Result<String, ()> {
        Ok(values.into())
    }

    fn parse_counter(values: &str) -> Result<String, ()> {
        Ok(values.into())
    }
}
impl Display for Stamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ver = self.ver();
        let bits = self.bits();
        let date = self.date();
        let resource = self.resource();
        let ext = self.ext();
        let rand = self.rand();
        let counter = self.counter();
        write!(f, "{ver}:{bits}:{date}:{resource}:{ext}:{rand}:{counter}")
    }
}
impl FromStr for Stamp {
    type Err = ParseStampError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s.split(':').collect::<Vec<&str>>();
        if values.len() != 7 {
            return Err(ParseStampError);
        }
        match Stamp::create_stamp(values) {
            Ok(s) => Ok(s),
            Err(()) => Err(ParseStampError),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
struct ParseStampError;
#[derive(Debug, PartialEq, Eq)]
struct Timestamp(chrono::DateTime<chrono::Utc>);
impl Timestamp {
    fn width() -> usize {
        12
    }
}
impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let date_str = self.0.format("%y%m%d%H%M%S");
        write!(f, "{date_str}")
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Extension(Vec<(String, String)>);
impl Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ext_str = self
            .0
            .iter()
            .map(|e| format!("{}={}", e.0, e.1))
            .collect::<Vec<String>>()
            .join(";");
        write!(f, "{ext_str}")
    }
}

pub trait DoubleSpentDatabase {
    fn insert(&self, stamp: String) -> Result<(), DuplicateStampError>;
    fn is_spent(&self, stamp: &str) -> bool;
    fn stamps(&self) -> HashSet<String>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct DuplicateStampError {
    stamp: String,
}

impl DuplicateStampError {
    pub fn new(stamp: String) -> Self {
        Self { stamp }
    }

    pub fn stamp(&self) -> &str {
        &self.stamp
    }
}
impl Error for DuplicateStampError {}
impl Display for DuplicateStampError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stamp '{}' is already spent", self.stamp)
    }
}

/// An in-memory implementation of a [DoubleSpentDatabase]
pub struct DoubleSpentDatabaseImpl {
    stamps: Mutex<HashSet<String>>,
}
impl Default for DoubleSpentDatabaseImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl DoubleSpentDatabaseImpl {
    pub fn new() -> Self {
        Self {
            stamps: Mutex::new(HashSet::<String>::new()),
        }
    }
    pub fn init(stamps: HashSet<String>) -> Self {
        Self {
            stamps: Mutex::new(stamps),
        }
    }
}
impl DoubleSpentDatabase for DoubleSpentDatabaseImpl {
    fn insert(&self, stamp: String) -> Result<(), DuplicateStampError> {
        let mut stamps = self.stamps.lock().unwrap();
        let is_new_stamp = stamps.insert(stamp.clone());
        if is_new_stamp {
            Ok(())
        } else {
            Err(DuplicateStampError::new(stamp))
        }
    }

    fn is_spent(&self, stamp: &str) -> bool {
        let stamps = &self.stamps.lock().unwrap();
        stamps.contains(stamp)
    }

    fn stamps(&self) -> HashSet<String> {
        let stamps = self.stamps.lock().unwrap();
        stamps.clone()
    }
}
