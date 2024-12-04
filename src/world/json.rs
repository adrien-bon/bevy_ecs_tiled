
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct TiledWorld {
    #[serde(flatten)]
    pub world_data: WorldData,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum WorldData {
    Map(TiledWorldMap),
    Pattern(TiledWorldPattern),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TiledWorldMap {
    filename: String,
    x: u64,
    y: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TiledWorldPattern {
    regexp: String,
    multiplier_x: u64,
    multiplier_y: u64,
    offset_x: u64,
    offset_y: u64,
}