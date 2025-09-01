#[cfg(test)]
mod tests;

use std::{error::Error, fmt::Display};

use tollkeeper::signatures::Signed;

use crate::{
    http::{self, server::HttpServe},
    proxy::{self, Recipient},
};

pub fn create_pay_toll_endpoint(path: &str) -> http::server::Endpoint {
    let method = http::request::Method::Post;
    todo!()
}

pub struct PayTollServe {}
impl HttpServe for PayTollServe {
    fn serve_http(
        &self,
        client_addr: &std::net::SocketAddr,
        request: http::Request,
    ) -> Result<http::Response, http::server::InternalServerError> {
        //TODO:
        // - read json from body and turn into Payment struct (extension method for bodies?)
        // - convert Payment dto into entity for Tollkeeper
        // - run tollkeeper payment process
        // - on success, return 200 OK with the X-Keeper-Token inside JSON (name property same as header? just
        //   include header name as additional property? Its fixed, but still nice as documentation)
        // - on failure, return 4xx code and the new toll
        todo!()
    }
}

pub trait PaymentService {
    fn pay_toll(
        &self,
        recipient: proxy::Recipient,
        payment: Payment,
    ) -> Result<proxy::Visa, PaymentError>;
}

#[derive(serde::Serialize, Debug, Eq, PartialEq, Clone)]
pub struct Payment {
    toll: proxy::Toll,
    value: String,
}
impl From<Payment> for tollkeeper::SignedPayment {
    fn from(payment: Payment) -> Self {
        let toll: Signed<tollkeeper::declarations::Toll> = payment.toll.into();
        Self::new(toll, payment.value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PaymentError {
    ChallengeFailed,
    MismatchedRecipient,
    InvalidSignature,
    GatewayError,
}
impl Error for PaymentError {}
impl Display for PaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Payment error") //TODO: Better error message
    }
}
