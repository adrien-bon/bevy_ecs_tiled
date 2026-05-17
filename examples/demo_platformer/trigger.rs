use avian2d::prelude::*;
use bevy::prelude::*;

use crate::UpdateSystems;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<TriggerZone>();
    app.register_type::<TriggerActor>();
    app.add_systems(
        Update,
        handle_collision_start.in_set(UpdateSystems::TriggerZones),
    );
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, Default)]
#[require(CollisionEventsEnabled, Sensor)]
pub enum TriggerZone {
    #[default]
    Unknown,
    Kill,
    Damage(f32),
    Teleport(Entity),
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, Default)]
pub struct TriggerActor;

fn handle_collision_start(
    mut message_reader: MessageReader<CollisionStart>,
    mut commands: Commands,
    zone_query: Query<&TriggerZone>,
    mut actor_query: Query<(Entity, &mut Transform), With<TriggerActor>>,
    teleport_dest_query: Query<&Transform, Without<TriggerActor>>,
) {
    for evt in message_reader.read() {
        let Ok(zone) = zone_query.get(evt.collider2) else {
            return;
        };
        let Ok((actor, mut transform)) = actor_query.get_mut(evt.collider1) else {
            return;
        };
        match zone {
            TriggerZone::Teleport(dest_entity) => {
                if let Ok(dest) = teleport_dest_query.get(*dest_entity) {
                    *transform = *dest;
                }
            }
            TriggerZone::Kill => {
                info!("Killed player");
                commands.entity(actor).despawn();
            }
            _ => {}
        }
    }
}
