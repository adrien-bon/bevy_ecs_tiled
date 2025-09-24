use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<TriggerZone>();
    app.register_type::<TriggerActor>();
    app.add_systems(Update, create_trigger_zone);
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
    mut evt_reader: EventReader<TiledEvent<ColliderCreated>>,
    collider_query: Query<&TiledColliderOf>,
    zone_query: Query<Entity, With<TriggerZone>>,
) {
    for evt in evt_reader.read() {
        if let Ok(collider_of) = collider_query.get(evt.origin) {
            if zone_query.get(collider_of.0).is_ok() {
                commands
                    .entity(evt.origin)
                    .insert((Sensor, RigidBody::Static, CollisionEventsEnabled))
                    .observe(handle_collision_start);
            }
        }
    }
}

fn handle_collision_start(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    zone_query: Query<&TriggerZone>,
    collider_query: Query<&TiledColliderOf>,
    mut actor_query: Query<(Entity, &mut Transform), With<TriggerActor>>,
    teleport_dest_query: Query<&Transform, Without<TriggerActor>>,
) {
    let Ok(zone) = collider_query
        .get(trigger.target())
        .and_then(|collider_of| zone_query.get(collider_of.0))
    else {
        return;
    };
    let Ok((actor, mut transform)) = actor_query.get_mut(trigger.event().collider) else {
        return;
    };
    match zone {
        TriggerZone::Teleport(teleport_dest) => {
            if let Ok(dest) = teleport_dest_query.get(*teleport_dest) {
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
