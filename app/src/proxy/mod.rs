use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::io::Write;
use std::net;
use std::str::FromStr;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use tollkeeper::declarations::Visa;
use tollkeeper::Tollkeeper;

use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::{self, Parse};

use super::http::server::*;

#[cfg(test)]
mod tests;

pub struct ProxyServe {
    proxy_service: Box<dyn ProxyService + Send + Sync>,
}

impl ProxyServe {
    pub fn new(proxy_service: Box<dyn ProxyService + Send + Sync>) -> Self {
        Self { proxy_service }
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
            Err(_) => Response::payment_required(),
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
        let visa_header = headers.extension("X-Keeper-Visa")?;
        let visa_json = BASE64_STANDARD.decode(visa_header).ok()?;
        let visa_json: serde_json::Value = serde_json::from_slice(visa_json.as_slice()).ok()?;
        let order_id = visa_json["order_id"].as_str()?;
        let order_id = OrderId::from_str(order_id).ok()?;
        let recipient = &visa_json["recipient"];
        let recipient = Recipient {
            client_ip: recipient["ip"].as_str()?.into(),
            user_agent: recipient["ua"].as_str()?.into(),
            destination: recipient["dest"].as_str()?.into(),
        };
        let visa = Visa::new(order_id.into(), recipient.into());
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
        match self.tollkeeper.check_access(&suspect, &visa) {
            Ok(()) => Ok(Self::send_request_to_proxy(req)),
            Err(access_err) => match access_err {
                tollkeeper::err::AccessError::AccessDeniedError(toll) => {
                    let toll: Toll = toll.into();
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
#[derive(Debug, PartialEq, Eq)]
pub struct Toll {
    recipient: Recipient,
    order_id: OrderId,
    challenge: HashMap<String, String>,
}
impl From<Box<tollkeeper::declarations::Toll>> for Toll {
    fn from(val: Box<tollkeeper::declarations::Toll>) -> Self {
        Toll {
            recipient: val.recipient().into(),
            order_id: val.order_id().into(),
            challenge: val.challenge().clone(),
        }
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
#[derive(Debug, PartialEq, Eq)]
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
