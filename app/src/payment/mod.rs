#[cfg(test)]
mod tests;

use std::{collections::VecDeque, error::Error, fmt::Display, str::FromStr, sync::Arc};

use base64::{prelude::BASE64_STANDARD, Engine};
use tollkeeper::signatures::{Base64, Signed};

use crate::{
    config::{self, ServerConfig},
    data_formats::{self, AsHalJson, AsHttpHeader},
    http::{self, request::body_reader::ReadJson, server::HttpServe},
    proxy::{self},
};

pub fn create_pay_toll_endpoint(
    path: &str,
    config: ServerConfig,
    payment_service: Box<dyn PaymentService + Send + Sync>,
) -> Vec<http::server::Endpoint> {
    let pay_toll_handler = PayTollServe::new(config, payment_service);
    let pay_post_endpoint = http::server::Endpoint::new(
        http::request::Method::Post,
        path,
        Box::new(pay_toll_handler),
    );
    let pay_options_endpoint = http::server::Endpoint::new(
        http::request::Method::Options,
        path,
        Box::new(PayTollOptionsServe),
    );
    vec![pay_post_endpoint, pay_options_endpoint]
}

pub struct PayTollServe {
    config: config::ServerConfig,
    payment_service: Box<dyn PaymentService + Send + Sync>,
}
impl HttpServe for PayTollServe {
    fn serve_http(
        &self,
        client_addr: &std::net::SocketAddr,
        mut request: http::Request,
    ) -> Result<http::Response, http::server::InternalServerError> {
        let json = request.read_json().unwrap();
        let payment: Payment = serde_json::from_value(json.clone()).unwrap();
        let user_agent = request.headers().user_agent().unwrap_or("");
        let recipient = proxy::Recipient::new(
            client_addr.ip().to_string(),
            user_agent,
            payment.toll.recipient().destination(),
        );
        match self.payment_service.pay_toll(recipient, payment) {
            Ok(v) => self.create_visa_response(v),
            Err(payment_error) => Self::create_error_response(self, payment_error),
        }
    }
}
impl PayTollServe {
    pub fn new(
        config: config::ServerConfig,
        payment_service: Box<dyn PaymentService + Send + Sync>,
    ) -> Self {
        Self {
            config,
            payment_service,
        }
    }

    fn create_visa_response(
        &self,
        visa: Visa,
    ) -> Result<http::Response, http::server::InternalServerError> {
        let visa_json = visa.as_hal_json(self.config.base_url());
        let visa_json: VecDeque<u8> = visa_json.to_string().into_bytes().into();
        let mut headers = cors_headers("POST");
        headers.insert("Content-Type", "application/hal+json");
        headers.insert("Content-Length", visa_json.len().to_string());
        let headers = http::response::Headers::new(headers);
        let body = http::StreamBody::new(visa_json);
        let response = http::Response::new(
            http::response::StatusCode::OK,
            Some("OK".into()),
            headers,
            Some(Box::new(body)),
        );
        Ok(response)
    }

    fn create_error_response(
        &self,
        payment_error: Box<PaymentError>,
    ) -> Result<http::Response, http::server::InternalServerError> {
        let error_json = payment_error.as_hal_json(self.config.base_url());
        let error_json: VecDeque<u8> = error_json.to_string().into_bytes().into();
        let mut headers = cors_headers("POST");
        headers.insert("Content-Type", "application/hal+json");
        headers.insert("Content-Length", error_json.len().to_string());
        let headers = http::response::Headers::new(headers);
        let body = http::StreamBody::new(error_json);
        let status_code = match *payment_error {
            PaymentError::ChallengeFailed(_, _) => http::response::StatusCode::BadRequest,
            PaymentError::MismatchedRecipient(_, _) => http::response::StatusCode::BadRequest,
            PaymentError::InvalidSignature => http::response::StatusCode::UnprocessableContent,
            PaymentError::GatewayError => http::response::StatusCode::Conflict,
        };
        let response = http::Response::new(
            status_code,
            Some("Bad Request".into()),
            headers,
            Some(Box::new(body)),
        );
        Ok(response)
    }
}

pub trait PaymentService {
    fn pay_toll(
        &self,
        recipient: proxy::Recipient,
        payment: Payment,
    ) -> Result<Visa, Box<PaymentError>>;
}
pub struct PaymentServiceImpl {
    tollkeeper: Arc<tollkeeper::Tollkeeper>,
}

