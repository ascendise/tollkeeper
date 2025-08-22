pub trait AsHalJson {
    fn as_hal_json(&self, base_url: &url::Url) -> serde_json::Value;
}

#[allow(dead_code)] //TODO: Remove after no longer unused
pub trait AsHttpHeader {
    fn as_http_header(&self) -> (String, String);
}
pub trait FromHttpHeader: Sized {
    type Err;
    fn from_http_header(value: &str) -> Result<Self, Self::Err>;
}
