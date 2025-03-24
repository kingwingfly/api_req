Make API calls more easier!

# Advantage
For example:
```rust ignore
use api_req::Method;

#[derive(Debug, Default, Clone, Serialize, Payload)]
#[payload(
    path = "/payments/{customer_id}", // format from struct fields
    method = Method::POST,
    ...
)]
pub struct ExamplePayload {
    #[serde(skip_serializing)]
    customer_id: String,
    amount: usize,
}
```
You can not only define the path, method, payload-format, pre-deserialize-action, and deserialize method,
but also can format the path, headervalue with fields in the payload struct.

# feature

- `stream` - support stream response: `RespStream`

# Example
```rust no_run
use api_req::{header, Payload, RedirectPolicy, ApiCaller, Method, ApiCaller as _};
use serde::{Serialize, Deserialize};
#[derive(Debug, Default, Clone, Serialize, Payload)]
#[payload(
    path = "/payments/{customer_id}",
    method = Method::POST,
    headers = ((header::AUTHORIZATION, "Bearer token {bearer_token}"),),
    req = form,  // use `form` to set body format instead of the default `json`
    before_deserialize = |text: String | text.strip_prefix("START: ").map(ToOwned::to_owned).ok_or(text),
    deserialize = serde_urlencoded::from_str,
)]
pub struct ExamplePayload {
    #[serde(skip_serializing)]  // skip this field when serializing payload
    customer_id: String,
    #[serde(skip_serializing)]
    bearer_token: String,
    amount: usize,
}

#[derive(Debug, Deserialize)]
struct ExampleResponse {
    client_secret: String,
}

#[derive(ApiCaller)]
#[api(
    base_url = "http://example.com",
    default_headers = ((header::USER_AGENT, "..."),),
    default_headers_env = (("api-key", "API_KEY"),),    // get from env
    redirect = RedirectPolicy::none()   // set redirect policy
)]
struct ExampleApi;

# async {
let payload = ExamplePayload::default();
let _resp: ExampleResponse = ExampleApi::request(payload).await.unwrap();
# };
// this will send a POST request to http://example.com/payments/{customer_id}
// with json `{"amount": 0}`
```

For POST request, the payload will be serialized as json body by default.

For GET request, the payload will be serialized as query parameters (urlencoded) by default.

You can set the payload format by `req` attribute in the `#[payload(...)]` attribute.

For other methods, not supported yet.
