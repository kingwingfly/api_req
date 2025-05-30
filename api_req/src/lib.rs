#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod api_caller;
pub mod error;
pub mod payload;
pub mod request;

pub use api_caller::ApiCaller;
#[cfg(feature = "cookies")]
pub use api_caller::COOKIE_JAR;
#[cfg(feature = "stream")]
pub use futures_util::StreamExt;
pub use payload::{ApiCaller, Payload};
pub use request::Request;
#[cfg(feature = "stream")]
pub use request::RespStream;
#[doc(hidden)]
pub use reqwest::Client as __reqwest_Client;
#[cfg(feature = "cookies")]
pub use reqwest::cookie::CookieStore;
pub use reqwest::{Method, RequestBuilder, header, redirect::Policy as RedirectPolicy};
#[doc(hidden)]
pub use serde as __serde;
#[doc(hidden)]
pub use serde_json as __serde_json;
