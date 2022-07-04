use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Data, DataEnum, DeriveInput};

pub fn variants(input: DeriveInput) -> TokenStream {
    let data_enum = match input.clone().data {
        Data::Enum(x) => x,
        _ => {
            return quote_spanned! {
                input.ident.span() => compiler_error!("Only enums are supported");
            }
        }
    };

    gen_variants(data_enum, input)
}

fn gen_variants(data_enum: DataEnum, input: DeriveInput) -> TokenStream {
    let variants = data_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;

            quote!(
                Self::#variant_ident
            )
        })
        .collect::<Vec<_>>();

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn variants() -> &'static [Self] {
                &[
                    #(#variants),*
                ]
            }
        }
    }
}