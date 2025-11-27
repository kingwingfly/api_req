use api_req::{ApiCaller, Method, Payload, RedirectPolicy, header};
use serde::Serialize;

#[derive(Debug, ApiCaller)]
#[api_req(
    base_url = "https://api.micoapi.com",
    default_headers = [(header::USER_AGENT, "Mozilla/5.0")],
    redirect = RedirectPolicy::none(),
)]
pub struct Api {}

#[derive(Debug, Serialize, Payload)]
#[api_req(
    path = "/v2/auth/{id}",
    method = Method::POST,
    headers = [(header::AUTHORIZATION, "{password}")],
    req = form,
    before_deserialize = |text: String| text.strip_prefix("&&&START&&&").map(ToOwned::to_owned).ok_or(text),
    deserialize = serde_urlencoded::from_str,
)]
pub struct LoginPayload {
    #[serde(skip_serializing)]
    pub id: String,
    pub email: String,
    pub password: String,
}

#[test]
fn test() {}
