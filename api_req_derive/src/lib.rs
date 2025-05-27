#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod api_caller;
mod payload;

use proc_macro::TokenStream;

/// Derive the `Api` trait
///
/// # Example
/// ```
/// use api_req_derive::Payload;
/// use serde::Serialize;
/// use reqwest::Method;
///
/// #[derive(Payload, Serialize)]
/// #[api_req(
///     path = "/api/v1/{id}",  // format `id` from struct field
///     method = Method::GET,
///     headers = (("k1", "v1"), ("k2", "{x}")) // format `x` from struct field
/// )]
/// struct Test {
///     #[serde(skip_serializing)]
///     id: u32,
///     #[serde(skip_serializing)]
///     x: String,
/// }
/// ```
///
/// If `method` is set to POST, one should set `req` to `json` or `form` as needed manually.
///
/// The default derive code is `GET` method with `query` request.
#[proc_macro_derive(Payload, attributes(api_req))]
pub fn derive_payload(input: TokenStream) -> TokenStream {
    payload::derive_payload(input)
}

/// Derive the `Api` trait
///
/// # Example
/// ```
/// use api_req_derive::ApiCaller;
/// use reqwest::header;
///
/// #[derive(ApiCaller)]
/// #[api_req(
///     base_url = "http://example.com",
///     default_headers = (("k1", "v1"), (header::ORIGIN, "v2")),
///     default_headers_env = (("k3", "API_KEY"),)  // `,` is essential
/// )]
/// struct ExampleApi;
/// ```
#[proc_macro_derive(ApiCaller, attributes(api_req))]
pub fn derive_api_caller(input: TokenStream) -> TokenStream {
    api_caller::derive_api_caller(input)
}
