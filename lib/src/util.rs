pub trait DateTimeProvider {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
}

pub struct DateTimeProviderImpl;
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
