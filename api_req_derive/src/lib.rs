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
///
/// #[derive(Payload, Serialize)]
/// #[payload(path = "/api/v1/test", method = "GET")]
/// struct Test;
/// ```
#[proc_macro_derive(Payload, attributes(payload))]
pub fn derive_payload(input: TokenStream) -> TokenStream {
    payload::derive_payload(input)
}

/// Derive the `Api` trait
///
/// # Example
/// ```
/// use api_req_derive::ApiCaller;
///
/// #[derive(ApiCaller)]
/// #[api(base_url = "http://example.com")]
/// struct ExampleApi;
/// ```
#[proc_macro_derive(ApiCaller, attributes(api))]
pub fn derive_api_caller(input: TokenStream) -> TokenStream {
    api_caller::derive_api_caller(input)
}
