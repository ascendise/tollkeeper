#[cfg(test)]
mod tests;

use std::{str::FromStr, sync::Mutex};

use chrono::TimeZone;
use ringmap::RingSet;
use sha1::Digest;

use crate::{descriptions::Destination, util::DateTimeProvider};

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
            tracing::info!("Stamp is already spent!");
            return error(self, payment);
        }
        let stamp = match Stamp::from_str(stamp) {
            Ok(s) => s,
            Err(_) => {
                tracing::info!("Stamp not parseable!");
                return error(self, payment);
            }
        };
        let minimum_valid_date = self.date_provider.now() - self.expiry - Self::GRACE_PERIOD;
        let today = self.date_provider.now() + Self::GRACE_PERIOD;
        let is_expired = stamp.date().0 < minimum_valid_date;
        let is_in_the_future = stamp.date().0 > today;
        if !(is_expired || is_in_the_future)
            && self.is_matching_challenge(suspect, &stamp)
            && stamp.is_valid()
        {
            match self.try_create_visa(&payment) {
                Ok(v) => Ok(v),
                Err(_) => {
                    tracing::info!("Stamp is already spent!");
                    error(self, payment)
                }
            }
        } else {
            tracing::info!("Stamp invalid! (No UTC?)");
            error(self, payment)
        }
    }
}
impl HashcashDeclaration {
    /// Time duration allowed after expiry to deal with small time desync
    const GRACE_PERIOD: chrono::TimeDelta = chrono::TimeDelta::seconds(5);

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
        let resource = Resource(suspect.destination().clone());
        challenge.insert("resource".into(), resource.to_string());
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

    fn is_matching_challenge(&self, suspect: &Suspect, stamp: &Stamp) -> bool {
        let stamp_ip = &stamp.ext().0.get("suspect.ip");
        let matches_suspect_ip = stamp_ip.map(|s| s == suspect.client_ip()).unwrap_or(false);
        self.difficulty == stamp.bits
            && suspect.destination() == &stamp.resource.0
            && matches_suspect_ip
    }

