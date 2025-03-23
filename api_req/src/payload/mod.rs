//! Payload

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use super::error::ApiErr;
pub use api_req_derive::{ApiCaller, Payload};
use pin_project::pin_project;
use reqwest::{Client, Method, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::LazyLock;

/// Define a API that can be called
///
/// # Example
/// ```
/// use api_req::Payload;
/// use serde::Serialize;
///
/// #[derive(Debug, Clone, Serialize, Payload)]
/// #[payload(
///     path = "/api/v1/{payment_id}",
///     method = "GET",
///     headers = (("k1", "v1"),)   // headers added to the default headers
/// )]
/// pub struct CompletePayload {
///     #[serde(skip_serializing)]
///     payment_id: String,
/// }
/// ```
pub trait Payload: Send + Sync + Serialize + 'static {
    /// The method of the API
    const METHOD: &'static str;

    /// The headers for the API.
    fn headers(&self) -> Option<HeaderMap> {
        None
    }

    /// The path for the API.
    fn path(&self) -> Option<String> {
        None
    }
}

/// Define a API caller
///
/// # Example
/// ```
/// use api_req::ApiCaller;
/// use reqwest::header;
///
/// #[derive(ApiCaller)]
/// #[api(
///     base_url = "http://example.com",
///     default_headers = (("k1", "v1"), (header::ORIGIN, "v2")),
///     default_headers_env = (("k3", "API_KEY"),)  // header value from env; `,` is essential
/// )]
/// struct ExampleApi;
/// ```
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
        static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
        CLIENT.clone()
    }
}

/// A request to the API, wrapping the payload into a future.
///
/// Create from a ApiCaller and a Payload.
///
/// # Example
/// ```no_run
/// use api_req::{Payload, ApiCaller, ApiCaller as _};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Default, Clone, Serialize, Payload)]
/// #[payload(path = "/payments/{customer_id}", method = "POST")]
/// pub struct ExamplePayload {
///     #[serde(skip_serializing)]
///     customer_id: String,    // this field is passed as a path parameter
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
                match P::METHOD.try_into() {
                    Ok(method) => {
                        let mut url = format!("{}{}", base_url, payload.path().unwrap_or_default());
                        match method {
                            Method::POST => {
                                let mut req = client.request(method, url).json(&payload);
                                if let Some(headers) = payload.headers() {
                                    req = req.headers(headers);
                                }
                                let resp = req.send().await?;
                                let text = resp.text().await?;
                                let output =
                                    serde_json::from_str(&text).map_err(|_| ApiErr::Serde(text))?;
                                Ok::<_, ApiErr>(output)
                            }
                            Method::GET => {
                                let query = serde_urlencoded::to_string(&payload)
                                    .expect("Payload should be urlencode-serializable");
                                if !query.is_empty() {
                                    url.push_str(&format!("?{}", query));
                                }
                                let mut req = client.request(method, url);
                                if let Some(headers) = payload.headers() {
                                    req = req.headers(headers);
                                }
                                let resp = req.send().await?;
                                let text = resp.text().await?;
                                let output =
                                    serde_json::from_str(&text).map_err(|_| ApiErr::Serde(text))?;
                                Ok::<_, ApiErr>(output)
                            }
                            _ => Err(ApiErr::Other(format!("Unsupported method: {}", P::METHOD))),
                        }
                    }
                    Err(_) => Err(ApiErr::Other(format!("Invalid method: {}", P::METHOD))),
                }
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
        const METHOD: &'static str = "POST";

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
