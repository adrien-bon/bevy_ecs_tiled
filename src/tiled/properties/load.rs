use crate::prelude::tiled::PropertyValue as PV;
use crate::prelude::*;
use bevy::{
    asset::LoadContext,
    ecs::reflect::ReflectBundle,
    platform::collections::HashMap,
    prelude::*,
    reflect::{
        NamedField, Reflect, ReflectMut, ReflectRef, TypeInfo, TypeRegistration, TypeRegistry,
        UnnamedField,
        array::DynamicArray,
        enums::{DynamicEnum, DynamicVariant, VariantInfo, VariantType},
        list::DynamicList,
        map::DynamicMap,
        set::DynamicSet,
        structs::DynamicStruct,
        tuple::DynamicTuple,
        tuple_struct::DynamicTupleStruct,
    },
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct DeserializedMapProperties<const HYDRATED: bool = false> {
    pub(crate) map: DeserializedProperties,
    pub(crate) layers: HashMap<u32, DeserializedProperties>,
    pub(crate) tiles: HashMap<u32, HashMap<tiled::TileId, DeserializedProperties>>,
    pub(crate) objects: HashMap<u32, DeserializedProperties>,
}

impl DeserializedMapProperties<false> {
    /// Load properties of the map, its layers, objects and tiles.
    ///
    /// `tilesets` gives the tilesets for which tiles properties should be loaded, along with their tileset ID.
    pub(crate) fn load<'a>(
        map: &tiled::Map,
        tilesets: impl Iterator<Item = (u32, &'a tiled::Tileset)>,
        registry: &TypeRegistry,
        load_context: &mut LoadContext<'_>,
    ) -> Self {
        let map_props = DeserializedProperties::load(&map.properties, registry, load_context, true);

        let mut objects = HashMap::default();
        let mut layers = HashMap::default();
        let mut to_process = Vec::from_iter(map.layers());
        while let Some(layer) = to_process.pop() {
            layers.insert(
                layer.id(),
                DeserializedProperties::load(&layer.properties, registry, load_context, false),
            );
            match layer.layer_type() {
                tiled::LayerType::Objects(object) => {
                    for object in object.objects() {
                        objects.insert(
                            object.id(),
                            DeserializedProperties::load(
                                &object.properties,
                                registry,
                                load_context,
                                false,
                            ),
                        );
                    }
                }
                tiled::LayerType::Group(group) => {
                    to_process.extend(group.layers());
                }
                _ => {}
            }
        }

        let tiles = tilesets
            .map(|(tileset_id, tileset)| {
                (
                    tileset_id,
                    tileset
                        .tiles()
                        .map(|(id, t)| {
                            (
                                id,
                                DeserializedProperties::load(
                                    &t.properties,
                                    registry,
                                    load_context,
                                    false,
                                ),
                            )
                        })
                        .collect(),
                )
            })
            .collect();

        Self {
            map: map_props,
            layers,
            tiles,
            objects,
        }
    }

    /// Retrieve the hydrated properties of a tile, if it has any
    pub(crate) fn tile_properties(
        &self,
        tileset_id: u32,
        tile_id: tiled::TileId,
        entity_map: &HashMap<u32, Entity>,
    ) -> Option<DeserializedProperties> {
        let mut properties = self.tiles.get(&tileset_id)?.get(&tile_id)?.clone();
        if properties.properties.is_empty() {
            return None;
        }
        properties.hydrate(entity_map);
        Some(properties)
    }

    pub(crate) fn hydrate(
        mut self,
        entity_map: &HashMap<u32, Entity>,
    ) -> DeserializedMapProperties<true> {
        self.map.hydrate(entity_map);
        for (_, layer) in self.layers.iter_mut() {
            layer.hydrate(entity_map);
        }
        for (_, obj) in self.objects.iter_mut() {
            obj.hydrate(entity_map);
        }
        for (_, tiles) in self.tiles.iter_mut() {
            for (_, tile) in tiles.iter_mut() {
                tile.hydrate(entity_map);
            }
        }

        DeserializedMapProperties::<true> {
            map: self.map,
            layers: self.layers,
            tiles: self.tiles,
            objects: self.objects,
        }
    }
}

/// Properties for an entity deserialized from a [`tiled::Properties`]
#[derive(Debug)]
pub(crate) struct DeserializedProperties {
    pub(crate) properties: Vec<Box<dyn PartialReflect>>,
}

impl Clone for DeserializedProperties {
    fn clone(&self) -> Self {
        Self {
            properties: self
                .properties
                .iter()
                .map(|r| r.to_dynamic().expect("Can't be changed into dynamic"))
                .collect(),
        }
    }
}

impl DeserializedProperties {
    fn load(
        properties: &tiled::Properties,
        registry: &TypeRegistry,
        load_cx: &mut LoadContext<'_>,
        resources_allowed: bool,
    ) -> Self {
        let mut props: Vec<Box<dyn PartialReflect>> = Vec::new();

        for (name, property) in properties.clone() {
            let (reg_name, reg) = match &property {
                PV::ClassValue {
                    property_type,
                    properties: _,
                } => (property_type, registry.get_with_type_path(property_type)),
                PV::FileValue(file) => {
                    props.push(Box::new(load_cx.load_builder().load_untyped(file)));
                    continue;
                }
                _ => {
                    bevy::log::warn!(
                        "error deserializing property: unknown property `{name}`:`{property:?}`"
                    );
                    continue;
                }
            };

            let Some(reg) = reg else {
                bevy::log::error!(
                    "error deserializing property `{name}`: `{reg_name}` is not registered in the TypeRegistry."
                );
                continue;
            };

            if reg.data::<ReflectComponent>().is_none() && reg.data::<ReflectBundle>().is_none() {
                if reg.data::<ReflectResource>().is_some() {
                    if !resources_allowed {
                        bevy::log::warn!(
                            "error deserializing property `{name}`: Resources are only allowed as map properties"
                        );
                        continue;
                    }
                } else {
                    bevy::log::warn!(
                        "error deserializing property `{name}`: type `{reg_name}` is not registered as a Component, Bundle, or Resource"
                    );
                    continue;
                }
            }

            match Self::deserialize_property(property, reg, registry, &mut Some(load_cx), None) {
                Ok(prop) => {
                    props.push(prop);
                }
                Err(e) => {
                    bevy::log::error!("error deserializing property `{name}`: {e}");
                }
            }
        }

        Self { properties: props }
    }

    fn deserialize_named_field(
        field: &NamedField,
        properties: &mut tiled::Properties,
        registration: &TypeRegistration,
        registry: &TypeRegistry,
        load_cx: &mut Option<&mut LoadContext<'_>>,
        parent_default_value: Option<&dyn Reflect>,
    ) -> Result<Box<dyn PartialReflect>, String> {
        let mut default_value = None;
        if let Some(default) = parent_default_value {
            default_value = match default.reflect_ref() {
                ReflectRef::Struct(t) => (*t).field(field.name()).and_then(|f| f.try_as_reflect()),
                _ => None,
            };
        }

        let value;
        if let Some(pv) = properties.remove(field.name()) {
            let Some(reg) = registry.get(field.type_id()) else {
                return Err(format!("type `{}` is not registered", field.type_path()));
            };
            value = Self::deserialize_property(pv, reg, registry, load_cx, default_value)?;
        } else if let Some(def) = default_value {
            // If a default value from parent is provided, use it
            value = def
                .to_dynamic()
                .expect("value can't be made dynamic")
                .into_partial_reflect();
        } else if let Some(def) = default_value_from_type_path(registry, field.type_path()) {
            // If no default value from parent is not provided, try to use type default()
            value = def.into_partial_reflect();
        } else {
            return Err(format!(
                "missing property `{}` on `{}` and no default value provided",
                field.name(),
                registration.type_info().type_path(),
            ));
        }
        Ok(value)
    }

    fn deserialize_unnamed_field(
        field: &UnnamedField,
        properties: &mut tiled::Properties,
        registration: &TypeRegistration,
        registry: &TypeRegistry,
        load_cx: &mut Option<&mut LoadContext<'_>>,
        parent_default_value: Option<&dyn Reflect>,
    ) -> Result<Box<dyn PartialReflect>, String> {
        let mut default_value = None;
        if let Some(default) = parent_default_value {
            default_value = match default.reflect_ref() {
                ReflectRef::TupleStruct(t) => {
                    (*t).field(field.index()).and_then(|f| f.try_as_reflect())
                }
                ReflectRef::Tuple(t) => (*t).field(field.index()).and_then(|f| f.try_as_reflect()),
                _ => None,
            };
        }

        let value;
        if let Some(pv) = properties.remove(&field.index().to_string()) {
            let Some(reg) = registry.get(field.type_id()) else {
                return Err(format!("type `{}` is not registered", field.type_path()));
            };
            value = Self::deserialize_property(pv, reg, registry, load_cx, default_value)?;
        } else if let Some(def) = default_value {
            // If a default value from parent is provided, use it
            value = def
                .to_dynamic()
                .expect("value can't be made dynamic")
                .into_partial_reflect();
        } else if let Some(default_value) =
            default_value_from_type_path(registry, field.type_path())
        {
            // If no default value from parent is not provided, try to use type default()
            value = default_value.into_partial_reflect();
        } else {
            return Err(format!(
                "missing property `{}` on `{}` and no default value found",
                field.index(),
                registration.type_info().type_path(),
            ));
        }
        Ok(value)
    }

    fn deserialize_property(
        property: PV,
        registration: &TypeRegistration,
        registry: &TypeRegistry,
        load_cx: &mut Option<&mut LoadContext<'_>>,
        default_value: Option<&dyn Reflect>,
    ) -> Result<Box<dyn PartialReflect>, String> {
        // I wonder if it's possible to call FromStr for String?
        // or ToString/Display?
        match (
            registration.type_info().type_path(),
            property,
            registration.type_info(),
        ) {
            ("bool", PV::BoolValue(b), _) => Ok(Box::new(b)),

            ("i8", PV::IntValue(i), _) => int_property::<i8>(i),
            ("i16", PV::IntValue(i), _) => int_property::<i16>(i),
            ("i32", PV::IntValue(i), _) => Ok(Box::new(i)),
            ("i64", PV::IntValue(i), _) => Ok(Box::new(i as i64)),
            ("i128", PV::IntValue(i), _) => Ok(Box::new(i as i128)),
            ("isize", PV::IntValue(i), _) => int_property::<isize>(i),
            ("u8", PV::IntValue(i), _) => int_property::<u8>(i),
            ("u16", PV::IntValue(i), _) => int_property::<u16>(i),
            ("u32", PV::IntValue(i), _) => int_property::<u32>(i),
            ("u64", PV::IntValue(i), _) => int_property::<u64>(i),
            ("u128", PV::IntValue(i), _) => int_property::<u128>(i),
            ("usize", PV::IntValue(i), _) => int_property::<usize>(i),

            ("f32", PV::FloatValue(f), _) => Ok(Box::new(f)),
            ("f64", PV::FloatValue(f), _) => Ok(Box::new(f as f64)),
            // Shouldn't need these but it's a backup
            ("f32", PV::IntValue(i), _) => Ok(Box::new(i as f32)),
            ("f64", PV::IntValue(i), _) => Ok(Box::new(i as f64)),

            ("bevy_color::color::Color", PV::ColorValue(c), _) => {
                Ok(Box::new(Color::srgba_u8(c.red, c.green, c.blue, c.alpha)))
            }
            ("alloc::string::String" | "alloc::borrow::Cow<str>", PV::StringValue(s), _) => {
                Ok(Box::new(s))
            }
            ("char", PV::StringValue(s), _) => s
                .chars()
                .next()
                .map(|c| Box::new(c) as Box<dyn PartialReflect>)
                .ok_or_else(|| "empty string cannot be converted to `char`".to_string()),
            ("std::path::PathBuf", PV::FileValue(s), _) => Ok(Box::new(PathBuf::from(s))),
            (a, PV::FileValue(s), _) if a.starts_with("bevy_asset::handle::Handle") => {
                if let Some(cx) = load_cx.as_mut() {
                    Ok(Box::new(cx.load_builder().load_untyped(s)))
                } else {
                    Err("No LoadContext provided: cannot load Handle<T>".to_string())
                }
            }
            ("bevy_ecs::entity::Entity", PV::ObjectValue(o), _) => {
                if o == 0 {
                    Err("empty object reference".to_string())
                } else {
                    entity_from_object_id(o).map(|e| Box::new(e) as Box<dyn PartialReflect>)
                }
            }
            ("core::option::Option<bevy_ecs::entity::Entity>", PV::ObjectValue(o), _) => {
                let entity = if o == 0 {
                    None
                } else {
                    Some(entity_from_object_id(o)?)
                };
                Ok(Box::new(entity))
            }
            (_, PV::StringValue(s), TypeInfo::Enum(info)) => {
                let Some(variant) = info.variant(&s) else {
                    return Err(format!("no variant `{s}` for `{}`", info.type_path()));
                };

                let VariantInfo::Unit(unit_info) = variant else {
                    return Err(format!(
                        "variant `{s}` is not a unit variant of `{}`",
                        info.type_path()
                    ));
                };

                let mut out = DynamicEnum::new(unit_info.name(), DynamicVariant::Unit);
                out.set_represented_type(Some(registration.type_info()));

                Ok(Box::new(out))
            }
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::Struct(info)) => {
                let mut out = DynamicStruct::default();
                out.set_represented_type(Some(registration.type_info()));

                // Use default value from type if available
                let tmp = default_value_from_type_path(registry, info.type_path());
                let default_value = match tmp {
                    Some(_) => tmp.as_deref(),
                    None => default_value,
                };

                for field in info.iter() {
                    let value = Self::deserialize_named_field(
                        field,
                        &mut properties,
                        registration,
                        registry,
                        load_cx,
                        default_value,
                    )?;
                    out.insert_boxed(field.name(), value);
                }

                Ok(Box::new(out))
            }
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::TupleStruct(info)) => {
                let mut out = DynamicTupleStruct::default();
                out.set_represented_type(Some(registration.type_info()));

                // Use default value from type if available
                let tmp = default_value_from_type_path(registry, info.type_path());
                let default_value = match tmp {
                    Some(_) => tmp.as_deref(),
                    None => default_value,
                };

                for field in info.iter() {
                    let value = Self::deserialize_unnamed_field(
                        field,
                        &mut properties,
                        registration,
                        registry,
                        load_cx,
                        default_value,
                    )?;
                    out.insert_boxed(value);
                }

                Ok(Box::new(out))
            }
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::Tuple(info)) => {
                let mut out = DynamicTuple::default();
                out.set_represented_type(Some(registration.type_info()));

                // Use default value from type if available
                let tmp = default_value_from_type_path(registry, info.type_path());
                let default_value = match tmp {
                    Some(_) => tmp.as_deref(),
                    None => default_value,
                };

                for field in info.iter() {
                    let value = Self::deserialize_unnamed_field(
                        field,
                        &mut properties,
                        registration,
                        registry,
                        load_cx,
                        default_value,
                    )?;
                    out.insert_boxed(value);
                }

                Ok(Box::new(out))
            }
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::Array(info)) => {
                let mut array = Vec::new();

                let Some(reg) = registry.get(info.item_ty().id()) else {
                    return Err(format!(
                        "type `{}` is not registered",
                        info.item_ty().path()
                    ));
                };

                for i in 0..info.capacity() {
                    let Some(pv) = properties.remove(&format!("[{i}]")) else {
                        return Err(format!("missing property on `{}`: `{i}`", info.type_path(),));
                    };

                    let value =
                        Self::deserialize_property(pv, reg, registry, load_cx, default_value)?;

                    array.push(value);
                }

                let mut out = DynamicArray::new(array.into());
                out.set_represented_type(Some(registration.type_info()));

                Ok(Box::new(out))
            }
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::Enum(info)) => {
                let mut out = DynamicEnum::default();
                out.set_represented_type(Some(registration.type_info()));

                // Use default value from type if available
                let tmp = default_value_from_type_path(registry, info.type_path());
                let default_value = match tmp {
                    Some(_) => tmp.as_deref(),
                    None => default_value,
                };

                if let Some(PV::StringValue(variant_name)) = properties.remove(":variant") {
                    let variant_out = match info.variant(&variant_name) {
                        Some(VariantInfo::Struct(variant_info)) => {
                            if let Some(PV::ClassValue { mut properties, .. }) =
                                properties.remove(&variant_name)
                            {
                                let mut out = DynamicStruct::default();
                                for field in variant_info.iter() {
                                    let value = Self::deserialize_named_field(
                                        field,
                                        &mut properties,
                                        registration,
                                        registry,
                                        load_cx,
                                        default_value,
                                    )?;
                                    out.insert_boxed(field.name(), value);
                                }
                                Ok(DynamicVariant::Struct(out))
                            } else {
                                Err(format!(
                                    "`{}` enum does not contain `{variant_name}` variant data",
                                    info.type_path(),
                                ))
                            }
                        }
                        Some(VariantInfo::Tuple(variant_info)) => {
                            if let Some(PV::ClassValue { mut properties, .. }) =
                                properties.remove(&variant_name)
                            {
                                let mut out = DynamicTuple::default();
                                for field in variant_info.iter() {
                                    let value = Self::deserialize_unnamed_field(
                                        field,
                                        &mut properties,
                                        registration,
                                        registry,
                                        load_cx,
                                        default_value,
                                    )?;
                                    out.insert_boxed(value);
                                }
                                Ok(DynamicVariant::Tuple(out))
                            } else {
                                Err(format!(
                                    "`{}` enum does not contain `{variant_name}` variant data",
                                    info.type_path(),
                                ))
                            }
                        }
                        Some(VariantInfo::Unit(_)) => Ok(DynamicVariant::Unit),
                        None => Err(format!(
                            "`{}` enum does not contain `{variant_name}` variant",
                            info.type_path(),
                        )),
                    }?;

                    out.set_variant_with_index(
                        info.index_of(&variant_name).unwrap(),
                        variant_name,
                        variant_out,
                    );

                    return Ok(Box::new(out));
                };

                if let Some(default_value) = default_value
                    && let ReflectRef::Enum(e) = default_value.reflect_ref()
                {
                    out = e.to_dynamic_enum().expect("value can't be made dynamic");
                    return Ok(Box::new(out));
                }

                Err(format!(
                    "missing enum properties for `{}` and no default value provided",
                    info.type_path()
                ))
            }
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::List(info)) => {
                let mut list = Vec::new();

                let Some(reg) = registry.get(info.item_ty().id()) else {
                    return Err(format!(
                        "type `{}` is not registered",
                        info.item_ty().path()
                    ));
                };

                let Some(pv) = properties.remove("list") else {
                    return Err(format!(
                        "missing property on `{}`: `list`",
                        info.type_path(),
                    ));
                };

                let PV::ListValue(items) = pv else {
                    return Err(format!(
                        "wrong property type on `{}`: `list`",
                        info.type_path(),
                    ));
                };

                for item in items {
                    let value =
                        Self::deserialize_property(item, reg, registry, load_cx, default_value)?;
                    list.push(value)
                }

                let mut out = DynamicList::from_iter(list);
                out.set_represented_type(Some(registration.type_info()));

                Ok(Box::new(out))
            }
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::Set(info)) => {
                let mut list = Vec::new();

                let Some(reg) = registry.get(info.value_ty().id()) else {
                    return Err(format!(
                        "type `{}` is not registered",
                        info.value_ty().path()
                    ));
                };

                let Some(pv) = properties.remove("set") else {
                    return Err(format!("missing property on `{}`: `set`", info.type_path(),));
                };

                let PV::ListValue(items) = pv else {
                    return Err(format!(
                        "wrong property type on `{}`: `set`",
                        info.type_path(),
                    ));
                };

                for item in items {
                    let value =
                        Self::deserialize_property(item, reg, registry, load_cx, default_value)?;
                    list.push(value);
                }

                let mut out = DynamicSet::from_iter(list);
                out.set_represented_type(Some(registration.type_info()));

                Ok(Box::new(out))
            }
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::Map(info)) => {
                let mut list = Vec::new();

                let Some(key_reg) = registry.get(info.key_ty().id()) else {
                    return Err(format!(
                        "key type `{}` is not registered",
                        info.key_ty().path()
                    ));
                };

                let Some(value_reg) = registry.get(info.value_ty().id()) else {
                    return Err(format!(
                        "value type `{}` is not registered",
                        info.value_ty().path()
                    ));
                };

                let Some(pv) = properties.remove("map") else {
                    return Err(format!("missing property on `{}`: `map`", info.type_path(),));
                };

                let PV::ListValue(items) = pv else {
                    return Err(format!(
                        "wrong property type on `{}`: `map`",
                        info.type_path(),
                    ));
                };

                for item in items {
                    let PV::ClassValue { mut properties, .. } = item else {
                        return Err("wrong property type for map item".to_string());
                    };

                    let key;

                    if let Some(key_prop) = properties.remove("key") {
                        key = Self::deserialize_property(
                            key_prop,
                            key_reg,
                            registry,
                            load_cx,
                            default_value,
                        )?;
                    } else if let Some(def) =
                        default_value_from_type_path(registry, info.key_ty().path())
                    {
                        key = def.into_partial_reflect()
                    } else {
                        return Err(format!(
                            "missing key for map item on type  {}",
                            info.type_path()
                        ));
                    }

                    let value;

                    if let Some(value_prop) = properties.remove("value") {
                        value = Self::deserialize_property(
                            value_prop,
                            value_reg,
                            registry,
                            load_cx,
                            default_value,
                        )?;
                    } else if let Some(def) =
                        default_value_from_type_path(registry, info.value_ty().path())
                    {
                        value = def.into_partial_reflect()
                    } else {
                        return Err(format!(
                            "missing value for map item on type  {}",
                            info.type_path()
                        ));
                    }

                    list.push((key, value));
                }

                let mut out = DynamicMap::from_iter(list);
                out.set_represented_type(Some(registration.type_info()));

                Ok(Box::new(out))
            }
            // Note: ClassValue and TypeInfo::Value is not included
            (a, b, c) => Err(format!(
                "unable to deserialize `{a}` from {b:?} (type_info = {c:?}"
            )),
        }
    }

    pub(crate) fn hydrate(&mut self, obj_entity_map: &HashMap<u32, Entity>) {
        for resource in self.properties.iter_mut() {
            hydrate(resource.as_mut(), obj_entity_map);
        }
    }
}

