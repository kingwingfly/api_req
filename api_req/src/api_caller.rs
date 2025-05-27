//! Caller of the API

use std::sync::LazyLock;

use reqwest::{Client, redirect::Policy};
use serde::de::DeserializeOwned;

use crate::{Payload, Request};

/// The cookies jar all callers' using.
///
/// Different API callers use different base_urls (domains), so a static global cookie jar is more
/// suitable than many jars for each caller.
#[cfg(feature = "cookies")]
pub static COOKIE_JAR: LazyLock<std::sync::Arc<reqwest::cookie::Jar>> =
    LazyLock::new(|| std::sync::Arc::new(reqwest::cookie::Jar::default()));

/// Define a API caller
///
/// # Example
/// ```
/// use api_req::{ApiCaller, RedirectPolicy};
/// use reqwest::header;
///
/// #[derive(ApiCaller)]
/// #[api_req(
///     base_url = "http://example.com",
///     default_headers = (("k1", "v1"), (header::ORIGIN, "v2")),
///     default_headers_env = (("k3", "API_KEY"),),  // header value from env; `,` is essential in tuple
///     redirect = RedirectPolicy::none()
/// )]
/// struct ExampleApi;
/// ```
///
/// Provide the root URL in `protocol://domain[:port]/api/` format (e.g., `https://example.com/api/`) as base_url.
///
/// Valid: `https://api.service.com`, `http://localhost:8080/api/`.
///
/// Invalid: `https://example.com/api` will be treated as `https://example.com`, use `https://example.com/api/` instead.
pub trait ApiCaller {
    /// The baseurl of the API, mostly the domain
    const BASE_URL: &'static str;

    /// Return a request future that can be awaited
    fn request<P, O>(payload: P) -> Request<P, O, ()>
    where
        P: Payload,
        O: DeserializeOwned,
    {
        Request::new(payload, Self::BASE_URL.to_string(), Self::client())
    }

    /// Return a stream future that can be awaited
    #[cfg(feature = "stream")]
    fn stream<P>(payload: P) -> Request<P, crate::RespStream, ((),)>
    where
        P: Payload,
    {
        Request::new(payload, Self::BASE_URL.to_string(), Self::client())
    }

    /// Return a client
    fn client() -> Client {
        static CLIENT: LazyLock<Client> = LazyLock::new(|| {
            let builder = Client::builder().redirect(Policy::none());
            #[cfg(feature = "cookies")]
            let builder = builder.cookie_provider(COOKIE_JAR.clone());
            builder.build().unwrap()
        });
        CLIENT.clone()
    }
}
