use serde::{Serialize, Deserialize};

#[test]
fn deserialize() {
    let file = std::fs::File::open("propertytypes2.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let x: Vec<TypeImport> = serde_json::from_reader(reader).unwrap();
    let s = serde_json::to_string(&x).unwrap();
    assert_eq!(x, serde_json::from_str::<Vec<_>>(&s).unwrap());
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct TypeImport {
    pub id: u32,
    pub name: String,
    #[serde(flatten)]
    pub type_data: TypeData,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TypeData {
    Enum(Enum),
    Class(Class),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    pub use_as: Vec<UseAs>,
    pub color: String,
    pub draw_fill: bool,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_type: Option<String>,
    #[serde(rename = "type")]
    pub type_field: FieldType,
    pub value: serde_json::Value,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enum {
    pub storage_type: StorageType,
    pub values: Vec<String>,
    pub values_as_flags: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    String,
    Int,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Bool,
    Color,
    Float,
    File,
    Int,
    Object,
    String,
    Class,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UseAs {
    Property,
    Map,
    Layer,
    Object,
    Tile,
    Tileset,
    WangColor,
    WangSet,
    Project,
}

impl UseAs {
    pub fn all_supported() -> Vec<UseAs> {
        vec!(
            UseAs::Property,
            UseAs::Map,
            UseAs::Layer,
            UseAs::Object,
            UseAs::Tile,
        )
    }
}