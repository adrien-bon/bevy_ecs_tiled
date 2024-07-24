static TILED_RENAME_ATTR: &str = "tiled_rename";
static TILED_SKIP_ATTR: &str = "tiled_skip";
static TILED_OBSERVER_ATTR: &str = "tiled_observer";

pub fn expand_tiled_custom_tiles_derive(input: syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = input.ident;
    let syn::Data::Struct(data_struct) = &input.data else {
        panic!("TiledCustomTile can only be derived for structs");
    };

    let observer_attrs = &input
        .attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap() == TILED_OBSERVER_ATTR);
    let observer = {
        if let Some(attr) = observer_attrs {
            match &attr.meta {
                syn::Meta::List(list) => {
                    let func = &list.tokens;
                    quote::quote!(
                        let mut observer = Observer::new(#func);
                        observer.watch_entity(create_event.entity);
                        commands
                            .spawn(observer)
                            .insert(Name::new(format!("Observer({})", stringify!(#func))))
                            .set_parent(create_event.entity);
                        commands.trigger_targets(create_event.clone(), create_event.entity);
                    )
                }
                _ => {
                    panic!("tiled_observer attribute must be a list!");
                }
            }
        } else {
            quote::quote!()
        }
    };

    let ctor = {
        let syn::Fields::Named(fields) = &data_struct.fields else {
            panic!("TiledCustomTile can only be derived for structs with named fields!");
        };
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
                fields_cton.push(expand_custom_tile_fields_rename(&field_name, &attr.meta));
            } else {
                fields_cton.push(expand_custom_tile_fields(&field_name));
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
    };

    quote::quote! {
        impl bevy_ecs_tiled::properties::traits::TiledCustomTile for #ty {
            fn initialize(
                commands: &mut Commands,
                create_event: &TiledCustomTileCreated,
            ) {
                commands.entity(create_event.entity).insert(#ctor);
                #observer
            }
        }
    }
    .into()
}

fn expand_custom_tile_fields(field_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote::quote!(
        #field_name: if create_event.tile_data.properties.contains_key(stringify!(#field_name)) {
            create_event.tile_data.properties
                .get(stringify!(#field_name))
                .unwrap()
                .clone()
                .into_user_type()
        } else { // Field not found: use default value
            Self::default().#field_name
        },
    )
}

fn expand_custom_tile_fields_rename(
    field_name: &syn::Ident,
    meta: &syn::Meta,
) -> proc_macro2::TokenStream {
    let name = match meta {
        syn::Meta::NameValue(value) => &value.value,
        _ => panic!("tiled_rename attribute must be a named value!"),
    };

    quote::quote!(
        #field_name: if create_event.tile_data.properties.contains_key(#name) {
            create_event.tile_data.properties
                .get(#name)
                .unwrap()
                .clone()
                .into_user_type()
        } else { // Field not found: use default value
            Self::default().#field_name
        },
    )
}
