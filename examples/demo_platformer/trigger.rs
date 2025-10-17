use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<TriggerZone>();
    app.register_type::<TriggerActor>();
    app.add_systems(
        Update,
        (create_trigger_zone, handle_collision_start).chain(),
    );
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, Default)]
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

fn create_trigger_zone(
    mut commands: Commands,
    mut message_reader: MessageReader<TiledEvent<ColliderCreated>>,
    zone_query: Query<Entity, With<TriggerZone>>,
) {
    for evt in message_reader.read() {
        if zone_query.get(*evt.event.collider_of).is_ok() {
            commands
                .entity(evt.origin)
                .insert((Sensor, CollisionEventsEnabled));
        }
    }
}

fn handle_collision_start(
    mut message_reader: MessageReader<CollisionStart>,
    mut commands: Commands,
    zone_query: Query<&TriggerZone>,
    collider_query: Query<&TiledColliderOf>,
    mut actor_query: Query<(Entity, &mut Transform), With<TriggerActor>>,
    teleport_dest_query: Query<&Transform, Without<TriggerActor>>,
) {
    for evt in message_reader.read() {
        let Ok(zone) = collider_query
            .get(evt.collider1)
            .and_then(|&collider_of| zone_query.get(*collider_of))
        else {
            return;
        };
        let Some(actor_entity) = evt.body2 else {
            return;
        };
        let Ok((actor, mut transform)) = actor_query.get_mut(actor_entity) else {
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
