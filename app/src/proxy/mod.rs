use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt::Display;
use std::io::Write;
use std::net;
use std::str::FromStr;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use tollkeeper::signatures::Signed;
use tollkeeper::Tollkeeper;

use crate::config;
use crate::data_formats::{self, AsHalJson, AsHttpHeader, FromHttpHeader};
use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::{self, Body, Parse};

use super::http::server::*;

#[cfg(test)]
mod tests;

pub struct ProxyServe {
    config: config::ServerConfig,
    proxy_service: Box<dyn ProxyService + Send + Sync>,
}

impl ProxyServe {
    pub fn new(
        config: config::ServerConfig,
        proxy_service: Box<dyn ProxyService + Send + Sync>,
    ) -> Self {
        Self {
            config,
            proxy_service,
        }
    }
}
impl HttpServe for ProxyServe {
    fn serve_http(
        &self,
        client_addr: &net::SocketAddr,
        request: Request,
    ) -> Result<Response, InternalServerError> {
        let response = self.proxy_service.proxy_request(client_addr, request);
        let response = match response {
            Ok(res) => res,
            Err(err) => {
                let toll = err.0;
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
        };
        Ok(response)
    }
}

pub struct ProxyServiceImpl {
    tollkeeper: Tollkeeper,
}
impl ProxyServiceImpl {
    pub fn new(tollkeeper: Tollkeeper) -> Self {
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
    fn extract_visa(headers: &http::request::Headers) -> Option<Visa> {
        let visa_header = headers.extension("X-Keeper-Token")?;
        let visa = Visa::from_http_header(visa_header).ok()?;
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

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct Toll {
    recipient: Recipient,
    order_id: OrderId,
    challenge: HashMap<String, String>,
    signature: String,
}
impl From<&Signed<tollkeeper::declarations::Toll>> for Toll {
    fn from(val: &Signed<tollkeeper::declarations::Toll>) -> Self {
        let (signature, toll) = val.deconstruct();
        Toll {
            recipient: toll.recipient().into(),
            order_id: toll.order_id().into(),
            challenge: toll.challenge().clone(),
            signature: signature.base64(),
        }
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

#[derive(Debug, PartialEq, Eq)]
pub struct OrderId {
    gate_id: String,
    order_id: String,
}
impl From<&tollkeeper::declarations::OrderIdentifier> for OrderId {
    fn from(val: &tollkeeper::declarations::OrderIdentifier) -> Self {
        OrderId {
            gate_id: val.gate_id().into(),
            order_id: val.order_id().into(),
        }
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
#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct Recipient {
    client_ip: String,
    user_agent: String,
    destination: String,
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

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct Visa {
    order_id: OrderId,
    recipient: Recipient,
    signature: Vec<u8>,
}
impl Visa {
    pub fn new(order_id: OrderId, recipient: Recipient, signature: Vec<u8>) -> Self {
        Self {
            order_id,
            recipient,
            signature,
        }
    }

    pub fn order_id(&self) -> &OrderId {
        &self.order_id
    }

    pub fn recipient(&self) -> &Recipient {
        &self.recipient
    }

    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
}
impl AsHttpHeader for Visa {
    fn as_http_header(&self) -> (String, String) {
        let visa_json = serde_json::json!({
            "ip": self.recipient.client_ip,
            "ua": self.recipient.user_agent,
            "dest": self.recipient.destination,
            "order_id": self.order_id
        })
        .to_string();
        let visa_base64 = BASE64_STANDARD.encode(visa_json);
        let signature_base64 = BASE64_STANDARD.encode(&self.signature);
        let header = format!("{visa_base64}.{signature_base64}");
        ("X-Keeper-Token".into(), header)
    }
}
impl FromHttpHeader for Visa {
    type Err = ();
    fn from_http_header(value: &str) -> Result<Visa, ()> {
        let (visa, signature) = value.split_once('.').ok_or(())?;
        let visa_json = BASE64_STANDARD.decode(visa).or(Err(()))?;
        let visa_json: serde_json::Value =
            serde_json::from_slice(visa_json.as_slice()).or(Err(()))?;
        let order_id = visa_json["order_id"].as_str().ok_or(())?;
        let order_id = OrderId::from_str(order_id).or(Err(()))?;
        let recipient = Recipient {
            client_ip: visa_json["ip"].as_str().ok_or(())?.into(),
            user_agent: visa_json["ua"].as_str().ok_or(())?.into(),
            destination: visa_json["dest"].as_str().ok_or(())?.into(),
        };
        let signature = BASE64_STANDARD.decode(signature).or(Err(()))?;
        let visa = Visa::new(order_id, recipient, signature);
        Ok(visa)
    }
}
impl From<Visa> for Signed<tollkeeper::declarations::Visa> {
    fn from(value: Visa) -> Self {
        let visa =
            tollkeeper::declarations::Visa::new(value.order_id.into(), value.recipient.into());
        Signed::new(visa, value.signature)
    }
}
