use std::path::PathBuf;
use bevy::asset::LoadContext;
use bevy::ecs::reflect::ReflectBundle;
use bevy::prelude::{Color, Entity, FromReflect, ReflectComponent, ReflectResource};
use bevy::reflect::{DynamicArray, DynamicEnum, DynamicStruct, DynamicTuple, DynamicTupleStruct, DynamicVariant, Reflect, ReflectKind, ReflectMut, TypeInfo, TypePath, TypeRegistration, VariantInfo};
use bevy::reflect::TypeRegistry;
use bevy::utils::HashMap;
use tiled::{LayerType, PropertyValue, TileId};



#[derive(Debug, Clone)]
pub struct DeserializedMapProperties<const HYDRATED: bool = false> {
    pub map: DeserializedProperties,
    pub layers: HashMap<u32, DeserializedProperties>,
    pub tiles: HashMap<String, HashMap<TileId, DeserializedProperties>>,
    pub objects: HashMap<u32, DeserializedProperties>,
}

impl DeserializedMapProperties<false> {
    pub fn load(map: &tiled::Map, registry: &TypeRegistry, load_context: &mut LoadContext<'_>) -> Self {
        let map_props = DeserializedProperties::load(&map.properties, registry, load_context);
        
        let mut objects = HashMap::new();
        let mut layers = HashMap::new();
        let mut to_process = Vec::from_iter(map.layers());
        while let Some(layer) = to_process.pop() {
            layers.insert(layer.id(), DeserializedProperties::load(&layer.properties, registry, load_context));
            match layer.layer_type() {
                LayerType::Objects(object) => {
                    for object in object.objects() {
                        objects.insert(object.id(), DeserializedProperties::load(&object.properties, registry, load_context));
                    }
                }
                LayerType::Group(group) => { to_process.extend(group.layers()); }
                _ => {}
            }
        }
        
        let tiles = map.tilesets().iter()
            .map(|s| (s.name.clone(), s.tiles()
                .map(|(id, t)| (id, DeserializedProperties::load(&t.properties, registry, load_context)))
                .collect()
            ))
            .collect();
        
        Self {
            map: map_props,
            layers,
            tiles,
            objects,
        }
    }
    
