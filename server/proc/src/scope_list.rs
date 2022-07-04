use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataEnum, DeriveInput};

const REGULAR_SCOPE_LIST: &str = "DEFAULT_USER_SCOPES";

const ADMIN_SCOPE_LIST: &str = "DEFAULT_ADMIN_SCOPES";

const DESCRIPTION_SCOPE_LIST: &str = "SCOPE_DESCRIPTIONS";

pub fn scope_list(input: DeriveInput) -> TokenStream {
    let enum_data = match input.data.clone() {
        Data::Enum(x) => x,
        _ => {
            return quote_spanned! {
                input.ident.span() => compiler_error!("The ScopeList macro may only be applied to enums");
            }
        }
    };

    gen(enum_data, input)
}

fn gen(data_enum: DataEnum, input: DeriveInput) -> TokenStream {
    let name = input.ident;

    let description_items: Vec<_> = data_enum
        .variants
        .iter()
        .map(|x| {
            let description = x
                .attrs
                .iter()
                .filter_map(|x| {
                    if let Ok(syn::Meta::NameValue(meta_name_value)) = x.parse_meta() {
                        if let syn::Lit::Str(doc) = meta_name_value.lit {
                            return Some(doc.value().trim().to_string());
                        }
                    }

                    None
                })
                .collect::<Vec<String>>()
                .first()
                .map(|x| x.to_string());

            let variant_name = &x.ident;

            if let Some(desc) = description {
                quote! {( #name::#variant_name, #desc )}
            } else {
                quote! {( #name::#variant_name, "" )}
            }
        })
        .collect();

    let (admin_scopes, user_scopes): (Vec<_>, Vec<_>) = data_enum
        .variants
        .iter()
        .map(|x| {
            let is_admin = x.attrs.iter().any(|x| {
                if let Some(ident) = x.path.get_ident() {
                    ident.to_string().eq("admin")
                } else {
                    false
                }
            });

            let variant_name = &x.ident;
            if is_admin {
                (
                    Some(quote! {
                        #name::#variant_name,
                    }),
                    None,
                )
            } else {
                (
                    None,
                    Some(quote! {
                        #name::#variant_name,
                    }),
                )
            }
        })
        .unzip();

    let regular_array_name = format_ident!("__{}", REGULAR_SCOPE_LIST);
    let admin_array_name = format_ident!("__{}", ADMIN_SCOPE_LIST);
    let description_array_name = format_ident!("__{}", DESCRIPTION_SCOPE_LIST);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        pub const #regular_array_name: &[#name] = &[
            #(#user_scopes)*
        ];

        pub const #admin_array_name: &[#name] = &[
            #(#admin_scopes)*
        ];

        pub const #description_array_name: &[(#name, &str)] = &[
            #(#description_items),*
        ];

        impl #impl_generics #name #ty_generics #where_clause {
            pub fn default_scopes() -> &'static [#name #ty_generics] {
                #regular_array_name
            }

            pub fn admin_scopes() -> &'static [#name #ty_generics] {
                #admin_array_name
            }

            pub fn descriptions() -> &'static [(#name #ty_generics, &'static str)] {
                #description_array_name
            }
        }
    }
}