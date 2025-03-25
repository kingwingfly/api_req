use proc_macro::TokenStream;
use quote::{ToTokens as _, quote};
use syn::{DeriveInput, Expr, ExprTuple, parse_macro_input, parse2};

pub(crate) fn derive_api_caller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut base_url: Option<Expr> = None;
    let mut default_headers_key: Vec<Expr> = vec![];
    let mut default_headers_value: Vec<Expr> = vec![];
    let mut default_headers_env_key: Vec<Expr> = vec![];
    let mut default_headers_env_value: Vec<Expr> = vec![];
    let mut redirect: Option<Expr> = None;

    if let Some(attr) = input.attrs.iter().find(|&attr| attr.path().is_ident("api_req")) {
        attr.parse_nested_meta(|meta| {
            match &meta.path {
                item if item.is_ident("base_url") => {
                    let value = meta.value()?;
                    base_url = value.parse().ok();
                }
                item if item.is_ident("default_headers") => {
                    let value = meta.value()?;
                    let kvs: ExprTuple = value.parse().unwrap();
                    for kv in kvs.elems {
                        let kv: ExprTuple = parse2(kv.into_token_stream()).unwrap();
                        let mut kv = kv.elems.into_iter();
                        default_headers_key.push(kv.next().unwrap());
                        default_headers_value.push(kv.next().unwrap());
                    }
                }
                item if item.is_ident("default_headers_env") => {
                    let value = meta.value()?;
                    let kvs: ExprTuple = value.parse().unwrap();
                    for kv in kvs.elems {
                        let kv: ExprTuple = parse2(kv.into_token_stream()).unwrap();
                        let mut kv = kv.elems.into_iter();
                        default_headers_env_key.push(kv.next().unwrap());
                        default_headers_env_value.push(kv.next().unwrap());
                    }
                }
                item if item.is_ident("redirect") => {
                    let value = meta.value()?;
                    redirect = value.parse().ok();
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

    if base_url.is_none() {
        panic!("base_url must be provided");
    }

    let redirct = match redirect {
        Some(expr) => quote! {
            builder = builder.redirect(#expr);
        },
        None => quote! {},
    };

    let expanded = quote! {
        impl #impl_generics::api_req::ApiCaller for #name #ty_generics #where_clause {
            const BASE_URL: &'static str = #base_url;

            /// return a client with default headers
            fn client() -> ::api_req::__reqwest_Client {
                static CLIENT: ::std::sync::LazyLock<::api_req::__reqwest_Client> = ::std::sync::LazyLock::new(|| {
                        let mut builder = ::api_req::__reqwest_Client::builder();
                        #redirct
                        let mut default_headers = ::api_req::header::HeaderMap::new();
                        #(
                            let mut value: ::api_req::header::HeaderValue = #default_headers_value.parse().unwrap();
                            value.set_sensitive(true);
                            default_headers.insert(
                                #default_headers_key,
                                value
                            );
                        )*
                        #(
                            let mut value: ::api_req::header::HeaderValue = ::std::env::var(#default_headers_env_value).unwrap().parse().unwrap();
                            value.set_sensitive(true);
                            default_headers.insert(
                                #default_headers_env_key,
                                value
                            );
                        )*
                        builder.default_headers(default_headers).build().unwrap()
                    }
                );
                CLIENT.clone()
            }
        }
    };

    TokenStream::from(expanded)
}
