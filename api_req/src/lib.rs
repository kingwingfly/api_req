#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod error;
mod payload;

pub use payload::{ApiCaller, Payload, Request};
#[doc(hidden)]
pub use reqwest::Client as __reqwest_Client;
pub use reqwest::{Method, RequestBuilder, header, redirect::Policy as RedirectPolicy};
#[doc(hidden)]
pub use serde as __serde;
#[doc(hidden)]
pub use serde_json as __serde_json;
