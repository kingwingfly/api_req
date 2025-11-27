//! Request from `ApiCaller` and `Payload`, a future that can be awaited to get the response

use std::{
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "stream")]
use futures_util::{Stream, TryStreamExt as _};
use pin_project::pin_project;
use reqwest::{Client, Url};
use serde::de::DeserializeOwned;

use crate::{Payload, error::ApiErr};

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
/// #[api_req(
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
/// #[api_req(base_url = "http://example.com")]
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
#[must_use = "futures do nothing unless polled"]
pub struct Request<P, O, M>
where
    P: Payload,
{
    client: Client,
    base_url: String,
    payload: Option<P>,
    future: Option<Pin<Box<dyn Future<Output = Result<O, ApiErr>> + Send + Sync + 'static>>>,
    _marker: std::marker::PhantomData<M>,
}

impl<P, O, M> Request<P, O, M>
where
    P: Payload,
{
    /// Create a new request
    pub fn new(payload: P, base_url: String, client: Client) -> Self {
        Self {
            client,
            base_url,
            payload: Some(payload),
            future: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<P, O> Future for Request<P, O, ()>
where
    P: Payload,
    O: DeserializeOwned,
{
    type Output = Result<O, ApiErr>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Some(payload) = this.payload.take() {
            let client = this.client.clone();
            let base_url = this.base_url.drain(..).collect::<String>();
            let future = Box::pin(async move {
                let mut req = client.request(
                    P::METHOD,
                    Url::parse(&base_url)
                        .and_then(|base| base.join(&payload.path().unwrap_or_default()))
                        .map_err(|e| ApiErr::Other(e.to_string()))?,
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

/// `Bytes` response stream name alias
///
/// `api_req` re-export `StreamExt` from `futures-util`.
#[cfg(feature = "stream")]
pub type RespStream = Box<dyn Stream<Item = Result<bytes::Bytes, ApiErr>> + Send>;

#[cfg(feature = "stream")]
impl<P> Future for Request<P, RespStream, ((),)>
where
    P: Payload,
{
    type Output = Result<RespStream, ApiErr>;

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
                let stream = resp.bytes_stream().err_into();
                Ok(Box::new(stream) as RespStream)
            });
            *this.future = Some(future);
        }
        let future = this.future.as_mut().unwrap().as_mut();
        future.poll(cx)
    }
}