    pub fn hydrate(mut self, entity_map: &HashMap<u32, Entity>) -> DeserializedMapProperties<true> {
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

/// Properties for an entity deserialized from a [`Properties`](tiled::Properties)
#[derive(Debug)]
pub struct DeserializedProperties {
    pub properties: Vec<Box<dyn Reflect>>,
}

impl Clone for DeserializedProperties {
    fn clone(&self) -> Self {
        Self {
            properties: self.properties.iter().map(|r| r.clone_value()).collect(),
        }
    }
}

impl DeserializedProperties {
    pub fn load(properties: &tiled::Properties, registry: &TypeRegistry, load_cx: &mut LoadContext<'_>) -> Self {
        let mut props: Vec<Box<dyn Reflect>> = Vec::new();
        
        for (name, property) in properties.clone() {
            let PropertyValue::ClassValue {
                property_type, properties: _
            } = &property else {
                if let PropertyValue::FileValue(file) = &property {
                    props.push(Box::new(load_cx.loader().untyped().load(file)));
                    continue;
                }
                
                bevy::log::warn!("error deserializing property: unknown property `{name}`:`{property:?}`");
                continue;
            };
            
            let Some(reg) = registry.get_with_type_path(property_type) else {
                bevy::log::error!("error deserializing property: `{property_type}` is not registered in the TypeRegistry.");
                continue;
            };
            
            let is_component;
            
            if reg.data::<ReflectComponent>().is_some() ||
                reg.data::<ReflectBundle>().is_some() {
                is_component = true;
            } else if reg.data::<ReflectResource>().is_some() {
                is_component = false;
            } else {
                bevy::log::warn!("error deserializing property: type `{property_type}` is not registered as a Component, Bundle, or Resource");
                continue;
            }

            match Self::deserialize_property(property, reg, registry, load_cx) {
                Ok(prop) => {
                    props.push(prop);
                }
                Err(e) => {
                    bevy::log::error!("error deserializing property: {e}");
                }
            }
        }
        
        Self {
            properties: props,
        }
    }
    
    pub fn deserialize_property(
        property: PropertyValue,
        registration: &TypeRegistration,
        registry: &TypeRegistry,
        load_cx: &mut LoadContext<'_>
    ) -> Result<Box<dyn Reflect>, String> {
        // I wonder if it's possible to call FromStr for String?
        // or ToString/Display?
        use PropertyValue as PV;
        match (registration.type_info().type_path(), property, registration.type_info()) {
            ("bool", PV::BoolValue(b), _) => Ok(Box::new(b)),
            
            ("i8", PV::IntValue(i), _) => Ok(Box::new(i8::try_from(i).unwrap())),
            ("i16", PV::IntValue(i), _) => Ok(Box::new(i16::try_from(i).unwrap())),
            ("i32", PV::IntValue(i), _) => Ok(Box::new(i)),
            ("i64", PV::IntValue(i), _) => Ok(Box::new(i as i64)),
            ("i128", PV::IntValue(i), _) => Ok(Box::new(i as i128)),
            ("u8", PV::IntValue(i), _) => Ok(Box::new(u8::try_from(i).unwrap())),
            ("u16", PV::IntValue(i), _) => Ok(Box::new(u16::try_from(i).unwrap())),
            ("u32", PV::IntValue(i), _) => Ok(Box::new(u32::try_from(i).unwrap())),
            ("u64", PV::IntValue(i), _) => Ok(Box::new(u64::try_from(i).unwrap())),
            ("u128", PV::IntValue(i), _) => Ok(Box::new(u128::try_from(i).unwrap())),
            
            ("f32", PV::FloatValue(f), _) => Ok(Box::new(f)),
            ("f64", PV::FloatValue(f), _) => Ok(Box::new(f as f64)),
            // Shouldn't need these but it's a backup
            ("f32", PV::IntValue(i), _) => Ok(Box::new(i as f32)),
            ("f64", PV::IntValue(i), _) => Ok(Box::new(i as f64)),

            ("bevy_color::color::Color", PV::ColorValue(c), _) => {
                Ok(Box::new(Color::srgba_u8(c.red, c.green, c.blue, c.alpha)))
            },
            
            ("std::alloc::String", PV::StringValue(s), _) => Ok(Box::new(s)),
            ("char", PV::StringValue(s), _) => Ok(Box::new(s.chars().next().unwrap())),
            ("std::path::PathBuf", PV::FileValue(s), _) => Ok(Box::new(PathBuf::from(s))),
            (a, PV::FileValue(s), _) if a.starts_with("bevy_asset::handle::Handle") => {
                Ok(Box::new(load_cx.loader().untyped().load(s)))
            },
            ("bevy_ecs::entity::Entity", PV::ObjectValue(o), _) => {
                if o == 0 {
                    Err("empty object reference".to_string())
                } else {
                    Ok(Box::new(Entity::from_raw(o)))
                }
            },
            ("core::option::Option<bevy_ecs::entity::Entity>", PV::ObjectValue(o), _) => {
                Ok(Box::new(Some(Entity::from_raw(o)).filter(|_| o != 0)))
            },
            (_, PV::StringValue(s), TypeInfo::Enum(info)) => {
                let Some(variant) = info.variant(&s) else {
                    return Err(format!("no variant `{}` for `{}`", s, info.type_path()));
                };
                
                let VariantInfo::Unit(unit_info) = variant else {
                    return Err(format!("variant `{}` is not a unit variant of `{}`", s, info.type_path()));
                };

                let mut out = DynamicEnum::new(unit_info.name(), DynamicVariant::Unit);
                out.set_represented_type(Some(registration.type_info()));
                
                Ok(Box::new(out))
            },
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::Struct(info)) => {
                let mut out = DynamicStruct::default();
                out.set_represented_type(Some(registration.type_info()));

                for field in info.iter() {
                    let Some(pv) = properties.remove(field.name()) else {
                        return Err(format!("missing property on `{}`: `{}`", info.type_path(), field.name()));
                    };

                    let Some(reg) = registry.get(field.type_id()) else {
                        return Err(format!("type `{}` is not registered", field.type_path()));
                    };

                    let value = Self::deserialize_property(pv, reg, registry, load_cx)?;

                    out.insert_boxed(field.name(), value);
                }

                Ok(Box::new(out))
            },
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::TupleStruct(info)) => {
                let mut out = DynamicTupleStruct::default();
                out.set_represented_type(Some(registration.type_info()));

                for field in info.iter() {
                    let Some(pv) = properties.remove(&field.index().to_string()) else {
                        return Err(format!("missing property on `{}`: `{}`", info.type_path(), field.index().to_string()));
                    };

                    let Some(reg) = registry.get(field.type_id()) else {
                        return Err(format!("type `{}` is not registered", field.type_path()));
                    };

                    let value = Self::deserialize_property(pv, reg, registry, load_cx)?;

                    out.insert_boxed(value);
                }

                Ok(Box::new(out))
            },
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::Tuple(info)) => {
                let mut out = DynamicTuple::default();
                out.set_represented_type(Some(registration.type_info()));

                for field in info.iter() {
                    let Some(pv) = properties.remove(&field.index().to_string()) else {
                        return Err(format!("missing property on `{}`: `{}`", info.type_path(), field.index().to_string()));
                    };

                    let Some(reg) = registry.get(field.type_id()) else {
                        return Err(format!("type `{}` is not registered", field.type_path()));
                    };

                    let value = Self::deserialize_property(pv, reg, registry, load_cx)?;

                    out.insert_boxed(value);
                }

                Ok(Box::new(out))
            },
            (_, PV::ClassValue { mut properties, .. }, TypeInfo::Array(info)) => {
                let mut array = Vec::new();

                let Some(reg) = registry.get(info.item_type_id()) else {
                    return Err(format!("type `{}` is not registered", info.item_type_path_table().path()));
                };

                for i in 0..array.capacity() {
                    let Some(pv) = properties.remove(&format!("[{}]", i)) else {
                        return Err(format!("missing property on `{}`: `{}`", info.type_path(), format!("[{}]", i)));
                    };

                    let value = Self::deserialize_property(pv, reg, registry, load_cx)?;

                    array.push(value);
                }
                
                let mut out = DynamicArray::new(array.into());
                out.set_represented_type(Some(registration.type_info()));

                Ok(Box::new(out))
            },
            (_, PV::ClassValue { .. }, TypeInfo::Enum(_)) => Err("enums are currently unsupported".to_string()),
            (_, PV::ClassValue { .. }, TypeInfo::List(_)) => Err("lists are currently unsupported".to_string()),
            (_, PV::ClassValue { .. }, TypeInfo::Map(_)) => Err("maps are currently unsupported".to_string()),
            // Note: ClassValue and TypeInfo::Value is not included
            
            (a, b, _) => {
                Err(format!("unable to deserialize `{a}` from {b:?}"))
            }
        }
    }
    
    pub fn hydrate(&mut self, obj_entity_map: &HashMap<u32, Entity>) {
        for resource in self.properties.iter_mut() {
            hydrate(resource.as_mut(), obj_entity_map);
        }
    }
}

