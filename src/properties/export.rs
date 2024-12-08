use crate::properties::types_json::{
    Class, Enum, FieldType, Member, StorageType, TypeData, TypeExport, UseAs,
};
use bevy::ecs::reflect::ReflectBundle;
use bevy::reflect::{
    ArrayInfo, EnumInfo, NamedField, StructInfo, TupleInfo, TupleStructInfo, TypeInfo,
    TypeRegistration, TypeRegistry, UnnamedField, VariantInfo,
};
use bevy::utils::hashbrown::HashMap;
use bevy::{prelude::*, reflect::ReflectRef};
use std::borrow::Cow;
use thiserror::Error;

const DEFAULT_COLOR: &str = "#000000";
const USE_AS_PROPERTY: &[UseAs] = &[UseAs::Property];

type ExportConversionResult = Result<Vec<TypeExport>, ExportConversionError>;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Error)]
enum ExportConversionError {
    #[error("lists fields are not supported")]
    ListUnsupported,
    #[error("map fields are not supported")]
    MapUnsupported,
    #[error("field of type {0} is not supported")]
    UnsupportedValue(&'static str),
    #[error("set fields are not supported")]
    SetUnsupported,
    #[error("a dependency is not supported")]
    DependencyError,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct TypeExportRegistry {
    types: HashMap<&'static str, Vec<TypeExport>>,
    id: u32,
}

impl TypeExportRegistry {
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_vec(self) -> Vec<TypeExport> {
        let mut out = self.types.into_values().flatten().collect::<Vec<_>>();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    pub(crate) fn from_registry(registry: &TypeRegistry) -> Self {
        let mut deps = vec![];
        let mut out = Self::default();
        for t in registry.iter() {
            if t.data::<ReflectComponent>().is_some()
                || t.data::<ReflectBundle>().is_some()
                || t.data::<ReflectResource>().is_some()
            {
                let mut new_deps =
                    out.register_from_type_registration(t, registry, USE_AS_PROPERTY.to_vec());
                deps.append(&mut new_deps);
            }
        }

        // We should have a dedicated 'useAs' flags so we cannot add these dependencies
        // directly as objects properties (only usable nested)
        for d in deps {
            if out.types.contains_key(d) {
                continue;
            }
            if let Some(t) = registry.get_with_type_path(d) {
                out.register_from_type_registration(t, registry, USE_AS_PROPERTY.to_vec());
            }
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
        use_as: Vec<UseAs>,
    ) -> Vec<&'static str> {
        let mut deps = vec![];
        match self.generate_export(registration, registry, use_as, &mut deps) {
            Ok(export) => {
                if !export.is_empty() {
                    self.types
                        .insert(registration.type_info().type_path(), export);
                }
                deps
            }
            Err(_) => {
                self.remove_with_dependency(registration.type_info().type_path());
                vec![]
            }
        }
    }

    fn is_supported(registration: &TypeRegistration) -> bool {
        matches!(
            registration.type_info(),
            TypeInfo::TupleStruct(_)
                | TypeInfo::Struct(_)
                | TypeInfo::Tuple(_)
                | TypeInfo::Array(_)
                | TypeInfo::Enum(_)
                | TypeInfo::Opaque(_)
        )
    }

    fn generate_export(
        &mut self,
        registration: &TypeRegistration,
        registry: &TypeRegistry,
        use_as: Vec<UseAs>,
        deps: &mut Vec<&'static str>,
    ) -> ExportConversionResult {
        let mut default_value = None;
        let tmp;
        let v = registration.data::<ReflectDefault>().map(|v| v.default());
        if v.is_some() {
            tmp = v.unwrap();
            default_value = Some(tmp.as_ref());
        }

        let out = match registration.type_info() {
            TypeInfo::TupleStruct(info) => {
                self.generate_tuple_struct_export(info, registry, default_value, use_as)
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
            TypeInfo::Enum(info) => self.generate_enum_export(info, registry, use_as),
            TypeInfo::Opaque(_) => Ok(vec![]),
            TypeInfo::Set(_) => Err(ExportConversionError::SetUnsupported),
        };

        if out.is_ok() {
            let mut new_deps = dependencies(registration, registry);
            if new_deps.iter().all(|n| {
                if let Some(t) = registry.get_with_type_path(n) {
                    return Self::is_supported(t);
                }
                false
            }) {
                deps.append(&mut new_deps);
                return out;
            } else {
                return Err(ExportConversionError::DependencyError);
            }
        }
        out
    }

    fn remove_with_dependency(&mut self, type_path: &str) {
        let mut to_remove = vec![type_path.to_string()];
        while let Some(type_path) = to_remove.pop() {
            self.types.retain(|_, export| {
                export.iter().all(|export| match &export.type_data {
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
            })
        }
    }

    fn generate_tuple_struct_export(
        &mut self,
        info: &TupleStructInfo,
        registry: &TypeRegistry,
        default_value: Option<&dyn Reflect>,
        _use_as: Vec<UseAs>,
    ) -> ExportConversionResult {
        let root = TypeExport {
            id: self.next_id(),
            name: info.type_path().to_string(),
            type_data: TypeData::Class(Class {
                use_as: USE_AS_PROPERTY.to_vec(),
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
                            value: unnamed_field_json_value(
                                default_value.map(|v| v.as_partial_reflect()),
                                s,
                            ),
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
            type_to_field(registry.get(info.item_ty().id()).unwrap())?;

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
                            value: unnamed_field_json_value(
                                default_value.map(|v| v.as_partial_reflect()),
                                s,
                            ),
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
                            value: named_field_json_value(
                                default_value.map(|v| v.as_partial_reflect()),
                                s,
                            ),
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
        registry: &TypeRegistry,
        _use_as: Vec<UseAs>,
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
            // Creates types for:
            // Enum for the enum variant
            // Class's for each non-unit variant
            // Class to hold the variant + each non-unit variant.

            // Note: extra `:` is done to not conflict with an enum variant named Variant
            let variants_name = info.type_path().to_string() + ":::Variant";

            let mut out = vec![TypeExport {
                id: self.next_id(),
                name: variants_name.clone(),
                type_data: TypeData::Enum(Enum {
                    storage_type: StorageType::String,
                    values_as_flags: false,
                    values: info.iter().map(|s| s.name().to_string()).collect(),
                }),
            }];

            let mut root_members = Vec::with_capacity(2);
            root_members.push(Member {
                // `:` is to separate from an enum variant named `variant`
                // and put it at the top of the fields (they are alphabetized in the editor)
                name: ":variant".to_string(),
                property_type: Some(variants_name),
                type_field: FieldType::Class,
                value: info
                    .iter()
                    .next()
                    .map(|s| serde_json::Value::String(s.name().to_string()))
                    .unwrap_or_default(),
            });

            for variant in info.iter() {
                match variant {
                    VariantInfo::Struct(s) => {
                        let name = format!("{}::{}", info.type_path(), s.name());
                        let import = TypeExport {
                            id: self.next_id(),
                            name: name.clone(),
                            type_data: TypeData::Class(Class {
                                use_as: USE_AS_PROPERTY.to_vec(),
                                color: DEFAULT_COLOR.to_string(),
                                draw_fill: true,
                                members: s
                                    .iter()
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
                        out.push(import);

                        let root_field = Member {
                            name: s.name().to_string(),
                            property_type: Some(name),
                            type_field: FieldType::Class,
                            value: Default::default(),
                        };

                        root_members.push(root_field);
                    }
                    VariantInfo::Tuple(tuple) => {
                        let name = format!("{}::{}", info.type_path(), tuple.name());
                        let import = TypeExport {
                            id: self.next_id(),
                            name: name.clone(),
                            type_data: TypeData::Class(Class {
                                use_as: USE_AS_PROPERTY.to_vec(),
                                color: DEFAULT_COLOR.to_string(),
                                draw_fill: true,
                                members: tuple
                                    .iter()
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
                        out.push(import);

                        let root_field = Member {
                            name: tuple.name().to_string(),
                            property_type: Some(name),
                            type_field: FieldType::Class,
                            value: Default::default(),
                        };

                        root_members.push(root_field);
                    }
                    VariantInfo::Unit(_) => continue,
                }
            }

            let root = TypeExport {
                id: self.next_id(),
                name: info.type_path().to_string(),
                type_data: TypeData::Class(Class {
                    use_as: USE_AS_PROPERTY.to_vec(),
                    color: DEFAULT_COLOR.to_string(),
                    draw_fill: true,
                    members: root_members,
                }),
            };

            out.push(root);

            Ok(out)
        }
    }
}

fn value_to_json(value: &dyn PartialReflect) -> serde_json::Value {
    let Some(type_info) = value.get_represented_type_info() else {
        return serde_json::Value::default();
    };

    match (type_info.type_path(), type_info, value.reflect_ref()) {
        ("bool", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<bool>().unwrap())
        }
        ("f32", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<f32>().unwrap())
        }
        ("f64", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<f64>().unwrap())
        }
        ("isize", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<isize>().unwrap())
        }
        ("i8", _, ReflectRef::Opaque(v)) => serde_json::json!(*v.try_downcast_ref::<i8>().unwrap()),
        ("i16", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<i16>().unwrap())
        }
        ("i32", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<i32>().unwrap())
        }
        ("i64", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<i64>().unwrap())
        }
        ("i128", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<i128>().unwrap())
        }
        ("usize", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<usize>().unwrap())
        }
        ("u8", _, ReflectRef::Opaque(v)) => serde_json::json!(*v.try_downcast_ref::<u8>().unwrap()),
        ("u16", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<u16>().unwrap())
        }
        ("u32", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<u32>().unwrap())
        }
        ("u64", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<u64>().unwrap())
        }
        ("u128", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<u128>().unwrap())
        }
        ("alloc::string::String", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<String>().unwrap())
        }
        ("alloc::borrow::Cow<str>", _, ReflectRef::Opaque(v)) => {
            serde_json::json!(*v.try_downcast_ref::<Cow<str>>().unwrap())
        }
        ("bevy_color::color::Color", _, _) => {
            let c = value.try_downcast_ref::<Color>().unwrap();
            serde_json::json!(format!("#{:08x}", c.to_linear().as_u32()))
        }
        (_, TypeInfo::Enum(info), ReflectRef::Enum(v)) => {
            if info.iter().all(|v| matches!(v, VariantInfo::Unit(_))) {
                serde_json::json!(v.variant_name())
            } else {
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
            // warn!(
            //     "cannot convert type '{}' to a JSON value",
            //     type_info.type_path()
            // );
            serde_json::Value::default()
        }
    }
}

fn named_field_json_value(
    value: Option<&dyn PartialReflect>,
    field: &NamedField,
) -> serde_json::Value {
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
    value: Option<&dyn PartialReflect>,
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
            if matches!(info, TypeInfo::Opaque(_)) {
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

fn dependencies(registration: &TypeRegistration, registry: &TypeRegistry) -> Vec<&'static str> {
    let deps = match registration.type_info() {
        TypeInfo::Struct(info) => info.iter().map(NamedField::type_path).collect(),
        TypeInfo::TupleStruct(info) => info.iter().map(UnnamedField::type_path).collect(),
        TypeInfo::Tuple(info) => info.iter().map(UnnamedField::type_path).collect(),
        TypeInfo::List(info) => vec![info.item_ty().type_path_table().path()],
        TypeInfo::Array(info) => vec![info.item_ty().type_path_table().path()],
        TypeInfo::Map(info) => vec![
            info.key_ty().type_path_table().path(),
            info.value_ty().type_path_table().path(),
        ],
        TypeInfo::Enum(info) => info
            .iter()
            .flat_map(|s| match s {
                VariantInfo::Struct(s) => s.iter().map(NamedField::type_path).collect(),
                VariantInfo::Tuple(s) => s.iter().map(UnnamedField::type_path).collect(),
                VariantInfo::Unit(_) => vec![],
            })
            .collect(),
        TypeInfo::Set(info) => vec![info.value_ty().type_path_table().path()],
        TypeInfo::Opaque(_) => vec![],
    };

    let mut all_deps = deps.clone();
    for d in deps {
        if let Some(t) = registry.get_with_type_path(d) {
            let mut new_deps = dependencies(t, registry);
            all_deps.append(&mut new_deps);
        }
    }
    all_deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::encase::rts_array::Length;

    #[test]
    fn generate_with_entity() {
        #[derive(Component, Reflect)]
        #[reflect(Component)]
        struct ComponentA(Entity);

        let mut registry = TypeRegistry::new();
        registry.register::<ComponentA>();

        let exports = TypeExportRegistry::from_registry(&registry);
        let export_type = &exports.types.get(ComponentA::type_path()).unwrap();
        assert_eq!(export_type.length(), 1);
        assert_eq!(export_type[0].name, ComponentA::type_path().to_string());
        assert_eq!(
            export_type[0].type_data,
            TypeData::Class(Class {
                use_as: USE_AS_PROPERTY.to_vec(),
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: vec![Member {
                    name: "0".to_string(),
                    property_type: None,
                    type_field: FieldType::Object,
                    value: Default::default(),
                }],
            }),
        );
    }

    #[test]
    fn generate_with_entity_option() {
        #[derive(Component, Reflect)]
        #[reflect(Component)]
        struct ComponentA(Option<Entity>);

        let mut registry = TypeRegistry::new();
        registry.register::<ComponentA>();

        let exports = TypeExportRegistry::from_registry(&registry);
        let export_type = &exports.types.get(ComponentA::type_path()).unwrap();
        assert_eq!(export_type.length(), 1);
        assert_eq!(export_type[0].name, ComponentA::type_path().to_string());
        assert_eq!(
            export_type[0].type_data,
            TypeData::Class(Class {
                use_as: USE_AS_PROPERTY.to_vec(),
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: vec![Member {
                    name: "0".to_string(),
                    property_type: None,
                    type_field: FieldType::Object,
                    value: Default::default(),
                }],
            }),
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

        let exports = TypeExportRegistry::from_registry(&registry);
        let export_type = &exports.types.get(EnumComponent::type_path()).unwrap();
        assert_eq!(export_type.length(), 1);
        assert_eq!(export_type[0].name, EnumComponent::type_path().to_string());
        assert_eq!(
            export_type[0].type_data,
            TypeData::Enum(Enum {
                storage_type: StorageType::String,
                values: vec!["VarA".to_string(), "VarB".to_string(), "VarC".to_string(),],
                values_as_flags: false,
            }),
        );
    }

    #[test]
    fn generate_nested_struct_with_default() {
        #[derive(Reflect, Default)]
        #[reflect(Default)]
        enum TestEnum {
            VarA,
            #[default]
            VarB,
            VarC,
        }

        #[derive(Reflect)]
        #[reflect(Default)]
        struct InnerStruct {
            another_float: f64,
            another_integer: u16,
            another_enum: TestEnum,
        }
        impl Default for InnerStruct {
            fn default() -> Self {
                Self {
                    another_float: 123.456,
                    another_integer: 42,
                    another_enum: TestEnum::VarC,
                }
            }
        }

        #[derive(Component, Reflect, Default)]
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

        let exports = TypeExportRegistry::from_registry(&registry);
        let export_type = &exports.types.get(StructComponent::type_path()).unwrap();
        assert_eq!(export_type.length(), 1);
        assert_eq!(
            export_type[0].name,
            StructComponent::type_path().to_string()
        );
        assert_eq!(
            export_type[0].type_data,
            TypeData::Class(Class {
                use_as: USE_AS_PROPERTY.to_vec(),
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: vec![
                    Member {
                        name: "a_float".to_string(),
                        property_type: None,
                        type_field: FieldType::Float,
                        value: serde_json::json!(0.0),
                    },
                    Member {
                        name: "an_enum".to_string(),
                        property_type: Some(TestEnum::type_path().to_string()),
                        type_field: FieldType::String,
                        value: serde_json::json!("VarB"),
                    },
                    Member {
                        name: "a_struct".to_string(),
                        property_type: Some(InnerStruct::type_path().to_string()),
                        type_field: FieldType::Class,
                        value: serde_json::json!({
                            "another_enum": "VarC",
                            "another_float": 123.456,
                            "another_integer": 42
                        })
                    },
                    Member {
                        name: "an_integer".to_string(),
                        property_type: None,
                        type_field: FieldType::Int,
                        value: serde_json::json!(0),
                    }
                ],
            })
        );
    }

    #[test]
    fn generate_nested_tuple_struct() {
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
        let export_type = &exports.types.get(TestOuter::type_path()).unwrap();
        assert_eq!(export_type.length(), 1);
        assert_eq!(export_type[0].name, TestOuter::type_path().to_string());
        assert_eq!(
            export_type[0].type_data,
            TypeData::Class(Class {
                use_as: USE_AS_PROPERTY.to_vec(),
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: vec![Member {
                    name: "0".to_string(),
                    property_type: Some(TestInner::type_path().to_string()),
                    type_field: FieldType::Class,
                    value: serde_json::json!({
                        "0": 0,
                        "1": serde_json::Value::default(),
                    })
                }]
            })
        );
    }

    #[test]
    fn generate_non_simple_enum() {
        #[derive(Reflect)]
        struct TestStruct {
            a: i32,
            b: String,
        }

        #[derive(Component, Reflect)]
        #[reflect(Component)]
        enum EnumComponent {
            VarA,
            VarB(u32),
            VarC(TestStruct),
            VarD((u16, f32)),
        }

        let mut registry = TypeRegistry::new();
        registry.register::<TestStruct>();
        registry.register::<EnumComponent>();

        let exports = TypeExportRegistry::from_registry(&registry);
        let export_type = &exports.types.get(EnumComponent::type_path()).unwrap();

        assert_eq!(export_type.length(), 5);
        assert_eq!(
            export_type[0].name,
            EnumComponent::type_path().to_string() + ":::Variant"
        );
        assert_eq!(
            export_type[0].type_data,
            TypeData::Enum(Enum {
                storage_type: StorageType::String,
                values: vec![
                    "VarA".to_string(),
                    "VarB".to_string(),
                    "VarC".to_string(),
                    "VarD".to_string(),
                ],
                values_as_flags: false,
            }),
        );
        assert_eq!(
            export_type[1].name,
            EnumComponent::type_path().to_string() + "::VarB"
        );
        assert_eq!(
            export_type[1].type_data,
            TypeData::Class(Class {
                use_as: USE_AS_PROPERTY.to_vec(),
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: vec![Member {
                    name: "0".to_string(),
                    property_type: None,
                    type_field: FieldType::Int,
                    value: serde_json::Value::default(),
                }],
            })
        );
        assert_eq!(
            export_type[2].name,
            EnumComponent::type_path().to_string() + "::VarC"
        );
        assert_eq!(
            export_type[2].type_data,
            TypeData::Class(Class {
                use_as: USE_AS_PROPERTY.to_vec(),
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: vec![Member {
                    name: "0".to_string(),
                    property_type: Some(TestStruct::type_path().to_string()),
                    type_field: FieldType::Class,
                    value: serde_json::Value::default(),
                }],
            })
        );
        assert_eq!(
            export_type[3].name,
            EnumComponent::type_path().to_string() + "::VarD"
        );
        assert_eq!(
            export_type[3].type_data,
            TypeData::Class(Class {
                use_as: USE_AS_PROPERTY.to_vec(),
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: vec![Member {
                    name: "0".to_string(),
                    property_type: Some("(u16, f32)".to_string()),
                    type_field: FieldType::Class,
                    value: serde_json::Value::default(),
                }],
            })
        );
        assert_eq!(export_type[4].name, EnumComponent::type_path().to_string());
        assert_eq!(
            export_type[4].type_data,
            TypeData::Class(Class {
                use_as: USE_AS_PROPERTY.to_vec(),
                color: DEFAULT_COLOR.to_string(),
                draw_fill: true,
                members: vec![
                    Member {
                        name: ":variant".to_string(),
                        property_type: Some(EnumComponent::type_path().to_string() + ":::Variant"),
                        type_field: FieldType::Class,
                        value: serde_json::json!("VarA"),
                    },
                    Member {
                        name: "VarB".to_string(),
                        property_type: Some(EnumComponent::type_path().to_string() + "::VarB"),
                        type_field: FieldType::Class,
                        value: serde_json::Value::default(),
                    },
                    Member {
                        name: "VarC".to_string(),
                        property_type: Some(EnumComponent::type_path().to_string() + "::VarC"),
                        type_field: FieldType::Class,
                        value: serde_json::Value::default(),
                    },
                    Member {
                        name: "VarD".to_string(),
                        property_type: Some(EnumComponent::type_path().to_string() + "::VarD"),
                        type_field: FieldType::Class,
                        value: serde_json::Value::default(),
                    },
                ],
            })
        );
    }
}