    fn try_create_visa(&self, payment: &Payment) -> Result<Visa, StampError> {
        match self.double_spent_db.insert(payment.value().into()) {
            Ok(()) => {
                let order_id = payment.toll.order_id().clone();
                let visa = Visa::new(
                    order_id,
                    payment.toll.recipient().clone(),
                    self.date_provider.now() + self.expiry,
                );
                Ok(visa)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Stamp {
    ver: u8,
    bits: u8,
    date: Timestamp,
    resource: Resource,
    ext: Extension,
    rand: String,
    counter: String,
}
impl Stamp {
    fn new(
        ver: u8,
        bits: u8,
        date: Timestamp,
        resource: Resource,
        ext: Extension,
        rand: impl Into<String>,
        counter: impl Into<String>,
    ) -> Self {
        Self {
            ver,
            bits,
            date,
            resource,
            ext,
            rand: rand.into(),
            counter: counter.into(),
        }
    }

    /// Returns `true` if hash has correct amount of zero bits
    pub fn is_valid(&self) -> bool {
        let mut sha1 = sha1::Sha1::new();
        sha1.update(self.to_string().into_bytes());
        let result = sha1.finalize();
        let mut zero_bits_left = self.bits;
        for byte in result {
            let expected_zeroes = zero_bits_left.min(8);
            let shift = u32::from(8 - expected_zeroes);
            if byte.checked_shr(shift).unwrap_or(0) != 0 || zero_bits_left == 0 {
                break;
            } else {
                zero_bits_left = zero_bits_left.saturating_sub(expected_zeroes);
            }
        }
        zero_bits_left == 0
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

    /// `YYMMDDhhmmss` format date the stamp was created at. Used for expiry check
    pub fn date(&self) -> &Timestamp {
        &self.date
    }

    /// Resource the stamp is minted for
    pub fn resource(&self) -> &Resource {
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

    fn parse_resource(values: &str) -> Result<Resource, ()> {
        Resource::from_str(values).or(Err(()))
    }

    fn parse_ext(values: &str) -> Result<Extension, ()> {
        Extension::from_str(values).or(Err(()))
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
struct Resource(Destination);
impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}){}",
            self.0.base_url(),
            self.0.port(),
            self.0.path()
        )
    }
}
impl FromStr for Resource {
    type Err = ParseStampError;

    fn from_str(s: &str) -> Result<Self, ParseStampError> {
        let regex = regex::Regex::new(r"^(?P<host>.+)\((?P<port>\d+)\)(?P<path>/.*)$").unwrap();
        let (_, [host, port, path]) = regex
            .captures(s)
            .map(|c| c.extract())
            .ok_or(ParseStampError)?;
        let port = port.parse().or(Err(ParseStampError))?;
        let destination = Destination::new(host, port, path);
        Ok(Resource(destination))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Extension(indexmap::IndexMap<String, String>);
impl Extension {
    pub fn empty() -> Self {
        Extension(indexmap::indexmap![])
    }
}
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
impl FromStr for Extension {
    type Err = ParseStampError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Extension::empty());
        }
        let kv_pairs = s.split(';');
        let kv_pairs: Vec<Vec<&str>> = kv_pairs.map(|kv| kv.split('=').collect()).collect();
        let mut ext = indexmap::IndexMap::new();
        for kv in kv_pairs {
            if kv.len() != 2 {
                return Err(ParseStampError);
            }
            ext.insert(kv[0].into(), kv[1].into());
        }
        let ext = Extension(ext);
        Ok(ext)
    }
}

pub trait DoubleSpentDatabase {
    fn insert(&self, stamp: String) -> Result<(), StampError>;
    fn is_spent(&self, stamp: &str) -> bool;
    fn stamps(&self) -> RingSet<String>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum StampError {
    DuplicateStamp(DuplicateStampError),
    StampTooLong,
}
impl Error for StampError {}
impl Display for StampError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StampError::DuplicateStamp(e) => write!(f, "{e}"),
            StampError::StampTooLong => write!(f, "Stamp too long"),
        }
    }
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
    stamps: Mutex<RingSet<String>>,
    stamp_limit: usize,
}
impl Default for DoubleSpentDatabaseImpl {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DoubleSpentDatabaseImpl {
    const STAMP_SIZE_LIMIT: usize = 255;
    const STAMP_COUNT_LIMIT: usize = 10000;

    pub fn new(stamp_limit: Option<usize>) -> Self {
        Self::init(RingSet::new(), stamp_limit)
    }
    pub fn init(stamps: RingSet<String>, stamp_limit: Option<usize>) -> Self {
        let stamp_limit = stamp_limit.unwrap_or(Self::STAMP_COUNT_LIMIT);
        Self {
            stamps: Mutex::new(stamps),
            stamp_limit,
        }
    }

    fn assert_stamp_size(stamp: &str) -> Result<(), StampError> {
        if stamp.len() <= Self::STAMP_SIZE_LIMIT {
            Ok(())
        } else {
            tracing::debug!("Oversized stamp! ({})", stamp.len());
            Err(StampError::StampTooLong)
        }
    }

    fn cleanup_old_stamps(&self, stamps: &mut RingSet<String>) {
        while stamps.len() > self.stamp_limit {
            stamps.pop_front();
        }
    }
}
impl DoubleSpentDatabase for DoubleSpentDatabaseImpl {
    fn insert(&self, stamp: String) -> Result<(), StampError> {
        Self::assert_stamp_size(&stamp)?;
        let mut stamps = self.stamps.lock().unwrap();
        let is_new_stamp = stamps.insert(stamp.clone());
        if is_new_stamp {
            self.cleanup_old_stamps(&mut stamps);
            Ok(())
        } else {
            let err = StampError::DuplicateStamp(DuplicateStampError::new(stamp));
            Err(err)
        }
    }

    fn is_spent(&self, stamp: &str) -> bool {
        let stamps = &self.stamps.lock().unwrap();
        stamps.contains(stamp)
    }

    fn stamps(&self) -> RingSet<String> {
        let stamps = self.stamps.lock().unwrap();
        stamps.clone()
    }
}
