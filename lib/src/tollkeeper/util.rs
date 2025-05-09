pub trait DateTimeProvider {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
}

pub struct DateTimeProviderImpl {}
impl DateTimeProvider for DateTimeProviderImpl {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

pub struct FakeDateTimeProvider(pub chrono::DateTime<chrono::Utc>);
impl DateTimeProvider for FakeDateTimeProvider {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }
}

pub trait RandomStringGen {
    fn generate_random_string(&self) -> String;
}

pub struct RandomStringGenImpl {
    length: u8,
}

impl RandomStringGenImpl {
    pub fn new(length: u8) -> Self {
        Self { length }
    }
}
impl RandomStringGen for RandomStringGenImpl {
    fn generate_random_string(&self) -> String {
        (0..self.length).map(|_| 'a').collect::<String>()
    }
}

pub struct FakeRandomStringGen(pub String);
impl RandomStringGen for FakeRandomStringGen {
    fn generate_random_string(&self) -> String {
        self.0.clone()
    }
}
