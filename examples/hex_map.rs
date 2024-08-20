//! This example cycle through all four kinds of hexagonal maps and display debug informations about Tiled objects.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        // Enable debug informations about Tiled objects
        .add_plugins(TiledMapDebugPlugin::default())
        .add_plugins(helper::HelperPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, switch_map)
        .run();
}

struct MapInfos {
    asset: Handle<TiledMap>,
    path: String,
    description: String,
}

impl MapInfos {
    fn new(asset_server: &Res<AssetServer>, path: &str, description: &str) -> Self {
        Self {
            asset: asset_server.load(path.to_owned()),
            path: path.to_owned(),
            description: description.to_owned(),
        }
    }
}

#[derive(Resource)]
struct AssetsManager {
    map_assets: Vec<MapInfos>,
    map_entity: Option<Entity>,
    text_entity: Entity,
    map_index: usize,
}

impl AssetsManager {
    const BASE_TEXT: &'static str = "<space> = Cycle through different hexagonal maps";

    fn new(commands: &mut Commands) -> Self {
        Self {
            map_assets: Vec::new(),
            map_entity: None,
            text_entity: commands
                .spawn(TextBundle::from(AssetsManager::BASE_TEXT))
                .id(),
            map_index: 0,
        }
    }

    fn cycle_map(&mut self, commands: &mut Commands) {
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
                        tiled_settings: TiledMapSettings {
                            map_positioning: MapPositioning::Centered,
                            //map_positioning: MapPositioning::LayerOffset,
                            ..Default::default()
                        },
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

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut mgr = AssetsManager::new(&mut commands);
    mgr.map_assets.push(MapInfos::new(
        &asset_server,
        "hex_map_flat_top_even.tmx",
        "A finite flat-top (stagger axis = X) hexagonal map with 'even' stagger index",
    ));
    mgr.map_assets.push(MapInfos::new(
        &asset_server,
        "hex_map_flat_top_odd.tmx",
        "A finite flat-top (stagger axis = X) hexagonal map with 'odd' stagger index",
    ));
    mgr.map_assets.push(MapInfos::new(
        &asset_server,
        "hex_map_pointy_top_even.tmx",
        "A finite pointy-top (stagger axis = Y) hexagonal map with 'even' stagger index",
    ));
    mgr.map_assets.push(MapInfos::new(
        &asset_server,
        "hex_map_pointy_top_odd.tmx",
        "A finite pointy-top (stagger axis = Y) hexagonal map with 'odd' stagger index",
    ));
    commands.insert_resource(mgr);
}

fn switch_map(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mgr: ResMut<AssetsManager>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        mgr.cycle_map(&mut commands);
    }
}
