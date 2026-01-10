#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ServerConfig {
    base_api_url: url::Url,
}
impl ServerConfig {
    pub fn new(base_api_url: url::Url) -> Self {
        Self { base_api_url }
    }
    pub fn base_api_url(&self) -> &url::Url {
        &self.base_api_url
    }
}
