use crate::http;
#[cfg(test)]
pub mod tests;

pub trait ReadJson {
    fn read_json(&mut self) -> Result<serde_json::Value, ()>;
}

impl ReadJson for http::Request {
    fn read_json(&mut self) -> Result<serde_json::Value, ()> {
        //let content_length = self.headers().content_length();
        let body = self.body().as_mut().ok_or(())?;
        let mut json = String::new();
        body.read_to_string(&mut json).or(Err(()))?;
        let json = serde_json::from_str(&json).or(Err(()))?;
        Ok(json)
    }
}
