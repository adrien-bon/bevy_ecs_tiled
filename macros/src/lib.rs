mod tiled_class;
mod tiled_custom_tile;
mod tiled_enum;
mod tiled_object;

#[proc_macro_derive(TiledObject, attributes(tiled_rename, tiled_skip, tiled_observer))]
pub fn derive_tiled_objects(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_object::expand_tiled_objects_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(TiledCustomTile, attributes(tiled_rename, tiled_skip, tiled_observer))]
pub fn derive_tiled_custom_tiles(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_custom_tile::expand_tiled_custom_tiles_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(TiledClass, attributes(tiled_rename, tiled_skip))]
pub fn derive_tiled_classes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_class::expand_tiled_class_derive(syn::parse(input).unwrap())
}

#[proc_macro_derive(TiledEnum, attributes(tiled_rename,))]
pub fn derive_tiled_enums(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_enum::expand_tiled_enum_derive(syn::parse(input).unwrap())
}
