use crate::properties::types_json::{
    Class, Enum, FieldType, Member, StorageType, TypeData, TypeExport, UseAs,
};
use bevy::ecs::reflect::ReflectBundle;
use bevy::reflect::{
    ArrayInfo, EnumInfo, NamedField, StructInfo, TupleInfo, TupleStructInfo, TypeInfo,
    TypeRegistration, TypeRegistry, UnitVariantInfo, UnnamedField, VariantInfo,
};
use bevy::utils::hashbrown::HashMap;
use bevy::{prelude::*, reflect::ReflectRef};
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use thiserror::Error;
use tiled::PropertyValue;

const UNIT_COLOR: &str = "#00aa00";
const DEFAULT_COLOR: &str = "#000000";

type ExportConversionResult = Result<Vec<TypeExport>, ExportConversionError>;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Error)]
enum ExportConversionError {
    #[error("lists fields are not supported")]
    ListUnsupported,
    #[error("map fields are not supported")]
    MapUnsupported,
    #[error("field of type {0} is not supported")]
    UnsupportedValue(&'static str),
    //#[error("type {0} does not reflect Component, Bundle or Resource")]
    //NotReflectable(&'static str),
}

#[derive(Debug, Default, Clone)]
pub(crate) struct TypeExportRegistry {
    types: HashMap<&'static str, TypeExport>,
    id: u32,
}

impl TypeExportRegistry {
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_vec(self) -> Vec<TypeExport> {
        let mut out = self.types.into_values().collect::<Vec<_>>();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    pub(crate) fn from_registry(registry: &TypeRegistry) -> Self {
        let mut out = Self::default();
        for t in registry.iter() {
            out.register_from_type_registration(t, registry);
        }
        out
    }

    fn next_id(&mut self) -> u32 {
        self.id += 1;
        self.id
    }

    fn register_from_type_registration(
        &mut self,
        registration: &TypeRegistration,
        registry: &TypeRegistry,
    ) {
        match self.generate_export(registration, registry) {
            Ok(export) => {
                for export in export {
                    self.types
                        .insert(registration.type_info().type_path(), export);
                }
            }
            Err(_) => {
                self.remove_with_dependency(registration.type_info().type_path());
            }
        }
    }

    fn generate_export(
        &mut self,
        registration: &TypeRegistration,
        registry: &TypeRegistry,
    ) -> ExportConversionResult {
        let mut default_value = None;
        let tmp;
        let v = registration.data::<ReflectDefault>().map(|v| v.default());
        if v.is_some() {
            tmp = v.unwrap();
            default_value = Some(tmp.as_ref());
        }

        let use_as;
        if registration.data::<ReflectResource>().is_some() {
            use_as = vec![UseAs::Map];
        } else if registration.data::<ReflectComponent>().is_some()
            || registration.data::<ReflectBundle>().is_some()
        {
            use_as = UseAs::all_supported();
        } else {
            // TODO: right now, we need to export all registered types even if we won't be able to use them
            // We will change that when we have implemented a way to recursively add types
            use_as = vec![];
            // return Err(ExportConversionError::NotReflectable(
            //    registration.type_info().type_path(),
            // ));
        }

        match registration.type_info() {
            TypeInfo::TupleStruct(info) => {
                self.generate_tuple_struct_export(info, registry, default_value)
            }
            TypeInfo::Struct(info) => {
                self.generate_struct_export(info, registry, default_value, use_as)
            }
            TypeInfo::Tuple(info) => {
                self.generate_tuple_export(info, registry, default_value, use_as)
            }
            TypeInfo::List(_) => Err(ExportConversionError::ListUnsupported),
            TypeInfo::Array(info) => self.generate_array_export(info, registry, use_as),
            TypeInfo::Map(_) => Err(ExportConversionError::MapUnsupported),
            TypeInfo::Enum(info) => self.generate_enum_export(info, registry),
            TypeInfo::Value(_) => Ok(vec![]),
        }
    }

    fn remove_with_dependency(&mut self, type_path: &str) {
        let mut to_remove = vec![type_path.to_string()];
        while let Some(type_path) = to_remove.pop() {
            self.types.retain(|_, export| match &export.type_data {
                TypeData::Enum(_) => true,
                TypeData::Class(class) => {
                    if class.members.iter().any(|m| {
                        m.property_type
                            .as_ref()
                            .is_some_and(|s| s.as_str() == type_path)
                    }) {
                        to_remove.push(export.name.clone());
                        false
                    } else {
                        true
                    }
                }
            })
        }
    }

    fn generate_tuple_struct_export(
        &mut self,
        info: &TupleStructInfo,
        registry: &TypeRegistry,
        default_value: Option<&dyn Reflect>,
    ) -> ExportConversionResult {
        let root = TypeExport {
            id: self.next_id(),
            name: info.type_path().to_string(),
            type_data: TypeData::Class(Class {
                use_as: UseAs::all_supported(),
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: info
                    .iter()
                    .map(|s| {
                        let (type_field, property_type) =
                            type_to_field(registry.get(s.type_id()).unwrap())?;
                        Ok(Member {
                            name: s.index().to_string(),
                            property_type,
                            type_field,
                            value: unnamed_field_json_value(default_value, s),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            }),
        };

        Ok(vec![root])
    }

    fn generate_array_export(
        &mut self,
        info: &ArrayInfo,
        registry: &TypeRegistry,
        use_as: Vec<UseAs>,
    ) -> ExportConversionResult {
        let (type_field, property_type) =
            type_to_field(registry.get(info.item_type_id()).unwrap())?;

        let root = TypeExport {
            id: self.next_id(),
            name: info.type_path().to_string(),
            type_data: TypeData::Class(Class {
                use_as,
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: (0..info.capacity())
                    .map(|i| Member {
                        name: format!("[{i}]"),
                        property_type: property_type.clone(),
                        type_field,
                        value: Default::default(),
                    })
                    .collect(),
            }),
        };

        Ok(vec![root])
    }

    fn generate_tuple_export(
        &mut self,
        info: &TupleInfo,
        registry: &TypeRegistry,
        default_value: Option<&dyn Reflect>,
        use_as: Vec<UseAs>,
    ) -> ExportConversionResult {
        let root = TypeExport {
            id: self.next_id(),
            name: info.type_path().to_string(),
            type_data: TypeData::Class(Class {
                use_as,
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: info
                    .iter()
                    .map(|s| {
                        let (type_field, property_type) =
                            type_to_field(registry.get(s.type_id()).unwrap())?;
                        Ok(Member {
                            name: s.index().to_string(),
                            property_type,
                            type_field,
                            value: unnamed_field_json_value(default_value, s),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            }),
        };

        Ok(vec![root])
    }

    fn generate_struct_export(
        &mut self,
        info: &StructInfo,
        registry: &TypeRegistry,
        default_value: Option<&dyn Reflect>,
        use_as: Vec<UseAs>,
    ) -> ExportConversionResult {
        let root = TypeExport {
            id: self.next_id(),
            name: info.type_path().to_string(),
            type_data: TypeData::Class(Class {
                use_as,
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: info
                    .iter()
                    .map(|s| {
                        let (type_field, property_type) =
                            type_to_field(registry.get(s.type_id()).unwrap())?;
                        Ok(Member {
                            name: s.name().to_string(),
                            property_type,
                            type_field,
                            value: named_field_json_value(default_value, s),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            }),
        };

        Ok(vec![root])
    }

    fn generate_enum_export(
        &mut self,
        info: &EnumInfo,
        _registry: &TypeRegistry,
    ) -> ExportConversionResult {
        let simple = info.iter().all(|s| matches!(s, VariantInfo::Unit(_)));

        if simple {
            Ok(vec![TypeExport {
                id: self.next_id(),
                name: info.type_path().to_string(),
                type_data: TypeData::Enum(Enum {
                    storage_type: StorageType::String,
                    values_as_flags: false,
                    values: info.iter().map(|s| s.name().to_string()).collect(),
                }),
            }])
        } else {
            Err(ExportConversionError::UnsupportedValue(info.type_path()))
        }
    }
}

fn value_to_json(value: &dyn Reflect) -> serde_json::Value {
    let Some(type_info) = value.get_represented_type_info() else {
        return serde_json::Value::default();
    };

    match (type_info.type_path(), type_info, value.reflect_ref()) {
        ("bool", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<bool>().unwrap()),
        ("f32", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<f32>().unwrap()),
        ("f64", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<f64>().unwrap()),
        ("isize", _, ReflectRef::Value(v)) => {
            serde_json::json!(*v.downcast_ref::<isize>().unwrap())
        }
        ("i8", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<i8>().unwrap()),
        ("i16", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<i16>().unwrap()),
        ("i32", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<i32>().unwrap()),
        ("i64", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<i64>().unwrap()),
        ("i128", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<i128>().unwrap()),
        ("usize", _, ReflectRef::Value(v)) => {
            serde_json::json!(*v.downcast_ref::<usize>().unwrap())
        }
        ("u8", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<u8>().unwrap()),
        ("u16", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<u16>().unwrap()),
        ("u32", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<u32>().unwrap()),
        ("u64", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<u64>().unwrap()),
        ("u128", _, ReflectRef::Value(v)) => serde_json::json!(*v.downcast_ref::<u128>().unwrap()),
        ("alloc::string::String", _, ReflectRef::Value(v)) => {
            serde_json::json!(*v.downcast_ref::<String>().unwrap())
        }
        ("alloc::borrow::Cow<str>", _, ReflectRef::Value(v)) => {
            serde_json::json!(*v.downcast_ref::<Cow<str>>().unwrap())
        }
        ("bevy_color::color::Color", _, _) => {
            let c = value.downcast_ref::<Color>().unwrap();
            serde_json::json!(format!("#{:08x}", c.to_linear().as_u32()))
        }
        (_, TypeInfo::Enum(info), ReflectRef::Enum(v)) => {
            if info.iter().all(|v| matches!(v, VariantInfo::Unit(_))) {
                serde_json::json!(v.variant_name())
            } else {
                // TODO: non-unit enums
                serde_json::Value::default()
            }
        }
        (_, TypeInfo::Struct(info), _) => info
            .iter()
            .map(|s| (s.name(), named_field_json_value(Some(value), s)))
            .collect(),
        (_, TypeInfo::Tuple(info), _) => info
            .iter()
            .map(|s| {
                (
                    s.index().to_string(),
                    unnamed_field_json_value(Some(value), s),
                )
            })
            .collect(),
        (_, TypeInfo::TupleStruct(info), _) => info
            .iter()
            .map(|s| {
                (
                    s.index().to_string(),
                    unnamed_field_json_value(Some(value), s),
                )
            })
            .collect(),
        _ => {
            warn!(
                "cannot convert type '{}' to a JSON value",
                type_info.type_path()
            );
            serde_json::Value::default()
        }
    }
}

fn named_field_json_value(value: Option<&dyn Reflect>, field: &NamedField) -> serde_json::Value {
    match value {
        Some(v) => match v.reflect_ref() {
            ReflectRef::Struct(t) => t
                .field(field.name())
                .map(value_to_json)
                .unwrap_or(serde_json::Value::default()),
            _ => serde_json::Value::default(),
        },
        _ => serde_json::Value::default(),
    }
}

fn unnamed_field_json_value(
    value: Option<&dyn Reflect>,
    field: &UnnamedField,
) -> serde_json::Value {
    match value {
        Some(v) => match v.reflect_ref() {
            ReflectRef::TupleStruct(t) => (*t)
                .field(field.index())
                .map(value_to_json)
                .unwrap_or(serde_json::Value::default()),
            ReflectRef::Tuple(t) => (*t)
                .field(field.index())
                .map(value_to_json)
                .unwrap_or(serde_json::Value::default()),
            _ => serde_json::Value::default(),
        },
        _ => serde_json::Value::default(),
    }
}

fn type_to_field(
    t: &TypeRegistration,
) -> Result<(FieldType, Option<String>), ExportConversionError> {
    let info = t.type_info();
    if matches!(info, TypeInfo::List(_)) {
        return Err(ExportConversionError::ListUnsupported);
    } else if matches!(info, TypeInfo::Map(_)) {
        return Err(ExportConversionError::MapUnsupported);
    }
    Ok(match info.type_path() {
        "bool" => (FieldType::Bool, None),
        "f32" | "f64" => (FieldType::Float, None),

        "isize" | "i8" | "i16" | "i32" | "i64" | "i128" | "usize" | "u8" | "u16" | "u32"
        | "u64" | "u128" => (FieldType::Int, None),

        "bevy_ecs::entity::Entity" | "core::option::Option<bevy_ecs::entity::Entity>" => {
            (FieldType::Object, None)
        }
        "alloc::borrow::Cow<str>" | "alloc::string::String" | "char" => (FieldType::String, None),

        "bevy_color::color::Color" => (FieldType::Color, None),
        "std::path::PathBuf" => (FieldType::File, None),
        f if f.starts_with("bevy_asset::handle::Handle") => (FieldType::File, None),
        path => {
            if matches!(info, TypeInfo::Value(_)) {
                return Err(ExportConversionError::UnsupportedValue(info.type_path()));
            }

            (
                if is_enum_and_simple(t) {
                    FieldType::String
                } else {
                    FieldType::Class
                },
                Some(path.to_string()),
            )
        }
    })
}

fn is_enum_and_simple(t: &TypeRegistration) -> bool {
    match t.type_info() {
        TypeInfo::Enum(info) => info
            .iter()
            .all(|variant| matches!(variant, VariantInfo::Unit(_))),
        _ => false,
    }
}

#[allow(dead_code)]
fn unit_variant_to_export(info: &UnitVariantInfo, id: u32) -> TypeExport {
    unit_export(id, info.name().to_string())
}

fn unit_export(id: u32, name: String) -> TypeExport {
    TypeExport {
        id,
        name,
        type_data: TypeData::Class(Class {
            use_as: UseAs::all_supported(),
            color: UNIT_COLOR.to_string(),
            draw_fill: true,
            members: vec![],
        }),
    }
}

#[allow(dead_code)]
fn insert_value(a: &mut PropertyValue, b: PropertyValue) {
    use PropertyValue as PV;
    match (a, b) {
        (PV::BoolValue(a), PV::BoolValue(b)) => {
            *a = b;
        }
        (PV::FloatValue(a), PV::FloatValue(b)) => {
            *a = b;
        }
        (PV::IntValue(a), PV::IntValue(b)) => {
            *a = b;
        }
        (PV::ColorValue(a), PV::ColorValue(b)) => {
            *a = b;
        }
        (PV::StringValue(a), PV::StringValue(b)) => {
            *a = b;
        }
        (PV::FileValue(a), PV::FileValue(b)) => {
            *a = b;
        }
        (PV::ObjectValue(a), PV::ObjectValue(b)) => {
            *a = b;
        }
        (
            PV::ClassValue {
                property_type: property_type_a,
                properties: properties_a,
            },
            PV::ClassValue {
                property_type: property_type_b,
                properties: properties_b,
            },
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
        _ => {
            panic!("mismatched property values");
        }
    }
}

#[allow(dead_code)]
fn insert_json_value(property: &mut PropertyValue, json: serde_json::Value) {
    use PropertyValue as PV;
    match (property, json) {
        (PV::BoolValue(a), serde_json::Value::Bool(b)) => {
            *a = b;
        }
        (PV::FloatValue(a), serde_json::Value::Number(b)) => {
            *a = b.as_f64().unwrap() as f32;
        }
        (PV::IntValue(a), serde_json::Value::Number(b)) => {
            *a = b.as_f64().unwrap() as i32;
        }
        (PV::ColorValue(a), serde_json::Value::String(b)) => {
            *a = b.parse().unwrap();
        }
        (PV::StringValue(a), serde_json::Value::String(b)) => {
            *a = b;
        }
        (PV::FileValue(a), serde_json::Value::String(b)) => {
            *a = b;
        }
        (PV::ObjectValue(a), serde_json::Value::Number(b)) => {
            *a = b.as_u64().unwrap() as u32;
        }
        (
            PV::ClassValue {
                property_type: _,
                properties,
            },
            serde_json::Value::Object(b),
        ) => {
            for (name, value) in b {
                if let Some(property) = properties.get_mut(&name) {
                    insert_json_value(property, value);
                }
            }
        }
        (a, b) => {
            panic!("mismatched property values: {:?} vs {:?}", a, b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Apples,
            Bananas,
            Oranges,
        }

        let mut registry = TypeRegistry::new();
        registry.register::<TestOuter>();
        registry.register::<TestInner>();
        registry.register::<TestStruct>();
        registry.register::<TestVariant>();

        let exports = TypeExportRegistry::from_registry(&registry);

        println!("{}", serde_json::to_string(&exports.to_vec()).unwrap());

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

    #[test]
    fn test_insert_value() {
        let mut a = PropertyValue::ClassValue {
            property_type: "[f32; 3]".to_string(),
            properties: tiled::Properties::from([
                ("[0]".to_string(), PropertyValue::IntValue(0)),
                ("[1]".to_string(), PropertyValue::IntValue(0)),
                ("[2]".to_string(), PropertyValue::IntValue(0)),
            ]),
        };

        let b = PropertyValue::ClassValue {
            property_type: "[f32; 3]".to_string(),
            properties: tiled::Properties::from([("[1]".to_string(), PropertyValue::IntValue(1))]),
        };

        let expected = PropertyValue::ClassValue {
            property_type: "[f32; 3]".to_string(),
            properties: tiled::Properties::from([
                ("[0]".to_string(), PropertyValue::IntValue(0)),
                ("[1]".to_string(), PropertyValue::IntValue(1)),
                ("[2]".to_string(), PropertyValue::IntValue(0)),
            ]),
        };

        insert_value(&mut a, b);
        assert_eq!(a, expected);
    }

    #[test]
    fn test_insert_json_value() {
        let mut a = PropertyValue::ClassValue {
            property_type: "[f32; 3]".to_string(),
            properties: tiled::Properties::from([
                ("[0]".to_string(), PropertyValue::IntValue(0)),
                ("[1]".to_string(), PropertyValue::IntValue(0)),
                ("[2]".to_string(), PropertyValue::IntValue(0)),
            ]),
        };

        let b = serde_json::json!({
            "[1]": 1,
            "[2]": 2
        });

        let expected = PropertyValue::ClassValue {
            property_type: "[f32; 3]".to_string(),
            properties: tiled::Properties::from([
                ("[0]".to_string(), PropertyValue::IntValue(0)),
                ("[1]".to_string(), PropertyValue::IntValue(1)),
                ("[2]".to_string(), PropertyValue::IntValue(2)),
            ]),
        };

        insert_json_value(&mut a, b);
        assert_eq!(a, expected);
    }

    #[test]
    fn test_value_to_json() {
        #[derive(Reflect, Default)]
        struct Testing {
            s: String,
            b: bool,
        }
        let t = Testing {
            s: "this is a test".to_string(),
            b: true,
        };
        let json = value_to_json(&t);
        println!("{}", serde_json::to_string(&json).unwrap());
    }

    #[test]
    fn generate_with_entity() {
        #[derive(Component, Reflect)]
        #[reflect(Component)]
        struct ComponentA(Entity);

        let mut registry = TypeRegistry::new();
        registry.register::<ComponentA>();

        let imports = TypeExportRegistry::from_registry(&registry);

        assert_eq!(
            imports.types.get(ComponentA::type_path()),
            Some(&TypeExport {
                id: 1,
                name: ComponentA::type_path().to_string(),
                type_data: TypeData::Class(Class {
                    use_as: UseAs::all_supported(),
                    color: "#000000".to_string(),
                    draw_fill: true,
                    members: vec![Member {
                        name: "0".to_string(),
                        property_type: None,
                        type_field: FieldType::Object,
                        value: Default::default(),
                    }],
                }),
            })
        );
    }

    #[test]
    fn generate_with_entity_option() {
        #[derive(Component, Reflect)]
        #[reflect(Component)]
        struct ComponentA(Option<Entity>);

        let mut registry = TypeRegistry::new();
        registry.register::<ComponentA>();

        let imports = TypeExportRegistry::from_registry(&registry);

        assert_eq!(
            imports.types.get(ComponentA::type_path()),
            Some(&TypeExport {
                id: 1,
                name: ComponentA::type_path().to_string(),
                type_data: TypeData::Class(Class {
                    use_as: UseAs::all_supported(),
                    color: "#000000".to_string(),
                    draw_fill: true,
                    members: vec![Member {
                        name: "0".to_string(),
                        property_type: None,
                        type_field: FieldType::Object,
                        value: Default::default(),
                    }],
                }),
            })
        );
    }

    #[test]
    fn generate_simple_enum() {
        #[derive(Component, Reflect)]
        #[reflect(Component)]
        enum EnumComponent {
            VarA,
            VarB,
            VarC,
        }

        let mut registry = TypeRegistry::new();
        registry.register::<EnumComponent>();

        let imports = TypeExportRegistry::from_registry(&registry);

        assert_eq!(
            imports.types.get(EnumComponent::type_path()),
            Some(&TypeExport {
                id: 1,
                name: EnumComponent::type_path().to_string(),
                type_data: TypeData::Enum(Enum {
                    storage_type: StorageType::String,
                    values: vec!["VarA".to_string(), "VarB".to_string(), "VarC".to_string(),],
                    values_as_flags: false,
                }),
            })
        );
    }
}
