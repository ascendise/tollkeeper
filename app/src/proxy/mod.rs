use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::Write;
use std::net;
use std::str::FromStr;
use std::sync::Arc;

use serde::ser::SerializeMap;
use tollkeeper::signatures::{Base64, Signed};
use tollkeeper::Tollkeeper;

use crate::data_formats::{self, AsHalJson, FromHttpHeader};
use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::{self, Body, Parse};
use crate::templates::{SerializedData, TemplateRenderer};
use crate::{config, payment};

use super::http::server::*;

#[cfg(test)]
mod tests;

pub struct ProxyServe {
    config: config::ServerConfig,
    proxy_service: Box<dyn ProxyService + Send + Sync>,
    template_renderer: Box<dyn TemplateRenderer + Send + Sync>,
}

impl ProxyServe {
    pub fn new(
        config: config::ServerConfig,
        proxy_service: Box<dyn ProxyService + Send + Sync>,
        template_renderer: Box<dyn TemplateRenderer + Send + Sync>,
    ) -> Self {
        Self {
            config,
            proxy_service,
            template_renderer,
        }
    }

    fn toll_to_json_response(&self, toll: &Toll) -> Response {
        let json = toll.as_hal_json(self.config.base_url());
        let data: VecDeque<u8> = json.to_string().into_bytes().into();
        let content_length = data.len().to_string();
        let body = http::StreamBody::new(data);
        let body = Box::new(body) as Box<dyn Body>;
        let mut headers = http::Headers::empty();
        headers.insert("Content-Type", "application/hal+json");
        headers.insert("Content-Length", content_length);
        let headers = http::response::Headers::new(headers);
        Response::payment_required(headers, Some(body))
    }

    fn toll_to_html_response(&self, toll: &Toll) -> Result<Response, InternalServerError> {
        let base_url = self.config.base_url();
        let toll = toll.as_hal_json(base_url);
        let page_html = self
            .template_renderer
            .render("challenge.html", &SerializedData::new(toll))
            .or(Err(InternalServerError::new()))?;
        let mut headers = http::Headers::empty();
        headers.insert("Content-Type", "text/html");
        headers.insert("Content-Length", page_html.len().to_string());
        let headers = http::response::Headers::new(headers);
        let page_html_stream: VecDeque<u8> = page_html.into_bytes().into();
        let body = http::StreamBody::new(page_html_stream);
        let body = Box::new(body) as Box<dyn Body>;
        Ok(Response::payment_required(headers, Some(body)))
    }
}
impl HttpServe for ProxyServe {
    fn serve_http(
        &self,
        client_addr: &net::SocketAddr,
        request: Request,
    ) -> Result<Response, InternalServerError> {
        let accept_header: String = request.headers().accept().unwrap_or("").into();
        let response = self.proxy_service.proxy_request(client_addr, request);
        let response = match response {
            Ok(res) => res,
            Err(err) => {
                if accept_header.contains("html") {
                    self.toll_to_html_response(&err.0)?
                } else {
                    self.toll_to_json_response(&err.0)
                }
            }
        };
        Ok(response)
    }
}

pub struct ProxyServiceImpl {
    tollkeeper: Arc<Tollkeeper>,
}
impl ProxyServiceImpl {
    pub fn new(tollkeeper: Arc<Tollkeeper>) -> Self {
        Self { tollkeeper }
    }
    fn get_host(request: &Request) -> String {
        let target = request.absolute_target();
        let host = target.host_str().unwrap();
        let port = target.port().unwrap_or(80);
        let addr = format!("{host}:{port}");
        addr
    }
    fn create_suspect(
        client_addr: &net::SocketAddr,
        req: &Request,
    ) -> tollkeeper::descriptions::Suspect {
        let default_ua = String::from("");
        let user_agent = req.headers().user_agent().unwrap_or(&default_ua);
        let target = req.absolute_target();
        let destination = tollkeeper::descriptions::Destination::new(
            target.host_str().unwrap(),
            target.port().unwrap_or(80),
            target.path(),
        );
        tollkeeper::descriptions::Suspect::new(
            client_addr.ip().to_string(),
            user_agent,
            destination,
        )
    }
    fn extract_visa(headers: &http::request::Headers) -> Option<payment::Visa> {
        let visa_header = headers
            .extension("X-Keeper-Token")
            .or_else(|| headers.cookie("X-Keeper-Token"))?;
        let visa = payment::Visa::from_http_header(visa_header).ok()?;
        Some(visa)
    }
    fn send_request_to_proxy(req: Request) -> Response {
        let addr = Self::get_host(&req);
        let mut target_conn = net::TcpStream::connect(&addr).unwrap();
        target_conn.write_all(&req.into_bytes()).unwrap();

        Response::parse(target_conn.try_clone().unwrap()).unwrap()
    }
}
impl ProxyService for ProxyServiceImpl {
    fn proxy_request(
        &self,
        client_addr: &net::SocketAddr,
        req: http::Request,
    ) -> Result<http::Response, PaymentRequiredError> {
        let suspect = Self::create_suspect(client_addr, &req);
        let visa = Self::extract_visa(req.headers());
        let visa = visa.map(|v| v.into());
        match self.tollkeeper.check_access(&suspect, &visa) {
            Ok(()) => Ok(Self::send_request_to_proxy(req)),
            Err(access_err) => match access_err {
                tollkeeper::err::AccessError::AccessDeniedError(toll) => {
                    let toll: Toll = toll.as_ref().into();
                    Err(PaymentRequiredError(Box::new(toll)))
                }
                tollkeeper::err::AccessError::DestinationNotFound(_) => {
                    Ok(http::Response::not_found())
                }
            },
        }
    }
}

