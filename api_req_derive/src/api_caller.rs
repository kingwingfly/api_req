use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, parse_macro_input};

pub(crate) fn derive_api_caller(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut base_url: Option<Expr> = None;

    if let Some(attr) = input.attrs.iter().find(|&attr| attr.path().is_ident("api")) {
        attr.parse_nested_meta(|meta| {
            match &meta.path {
                item if item.is_ident("base_url") => {
                    let value = meta.value()?;
                    base_url = value.parse().ok();
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

    let expanded = quote! {
        impl #impl_generics::api_req::ApiCaller for #name #ty_generics #where_clause {
            const BASE_URL: &'static str = #base_url;
        }
    };

    TokenStream::from(expanded)
}