fn object_ref(obj: &dyn Reflect, obj_entity_map: &HashMap<u32, Entity>) -> Option<Box<dyn Reflect>> {
    if obj.represents::<Entity>() {
        let obj = Entity::take_from_reflect(obj.clone_value()).unwrap();
        if let Some(&e) = obj_entity_map.get(&obj.index()) {
            Some(Box::new(e))
        } else {
            panic!("error hydrating properties: missing entity for object {}", obj.index());
        }
    } else if obj.represents::<Option<Entity>>() {
        // maybe the map get should panic actually
        Some(Box::new(Option::<Entity>::take_from_reflect(obj.clone_value()).unwrap()
            .and_then(|obj| obj_entity_map.get(&obj.index()).copied() )))
    } else {
        None
    }
}

#[test]
fn clone_s() {
    println!("{}", Color::type_path());
}

fn hydrate(object: &mut dyn Reflect, obj_entity_map: &HashMap<u32, Entity>) {
    if let Some(obj) = object_ref(object, obj_entity_map) {
        object.apply(&*obj);
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
        ReflectMut::Enum(s) => {
            for i in 0..s.field_len() {
                hydrate(s.field_at_mut(i).unwrap(), obj_entity_map);
            }
        }
        ReflectMut::Map(s) => {
            // todo: 
            for i in 0..s.len() {
                let (k, v) = s.get_at_mut(i).unwrap();
                if object_ref(k, obj_entity_map).is_some() {
                    panic!("Unable to hydrate a key in a map!");
                }
                hydrate(v, obj_entity_map);
            }
        }
        // we don't care about any of the other values
        ReflectMut::Value(_) => {}
    }
}



