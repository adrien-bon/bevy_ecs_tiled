use bevy::prelude::{AppTypeRegistry, ReflectComponent, Res};
use bevy::prelude::ReflectDefault;
use std::collections::hash_map::Entry;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::ops::Deref;
use bevy::core::Name;
use bevy::prelude::{Component, World};
use bevy::reflect::{ArrayInfo, EnumInfo, GetTypeRegistration, Reflect, StructInfo, TupleInfo, TupleStructInfo, TypeInfo, TypeRegistration, TypeRegistry, UnitVariantInfo, VariantInfo};
use bevy::utils::hashbrown::HashMap;
use serde_json::{json, Value};
use thiserror::Error;
use tiled::{Layer, Properties, PropertyValue};
use crate::properties::import::{Class, Enum, FieldType, Member, StorageType, TypeData, TypeImport, UseAs};

#[derive(Reflect)]
pub enum TestData {
    A(i32),
}

// struct A;

#[test]
fn test_serialize_handle() {
    let mut world = World::new();

}

#[test]
fn load() {
    let map = tiled::Loader::new().load_tmx_map("assets/colliders_and_user_properties.tmx").unwrap();

    let mut world = World::new();

    let object_entity_map = map.layers()
        .filter_map(Layer::as_object_layer)
        .flat_map(|x| x.objects())
        .map(|o| {

            let id = world.spawn_empty()
                .insert(Name::new(o.name.clone()))
                .id();
            (o.id(), id)
        })
        .collect::<HashMap<_, _>>();

    // .map(|o| )//
    // let mut c = world.entity_mut(*object_entity_map.get(&1).unwrap());
    // let res = world.resource::<AppTypeRegistry>();/
    // c.insert(());



    // let x = res.0.read().deref();



    for layer in map.layers() {
        if let Some(objects) = layer.as_object_layer() {
            for object in objects.objects() {
                println!("{}: {:?}", object.name, object.properties);

            }
        }
    }
}

#[test]
fn test_reflect() {
    let mut registry = TypeRegistry::new();
    registry.register::<TestData>();
    let name = "bevy_tiled_reflect::reflect::TestData";

    let x = registry.get_with_type_path(name).unwrap()
        .type_info();
    dbg!(x);
}

#[test]
fn test_nested_tuple_struct() {
    #[derive(Reflect, Default, Debug, Component)]
    #[reflect(Default, Component)]
    struct TestOuter(TestInner);

    #[derive(Reflect, Default, Debug)]
    struct TestInner(i32, [f32; 3]);

    #[derive(Reflect, Default)]
    struct TestStruct {
        a: i32,
        b: String,
    }

    #[derive(Reflect)]
    enum TestVariant {
        Apples, Bananas, Oranges
    }

    let mut registry = TypeRegistry::new();
    registry.register::<TestOuter>();
    registry.register::<TestInner>();
    registry.register::<TestStruct>();
    registry.register::<TestVariant>();

    let mut imports = TypeImportRegistry::from_registry(&registry);

    println!("{}", serde_json::to_string(&imports.to_vec()).unwrap());

    // let test_value = json!({
    //     "bevy_tiled_reflect::reflect::TestOuter"
    // });

    // let test_input = TestOuter(TestInner(12, [1., 2.0, 3.0]));
    // 
    // let deserializer = ReflectDeserializer::new(&registry);
    // let serializer = ReflectSerializer::new(&test_input, &registry);
    // let mut inner = DynamicTupleStruct::default();
    // inner.set_represented_type(Some(TestInner::type_info()));
    // inner.insert(12);
    // inner.insert([1f32, 2.0, 3.0]);
    // // inner.set
    // let mut outer = DynamicTupleStruct::default();
    // outer.set_represented_type(Some(TestOuter::type_info()));
    // // outer.insert(inner);
    // dbg!(test_input.as_reflect());
    // // println!("{}", ron::to_string(&serializer).unwrap());
    // dbg!(&outer);
    // dbg!(TestOuter::from_reflect(&outer));
    // 
    // 
    // 
    // 
    // 
    // let mut x= registry.get(TestOuter::type_info().type_id()).unwrap()
    //     .data::<ReflectComponent>().unwrap();
    // 
    // let mut world = World::new();
    // let mut ent = world.spawn_empty();
    // x.insert(&mut ent, outer.as_reflect(), &registry);
    // let i = ent.id();
    // dbg!(world.entity(i).get::<TestOuter>());
    // dbg!(world);



    // 
    // let reg = registry.get(TestOuter::type_info().type_id()).unwrap();
    // // reg.data::<Deserializ>
    // // reg.
    // 
    // dbg!(&x);
    // x.apply(&outer);
    // dbg!(&x);

    // dbg!(deserializer.deserialize(test_value));
}

