use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{DeriveInput, Expr, LitStr, parse_macro_input};

pub(crate) fn derive_payload(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut path: Expr = syn::parse_str("\"\"").unwrap();
    let mut method: LitStr = syn::parse_str("\"GET\"").unwrap();

    if let Some(attr) = input
        .attrs
        .iter()
        .find(|&attr| attr.path().is_ident("payload"))
    {
        attr.parse_nested_meta(|meta| {
            match &meta.path {
                item if item.is_ident("path") => {
                    let value = meta.value()?;
                    let p: LitStr = value.parse().unwrap();
                    let re = Regex::new(r"\{(\w+)\}").unwrap();
                    let format_args = re
                        .captures_iter(&p.value())
                        .map(|c| c.extract::<1>())
                        .map(|cap| cap.1[0].to_string())
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
                item => Err(meta.error(format!(
                    "unsupported attribute: {}",
                    item.get_ident().unwrap()
                )))?,
            }
            Ok(())
        })
        .unwrap();
    };

    if !["GET", "POST"].contains(&method.value().as_str()) {
        panic!("method must be either GET or POST");
    }

    let expanded = quote! {
        impl #impl_generics ::api_req::Payload for #name #ty_generics #where_clause {
            const METHOD: &'static str = #method;

            fn path(&self) -> Option<String> {
                Some(#path)
            }
        }
    };

    TokenStream::from(expanded)
}
