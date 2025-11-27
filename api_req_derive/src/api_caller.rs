use proc_macro::TokenStream;
use quote::{ToTokens as _, quote};
use syn::{
    DeriveInput, Error, Expr, ExprArray, ExprTuple, parse_macro_input, parse2, spanned::Spanned,
};

pub(crate) fn derive_api_caller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_span = input.span();
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut base_url: Option<Expr> = None;
    let mut default_headers_key: Vec<Expr> = vec![];
    let mut default_headers_value: Vec<Expr> = vec![];
    let mut default_headers_env_key: Vec<Expr> = vec![];
    let mut default_headers_env_value: Vec<Expr> = vec![];
    let mut default_headers_env_key_or_omit: Vec<Expr> = vec![];
    let mut default_headers_env_value_or_omit: Vec<Expr> = vec![];
    let mut redirect: Option<Expr> = None;

    let Some(attr) = input
        .attrs
        .iter()
        .find(|&attr| attr.path().is_ident("api_req"))
    else {
        return Error::new(input_span, "api_req attribute is needed")
            .to_compile_error()
            .into();
    };

    if let Err(e) = attr.parse_nested_meta(|meta| {
        match &meta.path {
            item if item.is_ident("base_url") => {
                let value = meta.value()?;
                base_url = Some(value.parse()?);
            }
            item if item.is_ident("default_headers") => {
                let value = meta.value()?;
                let kvs: ExprArray = value.parse()?;
                for kv_expr in kvs.elems.iter() {
                    let kv: ExprTuple = parse2(kv_expr.to_token_stream())?;
                    let mut kv = kv.elems.into_iter().collect::<Vec<_>>();
                    if kv.len() != 2 {
                        Err(Error::new(
                            kv_expr.span(),
                            format!(
                                "(header_key, header_value) expected, which contains 2 elems, but got {} elems", kv.len()
                            )
                        ))?;
                    }
                    default_headers_key.push(kv.remove(0));
                    default_headers_value.push(kv.remove(0));
                }
            }
            item if item.is_ident("default_headers_env") => {
                let value = meta.value()?;
                let kvs: ExprArray = value.parse()?;
                for kv_expr in kvs.elems.iter() {
                    let kv: ExprTuple = parse2(kv_expr.to_token_stream())?;
                    let mut kv = kv.elems.into_iter().collect::<Vec<_>>();
                    if kv.len() != 2 {
                        Err(Error::new(
                            kv_expr.span(),
                            format!(
                                "(header_key, env_var) expected, which contains 2 elems, but got {} elems", kv.len()
                            )
                        ))?;
                    }
                    default_headers_env_key.push(kv.remove(0));
                    default_headers_env_value.push(kv.remove(0));
                }
            }
            item if item.is_ident("default_headers_env_or_default") => {
                let value = meta.value()?;
                let kvs: ExprArray = value.parse()?;
                for kv_expr in kvs.elems.iter() {
                    let kv: ExprTuple = parse2(kv_expr.to_token_stream())?;
                    let mut kv = kv.elems.into_iter().collect::<Vec<_>>();
                    if kv.len() != 2 {
                        Err(Error::new(
                            kv_expr.span(),
                            format!(
                                "(header_key, env_var) expected, which contains 2 elems, but got {} elems", kv.len()
                            )
                        ))?;
                    }
                    default_headers_env_key_or_omit.push(kv.remove(0));
                    default_headers_env_value_or_omit.push(kv.remove(0));
                }
            }
            item if item.is_ident("redirect") => {
                let value = meta.value()?;
                redirect = Some(value.parse().unwrap());
            }
            item => Err(Error::new(item.span(), "unsupported meta"))?,
        }
        Ok(())
    }) {
        return e.to_compile_error().into();
    };

    if base_url.is_none() {
        return Error::new(input_span, "base_url must be provided")
            .to_compile_error()
            .into();
    }

    let redirct = match redirect {
        Some(expr) => quote! {
            builder = builder.redirect(#expr);
        },
        None => quote! {},
    };

    let cookie = match cfg!(feature = "cookies") {
        true => quote! {
            builder = builder.cookie_provider(::api_req::api_caller::COOKIE_JAR.clone());
        },
        false => quote! {},
    };

    let expanded = quote! {
        impl #impl_generics::api_req::ApiCaller for #name #ty_generics #where_clause {
            const BASE_URL: &'static str = #base_url;

            /// return a client with default headers
            fn client() -> ::api_req::__reqwest_Client {
                static CLIENT: ::std::sync::LazyLock<::api_req::__reqwest_Client> = ::std::sync::LazyLock::new(|| {
                        let mut builder = ::api_req::__reqwest_Client::builder();
                        #redirct
                        #cookie
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
                        #(
                            if let Ok(v) = ::std::env::var(#default_headers_env_value_or_omit) {
                                let mut value: ::api_req::header::HeaderValue = v.parse().unwrap();
                                value.set_sensitive(true);
                                default_headers.insert(
                                    #default_headers_env_key_or_omit,
                                    value
                                );
                            }
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
