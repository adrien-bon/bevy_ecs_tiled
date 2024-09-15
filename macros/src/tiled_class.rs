/// Tiled custom type
///

static TILED_RENAME_ATTR: &str = "tiled_rename";
static TILED_SKIP_ATTR: &str = "tiled_skip";

pub fn expand_tiled_class_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let syn::Data::Struct(data_struct) = input.data else {
        panic!("TiledClass can only be derived for structs!");
    };

    let ty = &input.ident;

    let ctor = {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            let fields = &fields.named;
            let mut fields_cton = Vec::new();

            for field in fields {
                let field_name = field.ident.as_ref().unwrap();

                let skip = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path().get_ident().unwrap() == TILED_SKIP_ATTR);
                if skip.is_some() {
                    continue;
                }

                let name = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path().get_ident().unwrap() == TILED_RENAME_ATTR);
                if let Some(attr) = name {
                    fields_cton.push(expand_class_fields_rename(field_name, &attr.meta));
                } else {
                    fields_cton.push(expand_class_fields(&field_name));
                }
            }

            let default = if fields_cton.len() < fields.len() {
                quote::quote!(..Default::default())
            } else {
                quote::quote!()
            };

            quote::quote!(
                Self {
                    #(#fields_cton)*
                    #default
                }
            )
        } else {
            quote::quote!(Self)
        }
    };

    quote::quote!(
        impl bevy_ecs_tiled::properties::traits::TiledClass for #ty {
            fn create(
                properties: &tiled::Properties,
            ) -> Self {
                #ctor
            }
        }
    )
    .into()
}

fn expand_class_fields(field_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote!(
        #field_name: if properties.contains_key(stringify!(#field_name)) {
            properties
                .get(stringify!(#field_name))
                .unwrap()
                .clone()
                .into_user_type()
        } else { // Field not found: use default value
            Self::default().#field_name
        },
    )
}

fn expand_class_fields_rename(
    field_name: &syn::Ident,
    meta: &syn::Meta,
) -> proc_macro2::TokenStream {
    let name = match meta {
        syn::Meta::NameValue(value) => &value.value,
        _ => panic!("tiled_rename attribute must be a named value!"),
    };

    quote::quote!(
        #field_name: if properties.contains_key(#name) {
            properties
                .get(#name)
                .unwrap()
                .clone()
                .into_user_type()
        } else { // Field not found: use default value
            Self::default().#field_name
        },
    )
}