pub trait ProxyService {
    fn proxy_request(
        &self,
        client_addr: &net::SocketAddr,
        req: http::Request,
    ) -> Result<http::Response, PaymentRequiredError>;
}
#[derive(Debug, PartialEq, Eq)]
pub struct PaymentRequiredError(Box<Toll>);
impl Error for PaymentRequiredError {}
impl Display for PaymentRequiredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No payment found for accessing url!")
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Toll {
    recipient: Recipient,
    order_id: OrderId,
    challenge: Challenge,
    signature: Base64,
}

impl Toll {
    pub fn new(
        recipient: Recipient,
        order_id: OrderId,
        challenge: Challenge,
        signature: Base64,
    ) -> Self {
        Self {
            recipient,
            order_id,
            challenge,
            signature,
        }
    }

    /// Client requesting resource
    pub fn recipient(&self) -> &Recipient {
        &self.recipient
    }

    /// Id of the order that issued this toll
    pub fn order_id(&self) -> &OrderId {
        &self.order_id
    }

    /// Challenge parameters
    pub fn challenge(&self) -> &Challenge {
        &self.challenge
    }

    /// Base64 encoded signature
    pub fn signature(&self) -> &Base64 {
        &self.signature
    }
}
impl From<&Signed<tollkeeper::declarations::Toll>> for Toll {
    fn from(val: &Signed<tollkeeper::declarations::Toll>) -> Self {
        let (signature, toll) = val.deconstruct();
        Toll {
            recipient: toll.recipient().into(),
            order_id: toll.order_id().into(),
            challenge: toll.challenge().into(),
            signature: signature.base64(),
        }
    }
}
impl From<Signed<tollkeeper::declarations::Toll>> for Toll {
    fn from(val: Signed<tollkeeper::declarations::Toll>) -> Self {
        (&val).into()
    }
}
impl TryFrom<Toll> for Signed<tollkeeper::declarations::Toll> {
    type Error = base64::DecodeError;

    fn try_from(value: Toll) -> Result<Self, base64::DecodeError> {
        let toll = tollkeeper::declarations::Toll::new(
            value.recipient.into(),
            value.order_id.into(),
            value.challenge.into(),
        );
        let toll = Signed::new(toll, value.signature.decode());
        Ok(toll)
    }
}
impl data_formats::AsHalJson for Toll {
    fn as_hal_json(&self, base_url: &url::Url) -> serde_json::Value {
        serde_json::json!({
            "toll": self,
            "_links": {
                "pay": format!("{base_url}api/pay/")
            }
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Challenge {
    values: Vec<(String, String)>,
}
impl Challenge {
    pub fn new(values: Vec<(String, String)>) -> Self {
        Self { values }
    }

    pub fn empty() -> Self {
        Self { values: Vec::new() }
    }

    pub fn values(&self) -> &[(String, String)] {
        &self.values
    }
}
impl From<Challenge> for tollkeeper::declarations::Challenge {
    fn from(value: Challenge) -> Self {
        let mut challenge = indexmap::IndexMap::new();
        for (k, v) in value.values {
            challenge.insert(k, v);
        }
        challenge
    }
}
impl From<&tollkeeper::declarations::Challenge> for Challenge {
    fn from(value: &tollkeeper::declarations::Challenge) -> Self {
        let mut challenge = vec![];
        for (k, v) in value {
            challenge.push((k.clone(), v.clone()));
        }
        Self::new(challenge)
    }
}
impl serde::Serialize for Challenge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let values = self.values();
        let mut struct_builder = serializer.serialize_map(Some(values.len()))?;
        for (k, v) in &self.values {
            struct_builder.serialize_entry(k, v)?;
        }
        struct_builder.end()
    }
}
impl<'de> serde::Deserialize<'de> for Challenge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ChallengeVisitor;
        impl<'de> serde::de::Visitor<'de> for ChallengeVisitor {
            type Value = Challenge;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string in the format 'part1:part2'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut challenge = Vec::new();
                while let Some((key, value)) = &map.next_entry::<String, String>()? {
                    challenge.push((String::from(key), String::from(value)));
                }
                Ok(Challenge::new(challenge))
            }
        }

