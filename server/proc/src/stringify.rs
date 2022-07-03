use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Data, DataEnum, DeriveInput};

pub fn stringify(input: DeriveInput) -> TokenStream {
    let data_enum = match input.clone().data {
        Data::Enum(x) => x,
        _ => {
            return quote_spanned! {
                input.ident.span() => compiler_error!("Only enums are supported");
            }
        }
    };

    let gen_to_string = gen_to_string(data_enum.clone(), input.clone());
    let gen_from_str = gen_from_str(data_enum, input);

    quote! {
        #gen_to_string
        #gen_from_str
    }
}

fn gen_to_string(data_enum: DataEnum, input: DeriveInput) -> TokenStream {
    let name = input.ident;

    let (quotes_owned, quotes_ref): (Vec<_>, Vec<_>) = data_enum
        .variants
        .iter()
        .map(|var| {
            let variant_name = &var.ident;
            let variant_name_string = variant_name.to_string();

            if var.fields.is_empty() {
                (
                    quote! {
                        Self::#variant_name => String::from(#variant_name_string),
                    },
                    quote! {
                        #name::#variant_name => String::from(#variant_name_string),
                    },
                )
            } else {
                (
                    quote_spanned! {
                        var.ident.span() => compiler_error!("Enums with fields in their variants are not supported");
                    },
                    quote_spanned! {
                        var.ident.span() => compiler_error!("Enums with fields in their variants are not supported");
                    },
                )
            }
        })
        .unzip();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::std::string::ToString for #name #ty_generics #where_clause {
            fn to_string(&self) -> String {
                match self {
                    #(#quotes_owned)*
                }
            }
        }

        impl #impl_generics ::std::string::ToString for &#name #ty_generics #where_clause {
            fn to_string(&self) -> String {
                match self {
                    #(#quotes_ref)*
                }
            }
        }
    }
}

fn gen_from_str(data_enum: DataEnum, input: DeriveInput) -> TokenStream {
    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let quotes = data_enum
        .variants
        .iter()
        .map(|var| {
            let variant_name = &var.ident;

            let variant_name_string = variant_name.to_string();

            if var.fields.is_empty() {
                quote! {
                    #variant_name_string => Self::#variant_name,
                }
            } else {
                quote_spanned! {
                    var.ident.span() => compiler_error!("Enums with fields in their variants are not supported");
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl #impl_generics ::std::str::FromStr for #name #ty_generics #where_clause {
            type Err = ();

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                let this = match s {
                    #(#quotes)*
                    _ => return Err(())
                };
                Ok(this)
            }
        }
    }
}