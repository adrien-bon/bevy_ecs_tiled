use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::map::TilemapRenderSettings;

pub type MapInfosCallback = fn(&mut EntityCommands);

pub struct MapInfos {
    asset: Handle<TiledMap>,
    path: String,
    description: String,
    callback: MapInfosCallback,
}

impl MapInfos {
    pub fn new(
        asset_server: &Res<AssetServer>,
        path: &str,
        description: &str,
        callback: MapInfosCallback,
    ) -> Self {
        Self {
            asset: asset_server.load(path.to_owned()),
            path: path.to_owned(),
            description: description.to_owned(),
            callback,
        }
    }
}

#[derive(Resource)]
pub struct AssetsManager {
    map_assets: Vec<MapInfos>,
    map_entity: Option<Entity>,
    text_entity: Entity,
    map_index: usize,
}

impl AssetsManager {
    const BASE_TEXT: &'static str = "<space> = Cycle through different maps";

    pub fn new(commands: &mut Commands) -> Self {
        Self {
            map_assets: Vec::new(),
            map_entity: None,
            text_entity: commands
                .spawn(TextBundle::from(AssetsManager::BASE_TEXT))
                .id(),
            map_index: 0,
        }
    }

    pub fn add_map(&mut self, map_infos: MapInfos) {
        self.map_assets.push(map_infos);
    }

    pub fn cycle_map(&mut self, commands: &mut Commands) {
        info!(
            " => Switching to map '{}'",
            self.map_assets[self.map_index].path
        );

        // Update displayed text
        commands
            .entity(self.text_entity)
            .insert(TextBundle::from(format!(
                "{}\nmap name = {}\n{}",
                AssetsManager::BASE_TEXT,
                self.map_assets[self.map_index].path,
                self.map_assets[self.map_index].description
            )));

        // Handle map update: spawn the map if it does not exist yet
        // or just update the map handle if already spawned
        if let Some(entity) = self.map_entity {
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert(TiledMapHandle(self.map_assets[self.map_index].asset.to_owned()));
            entity_commands.remove::<TiledMapSettings>();
            entity_commands.remove::<TilemapRenderSettings>();
            (self.map_assets[self.map_index].callback)(&mut entity_commands);
        } else {
            let mut entity_commands = commands.spawn(TiledMapHandle(self.map_assets[self.map_index].asset.to_owned()));
            (self.map_assets[self.map_index].callback)(&mut entity_commands);
            self.map_entity = Some(entity_commands.id());
        }

        // Update the map index
        self.map_index += 1;
        if self.map_index >= self.map_assets.len() {
            self.map_index = 0;
        }
    }
}