impl PaymentServiceImpl {
    pub fn new(tollkeeper: Arc<tollkeeper::Tollkeeper>) -> Self {
        Self { tollkeeper }
    }
}
impl PaymentService for PaymentServiceImpl {
    fn pay_toll(
        &self,
        recipient: proxy::Recipient,
        payment: Payment,
    ) -> Result<Visa, Box<PaymentError>> {
        let suspect = recipient.into();
        let payment = payment.try_into().unwrap();
        let visa = self.tollkeeper.pay_toll(&suspect, payment)?;
        Ok(visa.into())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Payment {
    toll: proxy::Toll,
    value: String,
}
impl Payment {
    pub fn new(toll: proxy::Toll, value: String) -> Self {
        Self { toll, value }
    }
}
impl TryFrom<Payment> for tollkeeper::SignedPayment {
    type Error = base64::DecodeError;

    fn try_from(payment: Payment) -> Result<Self, base64::DecodeError> {
        let toll: Signed<tollkeeper::declarations::Toll> = payment.toll.try_into()?;
        let payment = Self::new(toll, payment.value);
        Ok(payment)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Visa {
    order_id: proxy::OrderId,
    recipient: proxy::Recipient,
    signature: Base64,
}
impl Visa {
    pub fn new(order_id: proxy::OrderId, recipient: proxy::Recipient, signature: Base64) -> Self {
        Self {
            order_id,
            recipient,
            signature,
        }
    }

    /// Order the visa was declared for
    pub fn order_id(&self) -> &proxy::OrderId {
        &self.order_id
    }

    /// Recipient the visa was declared for
    pub fn recipient(&self) -> &proxy::Recipient {
        &self.recipient
    }

    /// Base64 encoded signature
    pub fn signature(&self) -> &Base64 {
        &self.signature
    }
}
impl data_formats::AsHttpHeader for Visa {
    fn as_http_header(&self) -> (String, String) {
        let visa_json = serde_json::json!({
            "ip": self.recipient().client_ip(),
            "ua": self.recipient().user_agent(),
            "dest": self.recipient().destination(),
            "order_id": self.order_id
        })
        .to_string();
        let visa_base64 = Base64::encode(visa_json.as_bytes());
        let header = format!("{visa_base64}.{}", self.signature);
        ("X-Keeper-Token".into(), header)
    }
}
impl data_formats::FromHttpHeader for Visa {
    type Err = ();
    fn from_http_header(value: &str) -> Result<Visa, ()> {
        let (visa, signature) = value.split_once('.').ok_or(())?;
        let visa_json = BASE64_STANDARD.decode(visa).or(Err(()))?;
        let visa_json: serde_json::Value =
            serde_json::from_slice(visa_json.as_slice()).or(Err(()))?;
        let order_id = visa_json["order_id"].as_str().ok_or(())?;
        let order_id = proxy::OrderId::from_str(order_id).or(Err(()))?;
        let client_ip = visa_json["ip"].as_str().ok_or(())?;
        let user_agent = visa_json["ua"].as_str().ok_or(())?;
        let destination = visa_json["dest"].as_str().ok_or(())?;
        let recipient = proxy::Recipient::new(client_ip, user_agent, destination);
        let signature = Base64::from(signature).or(Err(()))?;
        let visa = Visa::new(order_id, recipient, signature);
        Ok(visa)
    }
}
impl data_formats::AsHalJson for Visa {
    fn as_hal_json(&self, _: &url::Url) -> serde_json::Value {
        let origin_url = self.recipient.destination().to_string();
        let (header_name, token) = self.as_http_header();
        serde_json::json!({
            "token": token,
            "header_name": header_name,
            "_links": {
                "origin_url": origin_url
            }
        })
    }
}
impl From<Visa> for Signed<tollkeeper::declarations::Visa> {
    fn from(value: Visa) -> Self {
        let visa =
            tollkeeper::declarations::Visa::new(value.order_id.into(), value.recipient.into());
        Signed::new(visa, value.signature.decode())
    }
}
impl From<Signed<tollkeeper::declarations::Visa>> for Visa {
    fn from(value: Signed<tollkeeper::declarations::Visa>) -> Self {
        let (signature, visa) = value.deconstruct();
        Visa::new(
            visa.order_id().into(),
            visa.suspect().into(),
            signature.base64(),
        )
    }
}

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub enum PaymentError {
    ChallengeFailed(proxy::Toll, String),
    MismatchedRecipient(proxy::Recipient, proxy::Toll),
    InvalidSignature,
    GatewayError,
}
impl PaymentError {
    fn challenge_failed_json(
        message: &str,
        toll: &proxy::Toll,
        failed_payment: &str,
        base_url: &url::Url,
    ) -> serde_json::Value {
        serde_json::json!({
            "error": "Challenge failed!",
            "message": message,
            "failed_payment": failed_payment,
            "new_toll": toll.as_hal_json(base_url)
        })
    }

    fn mismatched_recipient_json(
        message: &str,
        expected_recipient: &proxy::Recipient,
        toll: &proxy::Toll,
        base_url: &url::Url,
    ) -> serde_json::Value {
        serde_json::json!({
            "error": "Mismatched Recipient!",
            "message": message,
            "expected_recipient": expected_recipient,
            "new_toll": toll.as_hal_json(base_url)
        })
    }

    fn invalid_signature(message: &str) -> serde_json::Value {
        serde_json::json!({
            "error": "Invalid Signature!",
            "message": message,
        })
    }

    fn gateway_error(message: &str) -> serde_json::Value {
        serde_json::json!({
            "error": "Gateway Error!",
            "message": message,
        })
    }
}
impl Error for PaymentError {}
impl Display for PaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentError::ChallengeFailed(_, failed_payment)
                => write!(f, "'{failed_payment}' was not the right answer! Try again with new toll"),
            PaymentError::MismatchedRecipient(_, _)
                => write!(f, "Toll was issued for a different recipient. New toll issued for current recipient"),
            PaymentError::InvalidSignature => write!(f, "Issued toll signature is not valid! Content was probably modified or the key rotated"),
            PaymentError::GatewayError => write!(f, "Toll no longer matches any order. Retry request"),
        }
    }
}
impl data_formats::AsHalJson for PaymentError {
    fn as_hal_json(&self, base_url: &url::Url) -> serde_json::Value {
        match self {
            PaymentError::ChallengeFailed(toll, failed_payment) => {
                Self::challenge_failed_json(&self.to_string(), toll, failed_payment, base_url)
            }
            PaymentError::MismatchedRecipient(recipient, toll) => {
                Self::mismatched_recipient_json(&self.to_string(), recipient, toll, base_url)
            }
            PaymentError::InvalidSignature => Self::invalid_signature(&self.to_string()),
            PaymentError::GatewayError => Self::gateway_error(&self.to_string()),
        }
    }
}
impl From<tollkeeper::err::PaymentDeniedError> for Box<PaymentError> {
    fn from(value: tollkeeper::err::PaymentDeniedError) -> Self {
        let error = match value {
            tollkeeper::err::PaymentDeniedError::InvalidPayment(e) => {
                PaymentError::ChallengeFailed(e.new_toll().into(), e.payment().value().into())
            }
            tollkeeper::err::PaymentDeniedError::MismatchedSuspect(e) => {
                PaymentError::MismatchedRecipient(e.expected().into(), e.new_toll().into())
            }
            tollkeeper::err::PaymentDeniedError::InvalidSignature => PaymentError::InvalidSignature,
            tollkeeper::err::PaymentDeniedError::GatewayError(_) => PaymentError::GatewayError,
        };
        Box::new(error)
    }
}

//TODO: Handle OPTIONS more elegantly
struct PayTollOptionsServe;
impl HttpServe for PayTollOptionsServe {
    fn serve_http(
        &self,
        _: &std::net::SocketAddr,
        _: http::Request,
    ) -> Result<http::Response, http::server::InternalServerError> {
        let mut headers = cors_headers("POST");
        headers.insert("Accept", "application/json");
        headers.insert("Allow", "POST");
        let headers = http::response::Headers::new(headers);
        let response =
            http::Response::new(http::response::StatusCode::NoContent, None, headers, None);
        Ok(response)
    }
}

fn cors_headers(methods: impl Into<String>) -> http::Headers {
    let mut headers = http::Headers::empty();
    headers.insert("Access-Control-Allow-Headers", "*");
    headers.insert("Access-Control-Allow-Methods", methods);
    headers.insert("Access-Control-Allow-Origin", "*");
    headers
}
