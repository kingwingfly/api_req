//! Caller of the API

use std::sync::LazyLock;

use reqwest::{Client, redirect::Policy};
use serde::de::DeserializeOwned;

use crate::{Payload, Request};

/// Define a API caller
///
/// # Example
/// ```
/// use api_req::{ApiCaller, RedirectPolicy};
/// use reqwest::header;
///
/// #[derive(ApiCaller)]
/// #[api(
///     base_url = "http://example.com",
///     default_headers = (("k1", "v1"), (header::ORIGIN, "v2")),
///     default_headers_env = (("k3", "API_KEY"),),  // header value from env; `,` is essential in tuple
///     redirect = RedirectPolicy::none()
/// )]
/// struct ExampleApi;
/// ```
///
/// Provide the root URL in `protocol://domain[:port]` format (e.g., `https://example.com`) as base_url.
///
/// Valid: `https://api.service.com`, `http://localhost:8080`.
///
/// Invalid: `https://example.com/api` will be treated as `https://example.com`.
pub trait ApiCaller {
    /// The baseurl of the API
    const BASE_URL: &'static str;

    /// return a request future that can be awaited
    fn request<P, O>(payload: P) -> Request<P, O, ()>
    where
        P: Payload,
        O: DeserializeOwned,
    {
        Request::new(payload, Self::BASE_URL.to_string(), Self::client())
    }

    /// return a client
    fn client() -> Client {
        static CLIENT: LazyLock<Client> =
            LazyLock::new(|| Client::builder().redirect(Policy::none()).build().unwrap());
        CLIENT.clone()
    }
}
