//! Payload

use crate::error::{ApiErr, ApiResult};
pub use api_req_derive::{ApiCaller, Payload};
use pin_project::pin_project;
use reqwest::{Client, Method, RequestBuilder, Url, header::HeaderMap, redirect::Policy};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::LazyLock;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// Define a API that can be called
///
/// # Example
/// ```
/// use api_req::{Payload, Method};
/// use serde::Serialize;
///
/// #[derive(Debug, Clone, Serialize, Payload)]
/// #[payload(
///     path = "/api/v1/{payment_id}",  // format `payment_id` from struct field
///     method = Method::GET,
///     // headers added to the default headers
///     headers = (("k1", "v1"), ("header", "{header}")),  // format `header` from struct field
///     req = query,    // `RequestBuilder::query` will be used; Can also be `json`, `form` as you need
///     // strip the prefix before deserialize
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

    /// befor deserialize response's body
    fn before_deserialize() -> Option<fn(String) -> ApiResult<String>> {
        None
    }

    /// deserialize
    fn deserialize<O: DeserializeOwned>(input: String) -> ApiResult<O> {
        serde_json::from_str(&input).map_err(|_| ApiErr::UnDeserializeable(input))
    }
}

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
    fn request<P, O>(payload: P) -> Request<P, O>
    where
        P: Payload,
        O: DeserializeOwned + Send + Sync + 'static,
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

/// A request to the API, wrapping the payload into a future.
///
/// Create from a ApiCaller and a Payload.
///
/// # Example
/// ```no_run
/// use api_req::{Payload, ApiCaller, Method, ApiCaller as _};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Default, Clone, Serialize, Payload)]
/// #[payload(
///     path = "/payments/{customer_id}",   // customer_id from struct field
///     method = Method::POST,
/// )]
/// pub struct ExamplePayload {
///     #[serde(skip_serializing)]
///     customer_id: String,
///     amount: usize,
/// }
///
/// #[derive(Debug, Deserialize)]
/// struct ExampleResponse {
///     client_secret: String,
/// }
///
/// #[derive(ApiCaller)]
/// #[api(base_url = "http://example.com")]
/// struct ExampleApi;
/// # async {
/// let payload = ExamplePayload::default();
/// let _resp: ExampleResponse = ExampleApi::request(payload).await.unwrap();
/// # };
/// // this will send a POST request to http://example.com/payments/{customer_id}
/// // with json `{"amount": 100}`
/// ```
#[allow(clippy::type_complexity)]
#[pin_project]
pub struct Request<P, O>
where
    P: Payload,
    O: DeserializeOwned + Send + Sync + 'static,
{
    client: Client,
    base_url: String,
    payload: Option<P>,
    future: Option<Pin<Box<dyn Future<Output = Result<O, ApiErr>> + Send + Sync + 'static>>>,
}

impl<P, O> Request<P, O>
where
    P: Payload,
    O: DeserializeOwned + Send + Sync + 'static,
{
    /// Create a new request
    pub fn new(payload: P, base_url: String, client: Client) -> Self {
        Self {
            client,
            base_url,
            payload: Some(payload),
            future: None,
        }
    }
}

impl<P, O> Future for Request<P, O>
where
    P: Payload,
    O: DeserializeOwned + Send + Sync + 'static,
{
    type Output = Result<O, ApiErr>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if this.payload.is_some() {
            let payload = this.payload.take().unwrap();
            let client = this.client.clone();
            let base_url = this.base_url.drain(..).collect::<String>();
            let future = Box::pin(async move {
                let mut req = client.request(
                    P::METHOD,
                    Url::parse(&base_url)
                        .unwrap()
                        .join(&payload.path().unwrap_or_default())
                        .unwrap(),
                );
                req = payload.req_option(req);
                let resp = req.send().await?;
                let mut body = resp.text().await?;
                if let Some(pre_op) = P::before_deserialize() {
                    body = pre_op(body)?;
                }
                P::deserialize(body)
            });
            *this.future = Some(future);
        }
        let future = this.future.as_mut().unwrap().as_mut();
        future.poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize)]
    struct TestPayload {
        amount: usize,
        currency: String,
        payment_link: bool,
        profile_id: String,
    }

    impl Payload for TestPayload {
        const METHOD: Method = Method::POST;

        fn path(&self) -> Option<String> {
            Some("/payments".to_string())
        }
    }

    struct HyperApi;

    impl ApiCaller for HyperApi {
        const BASE_URL: &'static str = "https://sandbox.hyperswitch.io";
    }

    #[tokio::test]
    #[ignore = "This test will send a request to the sandbox server"]
    async fn test_request() {
        let payload = TestPayload {
            amount: 100,
            currency: "CNY".to_string(),
            payment_link: true,
            profile_id: "pro_EfLFMGhHt4aaUsg9rAnL".to_string(),
        };
        let request = HyperApi::request(payload);
        let response: serde_json::Value = request.await.unwrap();
        println!("{:#?}", response);
    }
}
