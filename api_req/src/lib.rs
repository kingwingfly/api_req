#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod error;
mod payload;

pub use payload::{ApiCaller, Payload, Request};
pub use reqwest::{Client, header, redirect::Policy as RedirectPolicy};
