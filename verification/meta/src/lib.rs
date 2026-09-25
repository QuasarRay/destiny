extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Ident, ItemFn, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input,
};

struct ContractInput {
    name: Ident,
    source: LitStr,
    test: LitStr,
}

impl Parse for ContractInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let source = input.parse()?;
        input.parse::<Token![,]>()?;
        let test = input.parse()?;
        Ok(Self { name, source, test })
    }
}

/// Function-like proc macro used to declare a source-backed proof obligation.
///
/// Syntax:
/// destiny_contract!(NAME, "source/path.py", "test_name");
#[proc_macro]
pub fn destiny_contract(input: TokenStream) -> TokenStream {
    let ContractInput { name, source, test } = parse_macro_input!(input as ContractInput);
    quote! {
        pub const #name: (&'static str, &'static str) = (#source, #test);
    }
    .into()
}

/// Attribute macro that preserves the function while attaching source metadata
/// as a hidden constant.  The attribute payload is intentionally opaque to the
/// macro so new verifier-specific keys can be added without changing parsing.
#[proc_macro_attribute]
pub fn destiny_spec(attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let name = &function.sig.ident;
    let marker = format_ident!("__DESTINY_SPEC_{}", name.to_string().to_uppercase());
    let attr_tokens: TokenStream2 = attr.into();
    let metadata = attr_tokens.to_string();
    quote! {
        #function
        #[doc(hidden)]
        const #marker: &str = #metadata;
    }
    .into()
}

/// Derive macro for model types consumed by proof adapters.
///
/// It deliberately adds only representation-independent metadata; it does not
/// derive Bevy component access or any engine-facing behavior.
#[proc_macro_derive(DestinyModel)]
pub fn derive_destiny_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    quote! {
        impl #name {
            pub const DESTINY_FORMAL_MODEL: &'static str = stringify!(#name);
        }
    }
    .into()
}

/// Attribute macro marking a proof/helper as a delegation point.
///
/// This is separate from `destiny_spec` so the repetition scanner can
/// distinguish source specifications from representation delegation.
#[proc_macro_attribute]
pub fn destiny_delegate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let name = &function.sig.ident;
    let marker = format_ident!("__DESTINY_DELEGATE_{}", name.to_string().to_uppercase());
    let attr_tokens: TokenStream2 = attr.into();
    let metadata = attr_tokens.to_string();
    quote! {
        #function
        #[doc(hidden)]
        const #marker: &str = #metadata;
    }
    .into()
}