        deserializer.deserialize_map(ChallengeVisitor)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OrderId {
    gate_id: String,
    order_id: String,
}

impl OrderId {
    pub fn new(gate_id: impl Into<String>, order_id: impl Into<String>) -> Self {
        Self {
            gate_id: gate_id.into(),
            order_id: order_id.into(),
        }
    }

    pub fn gate_id(&self) -> &str {
        &self.gate_id
    }

    pub fn order_id(&self) -> &str {
        &self.order_id
    }
}
impl From<&tollkeeper::declarations::OrderIdentifier> for OrderId {
    fn from(val: &tollkeeper::declarations::OrderIdentifier) -> Self {
        OrderId {
            gate_id: val.gate_id().into(),
            order_id: val.order_id().into(),
        }
    }
}
impl From<tollkeeper::declarations::OrderIdentifier> for OrderId {
    fn from(val: tollkeeper::declarations::OrderIdentifier) -> Self {
        (&val).into()
    }
}
impl From<OrderId> for tollkeeper::declarations::OrderIdentifier {
    fn from(value: OrderId) -> Self {
        Self::new(value.gate_id, value.order_id)
    }
}
impl FromStr for OrderId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (gate_id, order_id) = s.split_once("#").ok_or(())?;
        let order_id = OrderId {
            gate_id: gate_id.into(),
            order_id: order_id.into(),
        };
        Ok(order_id)
    }
}
impl Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.gate_id, self.order_id)
    }
}
impl serde::Serialize for OrderId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> serde::Deserialize<'de> for OrderId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct OrderIdVisitor;
        impl serde::de::Visitor<'_> for OrderIdVisitor {
            type Value = OrderId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string in the format 'part1#part2'")
            }

            fn visit_str<E>(self, value: &str) -> Result<OrderId, E>
            where
                E: serde::de::Error,
            {
                let parse_err = serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(value),
                    &"gate_id#order_id",
                );
                let order_id = OrderId::from_str(value).or(Err(parse_err))?;
                Ok(order_id)
            }
        }

        deserializer.deserialize_str(OrderIdVisitor)
    }
}
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Recipient {
    client_ip: String,
    user_agent: String,
    destination: String,
}
impl Recipient {
    pub fn new(
        client_ip: impl Into<String>,
        user_agent: impl Into<String>,
        destination: impl Into<String>,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            user_agent: user_agent.into(),
            destination: destination.into(),
        }
    }

    pub fn client_ip(&self) -> &str {
        &self.client_ip
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }
}
impl From<&tollkeeper::descriptions::Suspect> for Recipient {
    fn from(val: &tollkeeper::descriptions::Suspect) -> Self {
        Recipient {
            client_ip: val.client_ip().into(),
            user_agent: val.user_agent().into(),
            destination: val.destination().to_string(),
        }
    }
}
impl From<tollkeeper::descriptions::Suspect> for Recipient {
    fn from(val: tollkeeper::descriptions::Suspect) -> Self {
        (&val).into()
    }
}
impl From<Recipient> for tollkeeper::descriptions::Suspect {
    fn from(recipient: Recipient) -> Self {
        let url = format!("http://{}", recipient.destination);
        let url = url::Url::parse(&url).unwrap();
        let destination = tollkeeper::descriptions::Destination::new(
            url.host().unwrap().to_string(),
            url.port().unwrap_or(80),
            url.path(),
        );
        Self::new(recipient.client_ip, recipient.user_agent, destination)
    }
}
