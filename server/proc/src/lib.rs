mod scope_list;
mod variants;
mod stringify;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Create a [`std::string::ToString`](std::string::ToString) and [`std::str::FromStr`](std::str::FromStr) implementation for an Enum.
/// The enum is not allowed to have any variants which contain fields.
#[proc_macro_derive(Stringify)]
pub fn stringify(input: TokenStream) -> TokenStream {
    let de_input = parse_macro_input!(input as DeriveInput);

    TokenStream::from(stringify::stringify(de_input))
}

#[proc_macro_derive(ScopeList, attributes(admin))]
pub fn scope_list(input: TokenStream) -> TokenStream {
    let de_input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(scope_list::scope_list(de_input))
}

/// Automatically generate an implementation to get all variants of the enum as a slice
#[proc_macro_derive(Variants)]
pub fn variants(input: TokenStream) -> TokenStream {
    let de_input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(variants::variants(de_input))
}