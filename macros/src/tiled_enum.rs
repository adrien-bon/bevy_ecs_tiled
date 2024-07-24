static TILED_RENAME_ATTR: &str = "tiled_rename";

pub fn expand_tiled_enum_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &input.ident;
    let variants = match input.data {
        syn::Data::Enum(data) => data.variants,
        _ => panic!("TiledEnum can only be derived for enums"),
    };

    let mut variants_cton = Vec::new();
    for variant in variants.iter() {
        let variant_name = &variant.ident;

        let attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap() == TILED_RENAME_ATTR);
        if let Some(attr) = attr {
            variants_cton.push(expand_enum_variant_rename(variant_name, &attr.meta));
        } else {
            variants_cton.push(expand_enum_variant(variant_name));
        }
    }

    quote::quote!(
        impl bevy_ecs_tiled::properties::traits::TiledEnum for #ty {
            fn get_identifier(ident: &str) -> Self {
                match ident {
                    #(#variants_cton)*
                    _ => panic!("Unknown enum variant: {}", ident),
                }
            }
        }

        impl IntoUserType<#ty> for tiled::PropertyValue {
            fn into_user_type(self) -> #ty {
                match self {
                    Self::StringValue(x) => {
                        <#ty as bevy_ecs_tiled::properties::traits::TiledEnum>::get_identifier(&x)
                    },
                    // Enum as IntegerValue are not supported yet
                    _ => panic!("Expected Enum({}) as StringValue!", stringify!(#ty)),
                }
            }
        }
    )
    .into()
}

fn expand_enum_variant_rename(
    variant_name: &syn::Ident,
    ldtk_name: &syn::Meta,
) -> proc_macro2::TokenStream {
    let name = match ldtk_name {
        syn::Meta::NameValue(value) => &value.value,
        _ => panic!("TiledEnum attribute must be a named value!"),
    };

    quote::quote!(
        #name => Self::#variant_name,
    )
    .into()
}

fn expand_enum_variant(variant_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote!(
        stringify!(#variant_name) => Self::#variant_name,
    )
    .into()
}
