//! Payload

use crate::error::{ApiErr, ApiResult};
pub use api_req_derive::{ApiCaller, Payload};
use reqwest::{Method, RequestBuilder, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};

/// Define a Payload that can be made into a Request by ApiCaller.
///
/// # Example
/// ```
/// use api_req::{Payload, Method};
/// use serde::Serialize;
///
/// #[derive(Debug, Clone, Serialize, Payload)]
/// #[api_req(
///     path = "/api/v1/{payment_id}",  // format `payment_id` from struct field
///     method = Method::GET,
///     // headers added to the default headers
///     headers = [("k1", "v1"), ("header", "{header}")],  // format `header` from struct field
///     req = query,    // `RequestBuilder::query` will be used; Can also be `json`, `form` as you need
///     // strip the prefix before deserialize; `F: FnOnce(String) -> Result<String, String>`
///     before_deserialize = |text: String| text.strip_prefix("&&&START&&&").map(ToOwned::to_owned).ok_or(text),
///     deserialize = serde_urlencoded::from_str    // use `serde_urlencoded` to deserialize the response body
/// )]
/// pub struct CompletePayload {
///     #[serde(skip_serializing)]
///     payment_id: String,
///     #[serde(skip_serializing)]
///     header: String,
/// }
/// ```
pub trait Payload: Send + Sync + Serialize + 'static {
    /// The method of the API; GET or POST;
    const METHOD: Method;

    /// The headers for the API.
    fn headers(&self) -> Option<HeaderMap> {
        None
    }

    /// The path for the API.
    fn path(&self) -> Option<String> {
        None
    }

    /// add options to RequestBuilder: headers, body, query...
    fn req_option(&self, mut req: RequestBuilder) -> RequestBuilder {
        if let Some(headers) = self.headers() {
            req = req.headers(headers);
        }
        match Self::METHOD {
            Method::GET => req.query(self),
            Method::POST => req.json(self),
            _ => unimplemented!(),
        }
    }

    /// before deserialize response's body
    fn before_deserialize() -> Option<fn(String) -> ApiResult<String>> {
        None
    }

    /// deserialize
    fn deserialize<O: DeserializeOwned>(input: String) -> ApiResult<O> {
        serde_json::from_str(&input).map_err(|e| ApiErr::UnDeserializeable(e.to_string()))
    }
}
