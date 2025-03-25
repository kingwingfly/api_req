use proc_macro::TokenStream;
use quote::{ToTokens as _, quote};
use regex::Regex;
use syn::{DeriveInput, Expr, ExprTuple, Ident, LitStr, parse_macro_input, parse2};

pub(crate) fn derive_payload(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut path: Expr = syn::parse_str("\"\"").unwrap();
    let mut method: Expr = syn::parse_str("::api_req::Method::GET").unwrap();
    let mut headers_key: Vec<Expr> = vec![];
    let mut headers_value: Vec<Expr> = vec![];
    let mut req: Ident = syn::parse_str("json").unwrap();
    let mut before_deserialize: Option<Expr> = None;
    let mut deserialize: Option<Expr> = None;
    let re = Regex::new(r"\{(\w+)\}").unwrap();

    if let Some(attr) = input
        .attrs
        .iter()
        .find(|&attr| attr.path().is_ident("api_req"))
    {
        attr.parse_nested_meta(|meta| {
            match &meta.path {
                item if item.is_ident("path") => {
                    let value = meta.value()?;
                    let p: LitStr = value.parse().unwrap();
                    let format_args = re
                        .captures_iter(&p.value())
                        .map(|c| c.extract::<1>())
                        .map(|(_, cap)| cap[0].to_string())
                        .collect::<Vec<_>>();
                    let p = format_args
                        .into_iter()
                        .fold(format!(r#""{}""#, p.value()), |acc, x| {
                            format!("{acc}, {x}=self.{x}", acc = acc, x = x)
                        });
                    let p = format!("format!({})", p);
                    path = syn::parse_str(&p)
                        .map_err(|_| format!("cannot convert: {}", p))
                        .unwrap();
                }
                item if item.is_ident("method") => {
                    let value = meta.value()?;
                    method = value.parse().unwrap();
                }
                item if item.is_ident("headers") => {
                    let value = meta.value()?;
                    let kvs: ExprTuple = value.parse().unwrap();
                    for kv in kvs.elems {
                        let kv: ExprTuple = parse2(kv.into_token_stream()).unwrap();
                        let mut kv = kv.elems.into_iter();
                        headers_key.push(kv.next().unwrap());
                        let mut value = kv.next().unwrap();
                        if let Ok(v) = parse2::<LitStr>(value.clone().into_token_stream()) {
                            let format_args = re
                                .captures_iter(&v.value())
                                .map(|c| c.extract::<1>())
                                .map(|(_, cap)| cap[0].to_string())
                                .collect::<Vec<_>>();
                            let v = format_args
                                .into_iter()
                                .fold(format!(r#""{}""#, v.value()), |acc, x| {
                                    format!("{acc}, {x}=self.{x}", acc = acc, x = x)
                                });
                            let v = format!("format!({})", v);
                            value = syn::parse_str(&v)
                                .map_err(|_| format!("cannot convert: {}", v))
                                .unwrap();
                        }
                        headers_value.push(value);
                    }
                }
                item if item.is_ident("req") => {
                    let value = meta.value()?;
                    req = value.parse().unwrap();
                }
                item if item.is_ident("before_deserialize") => {
                    let value = meta.value()?;
                    before_deserialize = Some(value.parse().unwrap());
                }
                item if item.is_ident("deserialize") => {
                    let value = meta.value()?;
                    deserialize = value.parse().ok();
                }
                item => Err(meta.error(format!(
                    "unsupported attribute: {}",
                    item.get_ident().unwrap()
                )))?,
            }
            Ok(())
        })
        .unwrap();
    };

    let headers = if headers_key.is_empty() {
        quote! { None }
    } else {
        quote! {
            let mut headers = ::api_req::header::HeaderMap::new();
            #(
                headers.insert(#headers_key, #headers_value.parse().unwrap());
            )*
            Some(headers)
        }
    };

    let before_deserialize_expanded = match before_deserialize {
        Some(expr) => quote! {
            Some(#expr.map_err(|e| ::api_req::error::ApiErr::Other(format!("Before deserialize failed:\n{}", e))))
        },
        None => quote! {
            None
        },
    };

    let deserialize_expanded = match deserialize {
        Some(expr) => quote! { #expr },
        None => quote! { ::api_req::__serde_json::from_str },
    };

    let expanded = quote! {
        impl #impl_generics ::api_req::Payload for #name #ty_generics #where_clause {
            const METHOD: ::api_req::Method = #method;

            fn headers(&self) -> Option<::api_req::header::HeaderMap> {
                #headers
            }

            fn path(&self) -> Option<String> {
                Some(#path)
            }

            fn req_option(&self, mut req: ::api_req::RequestBuilder) -> ::api_req::RequestBuilder {
                if let Some(headers) = self.headers() {
                    req = req.headers(headers);
                }
                req.#req(self)
            }

            fn before_deserialize() -> Option<fn(String) -> ::api_req::error::ApiResult<String>> {
                #before_deserialize_expanded
            }

            fn deserialize<O: ::api_req::__serde::de::DeserializeOwned>(input: String) -> ::api_req::error::ApiResult<O> {
                #deserialize_expanded(&input).map_err(|_| ::api_req::error::ApiErr::UnDeserializeable(input))
            }
        }
    };

    TokenStream::from(expanded)
}