/// Convert a Tiled integer property into the target integer type, failing if out of range
fn int_property<T>(i: i32) -> Result<Box<dyn PartialReflect>, String>
where
    T: TryFrom<i32> + PartialReflect,
{
    T::try_from(i)
        .map(|v| Box::new(v) as Box<dyn PartialReflect>)
        .map_err(|_| {
            format!(
                "value `{i}` is out of range for `{}`",
                std::any::type_name::<T>()
            )
        })
}

/// Temporary [`Entity`] holding a Tiled object ID, replaced by the actual entity on hydration
fn entity_from_object_id(object_id: u32) -> Result<Entity, String> {
    Entity::from_raw_u32(object_id).ok_or_else(|| format!("invalid object reference `{object_id}`"))
}

fn default_value_from_type_path(registry: &TypeRegistry, path: &str) -> Option<Box<dyn Reflect>> {
    registry
        .get_with_type_path(path)
        .and_then(|reg| reg.data::<ReflectDefault>().map(|v| v.default()))
}

fn object_ref(
    obj: &dyn PartialReflect,
    obj_entity_map: &HashMap<u32, Entity>,
) -> Option<Box<dyn PartialReflect>> {
    if obj.represents::<Entity>() {
        let obj = Entity::take_from_reflect(obj.to_dynamic().expect("Obj can't be made dynamic"))
            .unwrap();
        if let Some(&e) = obj_entity_map.get(&obj.index_u32()) {
            Some(Box::new(e))
        } else {
            bevy::log::error!(
                "error hydrating properties: missing entity for object {}, using Entity::PLACEHOLDER",
                obj.index()
            );
            Some(Box::new(Entity::PLACEHOLDER))
        }
    } else if obj.represents::<Option<Entity>>() {
        // maybe the map get should panic actually
        Some(Box::new(
            Option::<Entity>::take_from_reflect(
                obj.to_dynamic().expect("obj can't be made dynamic"),
            )
            .unwrap()
            .and_then(|obj| obj_entity_map.get(&obj.index_u32()).copied()),
        ))
    } else {
        None
    }
}

