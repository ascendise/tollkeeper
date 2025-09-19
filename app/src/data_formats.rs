pub trait AsHalJson {
    fn as_hal_json(&self, base_url: &url::Url) -> serde_json::Value;
}

pub trait AsHttpHeader {
    /// Returns the header name and value
    fn as_http_header(&self) -> (String, String);
}
pub trait FromHttpHeader: Sized {
    type Err;
    fn from_http_header(value: &str) -> Result<Self, Self::Err>;
}
