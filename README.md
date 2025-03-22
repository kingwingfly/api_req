Make API calls more easier
# Advantage
For example:
```rust ignore
#[derive(Debug, Default, Clone, Serialize, Payload)]
#[payload(path = "/payments/{customer_id}", method = "POST")]
pub struct ExamplePayload {
    #[serde(skip_serializing)]
    customer_id: String,    // this field is passed as a path parameter, not in the json body
    amount: usize,
}
```
You can not only define the method for each payload,
but also define the path together with fields in the payload.
# Example
```rust no_run
use api_req::{Payload, ApiCaller, ApiCaller as _};
use serde::{Serialize, Deserialize};
#[derive(Debug, Default, Clone, Serialize, Payload)]
#[payload(path = "/payments/{customer_id}", method = "POST")]
pub struct ExamplePayload {
    #[serde(skip_serializing)]
    customer_id: String,    // this field is passed as a path parameter
    amount: usize,
}
#[derive(Debug, Deserialize)]
struct ExampleResponse {
    client_secret: String,
}
#[derive(ApiCaller)]
#[api(base_url = "http://example.com")]
struct ExampleApi;
# async {
let payload = ExamplePayload::default();
let _resp: ExampleResponse = ExampleApi::request(payload).await.unwrap();
# };
// this will send a POST request to http://example.com/payments/{customer_id}
// with json `{"amount": 100}`
```
For POST request, the payload will be serialized as json body.

For GET request, the payload will be serialized as query parameters (urlencoded).

For other methods, not supported yet.
