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