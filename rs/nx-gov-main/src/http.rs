//! The data structure for HTTP requests and responses as supported natively by the replica.
//!
//! This file is taken from [ic-eth-wallet](https://github.com/dfinity/ic-eth-wallet) which is licensed under Apache-2.0.

use crate::metrics::get_metrics;

use candid::{CandidType, Deserialize};
use ic_cdk_macros::query;
use serde_bytes::ByteBuf;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    /// The HTTP method of the request, such as `GET` or `POST`.
    pub method: String,
    /// The requested path and query string, for example `/some/path?foo=bar`.
    ///
    /// Note: This does NOT contain the domain, port or protocol.
    pub url: String,
    /// The HTTP request headers
    pub headers: Vec<HttpHeaderField>,
    /// The complete body of the HTTP request
    pub body: ByteBuf,
}

pub type HttpHeaderField = (String, String);

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HttpHeaderField>,
    pub body: ByteBuf,
}

/// Processes external HTTP requests.
#[query]
pub fn http_request(request: HttpRequest) -> HttpResponse {
    let parts: Vec<&str> = request.url.split('?').collect();
    match parts[0] {
        "/metrics" => get_metrics(),
        _ => HttpResponse {
            status_code: 404,
            headers: vec![],
            body: ByteBuf::from(String::from("Not found.")),
        },
    }
}