pub fn export_types(
    reg: Res<AppTypeRegistry>,
) {
    // todo: use a config
    
    let mut path = env::current_dir().unwrap();
    path.push("tiled_type_export.json");
    
    let file = File::create(path).unwrap();
    let writer = BufWriter::new(file);
    
    let registry = TypeImportRegistry::from_registry(reg.0.read().deref());
    
    serde_json::to_writer(writer, &registry.to_vec()).unwrap();
}

#[derive(Debug, Default, Clone)]
pub struct TypeImportRegistry {
    types: HashMap<&'static str, TypeImport>,
    id: u32,
}

impl TypeImportRegistry {
    pub fn from_registry(registry: &TypeRegistry) -> Self {
        let mut out = Self::default();

        for t in registry.iter() {
            out.register_from_type_registration(t, registry);
        }

        out
    }

    pub fn register<T: Reflect + GetTypeRegistration>(&mut self, registry: &TypeRegistry) {
        self.register_from_type_registration(&T::get_type_registration(), registry)
    }

    pub fn register_from_type_registration(&mut self, registration: &TypeRegistration, registry: &TypeRegistry) {
        match self.generate_import(registration, registry) {
            Ok(import) => {
                for import in import {
                    self.types.insert(registration.type_info().type_path(), import);
                }
            }
            Err(_) => {
                self.remove_with_dependency(registration.type_info().type_path());
            }
        }
    }

    fn generate_import(&mut self, registration: &TypeRegistration, registry: &TypeRegistry)
                       -> ImportConversionResult {
        match registration.type_info() {
            TypeInfo::TupleStruct(info) => self.generate_tuple_struct_import(info, registry),
            TypeInfo::Struct(info) => self.generate_struct_import(info, registry),
            TypeInfo::Tuple(info) => self.generate_tuple_import(info, registry),
            TypeInfo::List(_) => Err(ImportConversionError::ListUnsupported),
            TypeInfo::Array(info) => self.generate_array_import(info, registry),
            TypeInfo::Map(_) => Err(ImportConversionError::ListUnsupported),
            TypeInfo::Enum(info) => self.generate_enum_import(info, registry),
            TypeInfo::Value(_) => Ok(vec![])
        }
    }

    fn next_id(&mut self) -> u32 {
        self.id += 1;
        self.id
    }

    pub fn to_vec(self) -> Vec<TypeImport> {
        let mut out = self.types.into_values().collect::<Vec<_>>();

        out.sort_by(|a, b| a.name.cmp(&b.name));

        out
    }

    pub fn remove_with_dependency(&mut self, type_path: &str) {
        let mut to_remove = vec![type_path.to_string()];
        while let Some(type_path) = to_remove.pop() {
            self.types.retain(|_, import| {
                match &import.type_data {
                    TypeData::Enum(_) => true,
                    TypeData::Class(class) => {
                        if class.members.iter()
                            .any(|m| m.property_type.as_ref().is_some_and(|s| s.as_str() == type_path)) {
                            to_remove.push(import.name.clone());
                            false
                        } else {
                            true
                        }
                    }
                }
            })
        }
    }

