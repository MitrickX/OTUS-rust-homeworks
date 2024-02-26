extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::Parser;

#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let data = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated
        .parse(input)
        .expect("support only string literals");

    let elem: Vec<Ident> = data
        .into_iter()
        .filter(|l| l.value().len() % 2 == 0)
        .map(|l| Ident::new(l.value().as_str(), Span::call_site()))
        .collect();

    let q = quote! {
        (#( #elem(), )*)
    };

    q.into()
}
