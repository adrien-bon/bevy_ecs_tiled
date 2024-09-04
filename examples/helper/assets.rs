use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct MapInfos {
    asset: Handle<TiledMap>,
    path: String,
    description: String,
    render_settings: TilemapRenderSettings,
    tiled_settings: TiledMapSettings,
}

impl MapInfos {
    pub fn new(
        asset_server: &Res<AssetServer>,
        render_settings: TilemapRenderSettings,
        tiled_settings: TiledMapSettings,
        path: &str,
        description: &str,
    ) -> Self {
        Self {
            asset: asset_server.load(path.to_owned()),
            path: path.to_owned(),
            description: description.to_owned(),
            render_settings,
            tiled_settings,
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
            commands
                .entity(entity)
                .insert(self.map_assets[self.map_index].asset.to_owned());
        } else {
            self.map_entity = Some(
                commands
                    .spawn(TiledMapBundle {
                        tiled_map: self.map_assets[self.map_index].asset.to_owned(),
                        render_settings: self.map_assets[self.map_index].render_settings,
                        tiled_settings: self.map_assets[self.map_index].tiled_settings.clone(),
                        ..Default::default()
                    })
                    .id(),
            );
        }

        // Update the map index
        self.map_index += 1;
        if self.map_index >= self.map_assets.len() {
            self.map_index = 0;
        }
    }
}