fn hydrate(object: &mut dyn PartialReflect, obj_entity_map: &HashMap<u32, Entity>) {
    if let Some(obj) = object_ref(object, obj_entity_map) {
        object.apply(obj.as_partial_reflect());
        return;
    }

    match object.reflect_mut() {
        ReflectMut::Struct(s) => {
            for i in 0..s.field_len() {
                hydrate(s.field_at_mut(i).unwrap(), obj_entity_map);
            }
        }
        ReflectMut::TupleStruct(s) => {
            for i in 0..s.field_len() {
                hydrate(s.field_mut(i).unwrap(), obj_entity_map);
            }
        }
        ReflectMut::Tuple(s) => {
            for i in 0..s.field_len() {
                hydrate(s.field_mut(i).unwrap(), obj_entity_map);
            }
        }
        ReflectMut::List(s) => {
            for i in 0..s.len() {
                hydrate(s.get_mut(i).unwrap(), obj_entity_map);
            }
        }
        ReflectMut::Array(s) => {
            for i in 0..s.len() {
                hydrate(s.get_mut(i).unwrap(), obj_entity_map);
            }
        }
        ReflectMut::Enum(s) => match s.variant_type() {
            VariantType::Tuple => {
                for i in 0..s.field_len() {
                    hydrate(s.field_at_mut(i).unwrap(), obj_entity_map);
                }
            }
            VariantType::Struct => {
                for i in 0..s.field_len() {
                    let name = s.name_at(i).unwrap().to_owned();
                    hydrate(s.field_mut(&name).unwrap(), obj_entity_map);
                }
            }
            _ => {}
        },
        ReflectMut::Map(s) => {
            for (k, v) in s.drain().iter_mut() {
                let k = k.as_partial_reflect();
                let v = v.as_partial_reflect_mut();
                if k.represents::<Entity>() || k.represents::<Option<Entity>>() {
                    bevy::log::error!(
                        "error hydrating properties: object references cannot be used as map keys, keeping raw value"
                    );
                }
                hydrate(v, obj_entity_map);
                s.insert_boxed(
                    k.to_dynamic().expect("obj can't be made dynamic"),
                    v.to_dynamic().expect("obj can't be made dynamic"),
                );
            }
        }
        // Cannot hydrate a Set since it does not have a get_mut() function
        ReflectMut::Set(_) => {}
        // we don't care about any of the other values
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_all_properties() {
        let map = tiled::Loader::new()
            .load_tmx_map("assets/maps/hexagonal/finite_pointy_top_odd.tmx")
            .unwrap();
        println!("Found map: {:?}", map.properties);
        for layer in map.layers() {
            println!("Found layer: '{}' = {:?}", layer.name, layer.properties);
            if let Some(objects_layer) = layer.as_object_layer() {
                for object in objects_layer.objects() {
                    println!("Found object: '{}' = {:?}", object.name, object.properties);
                }
            }
            if let Some(tiles_layer) = layer.as_tile_layer() {
                for x in 0..map.height {
                    for y in 0..map.width {
                        if let Some(tile) = tiles_layer.get_tile(x as i32, y as i32)
                            && let Some(t) = tile.get_tile()
                        {
                            println!(
                                "Found tile: {}({},{})' = {:?}",
                                layer.name, x, y, t.properties
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn deserialize_simple_enum() {
        #[derive(Reflect, PartialEq, Debug)]
        enum EnumComponent {
            VarA,
            VarB,
            VarC,
        }

        let mut registry = TypeRegistry::new();
        registry.register::<EnumComponent>();

        let raw_value = EnumComponent::VarB;
        let tiled_value = PV::StringValue("VarB".to_string());

        let res = DeserializedProperties::deserialize_property(
            tiled_value,
            registry
                .get_with_type_path(EnumComponent::type_path())
                .unwrap(),
            &registry,
            &mut None,
            None,
        )
        .unwrap();
        assert!(res.represents::<EnumComponent>());

        let v: Result<EnumComponent, _> = FromReflect::take_from_reflect(res);
        assert_eq!(v.unwrap(), raw_value);
    }

    #[test]
    fn deserialize_nested_struct() {
        #[derive(Reflect, Default, PartialEq, Debug)]
        #[reflect(Default)]
        enum TestEnum {
            VarA,
            #[default]
            VarB,
            VarC,
        }

        #[derive(Reflect, PartialEq, Debug)]
        #[reflect(Default)]
        struct InnerStruct {
            another_float: f64,
            another_integer: u16,
            another_enum: TestEnum,
        }
        impl Default for InnerStruct {
            fn default() -> Self {
                Self {
                    another_float: 0.,
                    another_integer: 42,
                    another_enum: TestEnum::VarC,
                }
            }
        }

        #[derive(Component, Reflect, Default, PartialEq, Debug)]
        #[reflect(Component, Default)]
        struct StructComponent {
            a_float: f32,
            an_enum: TestEnum,
            a_struct: InnerStruct,
            an_integer: i32,
        }

        let mut registry = TypeRegistry::new();
        registry.register::<TestEnum>();
        registry.register::<InnerStruct>();
        registry.register::<StructComponent>();

        let raw_value = StructComponent::default();
        let tiled_value = PV::ClassValue {
            property_type: StructComponent::type_path().to_string(),
            properties: std::collections::HashMap::from([
                ("a_float".to_string(), PV::FloatValue(0.)),
                ("an_enum".to_string(), PV::StringValue("VarB".to_string())),
                (
                    "a_struct".to_string(),
                    PV::ClassValue {
                        property_type: InnerStruct::type_path().to_string(),
                        properties: std::collections::HashMap::from([
                            ("another_float".to_string(), PV::FloatValue(0.)),
                            ("another_integer".to_string(), PV::IntValue(42)),
                            (
                                "another_enum".to_string(),
                                PV::StringValue("VarC".to_string()),
                            ),
                        ]),
                    },
                ),
                ("an_integer".to_string(), PV::IntValue(0)),
            ]),
        };

        let res = DeserializedProperties::deserialize_property(
            tiled_value,
            registry
                .get_with_type_path(StructComponent::type_path())
                .unwrap(),
            &registry,
            &mut None,
            None,
        )
        .unwrap();
        assert!(res.represents::<StructComponent>());

        let v: Result<StructComponent, _> = FromReflect::take_from_reflect(res);
        assert_eq!(v.unwrap(), raw_value);
    }

    fn deserialize_primitive<T: Reflect + TypePath + bevy::reflect::GetTypeRegistration>(
        value: PV,
    ) -> Result<T, String> {
        let mut registry = TypeRegistry::new();
        registry.register::<T>();
        let res = DeserializedProperties::deserialize_property(
            value,
            registry.get_with_type_path(T::type_path()).unwrap(),
            &registry,
            &mut None,
            None,
        )?;
        res.try_take::<T>()
            .map_err(|_| "unexpected deserialized type".to_string())
    }

    #[test]
    fn deserialize_out_of_range_integer_returns_error() {
        assert_eq!(deserialize_primitive::<u8>(PV::IntValue(200)), Ok(200));
        assert!(deserialize_primitive::<u8>(PV::IntValue(300)).is_err());
        assert!(deserialize_primitive::<u32>(PV::IntValue(-1)).is_err());
        assert!(deserialize_primitive::<i8>(PV::IntValue(-129)).is_err());
    }

    #[test]
    fn deserialize_empty_char_returns_error() {
        assert_eq!(
            deserialize_primitive::<char>(PV::StringValue("ab".to_string())),
            Ok('a')
        );
        assert!(deserialize_primitive::<char>(PV::StringValue(String::new())).is_err());
    }

    #[test]
    fn deserialize_array() {
        let tiled_value = PV::ClassValue {
            property_type: <[u8; 3]>::type_path().to_string(),
            properties: std::collections::HashMap::from([
                ("[0]".to_string(), PV::IntValue(1)),
                ("[1]".to_string(), PV::IntValue(2)),
                ("[2]".to_string(), PV::IntValue(3)),
            ]),
        };
        let mut registry = TypeRegistry::new();
        registry.register::<[u8; 3]>();
        let res = DeserializedProperties::deserialize_property(
            tiled_value,
            registry.get_with_type_path(<[u8; 3]>::type_path()).unwrap(),
            &registry,
            &mut None,
            None,
        )
        .unwrap();
        let v: Option<[u8; 3]> = FromReflect::from_reflect(res.as_ref());
        assert_eq!(v, Some([1, 2, 3]));
    }

    #[test]
    fn hydrate_missing_object_uses_placeholder() {
        let mut property: Box<dyn PartialReflect> = Box::new(Entity::from_raw_u32(42).unwrap());
        hydrate(property.as_mut(), &HashMap::default());
        assert_eq!(
            property.try_take::<Entity>().ok(),
            Some(Entity::PLACEHOLDER)
        );
    }
}