    fn generate_tuple_struct_import(&mut self, info: &TupleStructInfo, registry: &TypeRegistry)
                                    -> ImportConversionResult {

        let root = TypeImport {
            id: self.next_id(),
            name: info.type_path().to_string(),
            type_data: TypeData::Class(Class {
                use_as: UseAs::all_supported(),
                color: "#000000".to_string(),
                draw_fill: true,
                members: info.iter()
                    .map(|s| {
                        let (type_field, property_type) =
                            type_to_field(registry.get(s.type_id()).unwrap())?;

                        Ok(Member {
                            name: s.index().to_string(),
                            property_type,
                            type_field,
                            value: Default::default(),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            }),
        };

        Ok(vec![root])
    }

    fn generate_array_import(&mut self, info: &ArrayInfo, registry: &TypeRegistry)
                             -> ImportConversionResult {
        let (type_field, property_type) =
            type_to_field(registry.get(info.item_type_id()).unwrap())?;

        let root = TypeImport {
            id: self.next_id(),
            name: info.type_path().to_string(),
            type_data: TypeData::Class(Class {
                use_as: UseAs::all_supported(),
                color: "#000000".to_string(),
                draw_fill: true,
                members: (0..info.capacity())
                    .map(|i| {
                        Member {
                            name: format!("[{i}]"),
                            property_type: property_type.clone(),
                            type_field,
                            value: Default::default(),
                        }
                    })
                    .collect(),
            }),
        };

        Ok(vec![root])
    }

    fn generate_tuple_import(&mut self, info: &TupleInfo, registry: &TypeRegistry)
                             -> ImportConversionResult {
        let root = TypeImport {
            id: self.next_id(),
            name: info.type_path().to_string(),
            type_data: TypeData::Class(Class {
                use_as: UseAs::all_supported(),
                color: "#000000".to_string(),
                draw_fill: true,
                members: info.iter()
                    .map(|s| {
                        let (type_field, property_type) =
                            type_to_field(registry.get(s.type_id()).unwrap())?;

                        Ok(Member {
                            name: s.index().to_string(),
                            property_type,
                            type_field,
                            value: Default::default(),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            }),
        };

        Ok(vec![root])
    }

    fn generate_struct_import(&mut self, info: &StructInfo, registry: &TypeRegistry)
                              -> ImportConversionResult {

        let root = TypeImport {
            id: self.next_id(),
            name: info.type_path().to_string(),
            type_data: TypeData::Class(Class {
                use_as: UseAs::all_supported(),
                color: "#000000".to_string(),
                draw_fill: true,
                members: info.iter()
                    .map(|s| {
                        let (type_field, property_type) =
                            type_to_field(registry.get(s.type_id()).unwrap())?;

                        Ok(Member {
                            name: s.name().to_string(),
                            property_type,
                            type_field,
                            value: Default::default(),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            }),
        };

        Ok(vec![root])
    }

    fn generate_enum_import(&mut self, info: &EnumInfo, registry: &TypeRegistry) -> ImportConversionResult {
        let simple = info.iter()
            .all(|s| matches!(s, VariantInfo::Unit(_)));

        if simple {
            Ok(vec![TypeImport {
                id: self.next_id(),
                name: info.type_path().to_string(),
                type_data: TypeData::Enum(Enum {
                    storage_type: StorageType::String,
                    values_as_flags: false,
                    values: info.iter()
                        .map(|s| s.name().to_string())
                        .collect(),
                }),
            }])
        } else {
            Err(ImportConversionError::UnsupportedValue(info.type_path()))
        }
    }


    pub fn generate_default(&self, type_path: &str) -> Result<PropertyValue, ()> {


        todo!()
    }
}

fn insert_value(a: &mut PropertyValue, b: PropertyValue) {
    use PropertyValue as PV;
    match (a, b) {
        (PV::BoolValue(a), PV::BoolValue(b)) => { *a = b; },
        (PV::FloatValue(a), PV::FloatValue(b)) => { *a = b; },
        (PV::IntValue(a), PV::IntValue(b)) => { *a = b; },
        (PV::ColorValue(a), PV::ColorValue(b)) => { *a = b; },
        (PV::StringValue(a), PV::StringValue(b)) => { *a = b; },
        (PV::FileValue(a), PV::FileValue(b)) => { *a = b; },
        (PV::ObjectValue(a), PV::ObjectValue(b)) => { *a = b; },
        (
            PV::ClassValue {
                property_type: property_type_a,
                properties: properties_a,
            },
            PV::ClassValue {
                property_type: property_type_b,
                properties: properties_b,
            }
        ) => {
            assert_eq!(property_type_a, &property_type_b);

            for (name, value) in properties_b {
                match properties_a.entry(name) {
                    Entry::Occupied(mut a) => {
                        insert_value(a.get_mut(), value);
                    }
                    Entry::Vacant(a) => {
                        a.insert(value);
                    }
                }
            }
        }
        _ => { panic!("mismatched property values"); }
    }
}

#[test]
fn test_insert_value() {
    let mut a = PropertyValue::ClassValue {
        property_type: "[f32; 3]".to_string(),
        properties: Properties::from([
            ("[0]".to_string(), PropertyValue::IntValue(0)),
            ("[1]".to_string(), PropertyValue::IntValue(0)),
            ("[2]".to_string(), PropertyValue::IntValue(0)),
        ]),
    };

    let b = PropertyValue::ClassValue {
        property_type: "[f32; 3]".to_string(),
        properties: Properties::from([
            ("[1]".to_string(), PropertyValue::IntValue(1)),
        ]),
    };

    let expected = PropertyValue::ClassValue {
        property_type: "[f32; 3]".to_string(),
        properties: Properties::from([
            ("[0]".to_string(), PropertyValue::IntValue(0)),
            ("[1]".to_string(), PropertyValue::IntValue(1)),
            ("[2]".to_string(), PropertyValue::IntValue(0)),
        ]),
    };

    let actual = insert_value(&mut a, b);
    assert_eq!(a, expected);
}

fn insert_json_value(property: &mut PropertyValue, json: serde_json::Value) {
    use PropertyValue as PV;
    match (property, json) {
        (PV::BoolValue(a), Value::Bool(b)) => { *a = b; },
        (PV::FloatValue(a), Value::Number(b)) => { *a = b.as_f64().unwrap() as f32; }
        (PV::IntValue(a), Value::Number(b)) => { *a = b.as_f64().unwrap() as i32; }
        (PV::ColorValue(a), Value::String(b)) => { *a = b.parse().unwrap(); }
        (PV::StringValue(a), Value::String(b)) => { *a = b; }
        (PV::FileValue(a), Value::String(b)) => { *a = b; }
        (PV::ObjectValue(a), Value::Number(b)) => { *a = b.as_u64().unwrap() as u32; }
        (PV::ClassValue { property_type: _, properties }, Value::Object(b)) => {
            for (name, value) in b {
                if let Some(property) = properties.get_mut(&name) {
                    insert_json_value(property, value);
                }
            }
        }
        (a, b) => { panic!("mismatched property values: {:?} vs {:?}", a, b); }
    }
}


#[test]
fn test_insert_json_value() {
    let mut a = PropertyValue::ClassValue {
        property_type: "[f32; 3]".to_string(),
        properties: Properties::from([
            ("[0]".to_string(), PropertyValue::IntValue(0)),
            ("[1]".to_string(), PropertyValue::IntValue(0)),
            ("[2]".to_string(), PropertyValue::IntValue(0)),
        ]),
    };

    let b = json!({
        "[1]": 1,
        "[2]": 2
    });

    let expected = PropertyValue::ClassValue {
        property_type: "[f32; 3]".to_string(),
        properties: Properties::from([
            ("[0]".to_string(), PropertyValue::IntValue(0)),
            ("[1]".to_string(), PropertyValue::IntValue(1)),
            ("[2]".to_string(), PropertyValue::IntValue(2)),
        ]),
    };

    let actual = insert_json_value(&mut a, b);
    assert_eq!(a, expected);
}

type ImportConversionResult = Result<Vec<TypeImport>, ImportConversionError>;

// #[derive(Default)]
// struct IdGen(u32);
// 
// impl IdGen {
//     pub fn 
// }

#[derive(Debug, Eq, PartialEq, Copy, Clone, Error)]
enum ImportConversionError {
    #[error("lists fields are not supported")]
    ListUnsupported,
    #[error("map fields are not supported")]
    MapUnsupported,
    #[error("field of type {0} is not supported")]
    UnsupportedValue(&'static str)
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct ListUnsupported;

fn type_to_field(t: &TypeRegistration) -> Result<(FieldType, Option<String>), ImportConversionError> {
    let info = t.type_info();
    if matches!(info, TypeInfo::List(_)) {
        return Err(ImportConversionError::ListUnsupported);
    } else if matches!(info, TypeInfo::Map(_)) {
        return Err(ImportConversionError::MapUnsupported);
    }
    Ok(match info.type_path() {
        "bool" => (FieldType::Bool, None),
        "f32" | "f64" => (FieldType::Float, None),
        
        "isize" | "i8" | "i16" | "i32" | "i64" | "i128" |
        "usize" | "u8" | "u16" | "u32" | "u64" | "u128" => (FieldType::Int, None),
        
        "bevy_ecs::entity::Entity" | "core::option::Option<bevy_ecs::entity::Entity>" => (FieldType::Object, None),
        "alloc::borrow::Cow<str>" | "alloc::string::String" | "char" => (FieldType::String, None),
        
        "bevy_color::color::Color" => (FieldType::Color, None),
        "std::path::PathBuf" => (FieldType::File, None),
        f if f.starts_with("bevy_asset::handle::Handle") => (FieldType::File, None),
        path => {
            if matches!(info, TypeInfo::Value(_)) {
                return Err(ImportConversionError::UnsupportedValue(info.type_path()));
            }

            (if is_enum_and_simple(t) {
                FieldType::String
            } else {
                FieldType::Class
            }, Some(path.to_string()))
        }
    })
}

fn unit_variant_to_import(info: &UnitVariantInfo, id: u32) -> TypeImport {
    unit_import(id, info.name().to_string())
}

fn is_enum_and_simple(t: &TypeRegistration) -> bool {
    match t.type_info() {
        TypeInfo::Enum(info) => {
            info.iter()
                .all(|variant| matches!(variant, VariantInfo::Unit(_)))
        },
        _ => false
    }
}

const UNIT_COLOR: &str = "#00aa00";

pub fn unit_import(id: u32, name: String) -> TypeImport {
    TypeImport {
        id,
        name,
        type_data: TypeData::Class(Class {
            use_as: UseAs::all_supported(),
            color: UNIT_COLOR.to_string(),
            draw_fill: true,
            members: vec![],
        })
    }
}
