pub struct ServerConfig {
    base_url: url::Url,
}
impl ServerConfig {
    pub fn new(base_url: url::Url) -> Self {
        Self { base_url }
    }
    pub fn base_url(&self) -> &url::Url {
        &self.base_url
    }
}
