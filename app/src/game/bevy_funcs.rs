use bevy::prelude::*;
use eds_bevy_common::{
    base_dir::find_runtime_base_directory_by_folder,
    effects::ShrinkAndDisappear,
    layers::GameLayer,
    markers::{DespawnAfter, Spawned},
    physics::{
        Collider, ColliderConstructor, ColliderConstructorHierarchy,
        ColliderConstructorHierarchyReady, CollisionEventsEnabled, CollisionLayers, CollisionStart,
        Friction, LinearDamping, LinearVelocity, Mass, Restitution, RigidBody, Vector,
    },
    surface_material::SurfaceMaterial,
};
use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::*;
use rand::RngExt as _;

use crate::game::{GameScript, env::remove_water};

/// Register game-specific functions.
///
/// :see [fedry_bevy_plugin::bevy_funcs] for the common ones.
///
pub(crate) fn register_script_funcs(mut commands: Commands, runtime: If<Res<ScriptRuntime>>) {
    let rt = &runtime.rt;

    let source_service = rt.get_service::<SourceService>().unwrap();
    let base_dir = find_runtime_base_directory_by_folder("assets").unwrap(); // found outside
    source_service.prepend_search_directory(base_dir.join("assets/scripts").to_path_buf());

    let bevy_system_service = rt
        .get_service::<BevySystemService>()
        .expect("expected BevySystemService");
    // bevy_system_service.add_runtime_system(
    //     rt.to_atom("make_children_physical"),
    //     commands.register_system(make_children_physical),
    // );
    // bevy_system_service.add_runtime_system(
    //     rt.to_atom("make_dynamic_after_collision"),
    //     commands.register_system(make_dynamic_after_collision),
    // );
    bevy_system_service.add_runtime_system(
        rt.to_atom("remove_spawned"),
        commands.register_system(remove_spawned),
    );
    bevy_system_service.add_runtime_system(
        rt.to_atom("make_children_physical_after_collision"),
        commands.register_system(make_children_physical_after_collision),
    );

    bevy_system_service.add_runtime_system(
        rt.to_atom("spawn_water"),
        commands.register_system(spawn_water),
    );
}

/// Remove items with the [Spawned] marker.
pub(crate) fn remove_spawned(
    In((_entity, rt, _args)): In<(Entity, RuntimeArc, Vec<ObjectPtr>)>,
    mut commands: Commands,
    spawned_q: Query<Entity, With<Spawned>>,
) -> ExecResult {
    for ent in spawned_q.iter() {
        commands.entity(ent).try_despawn();
    }
    commands.run_system_cached(remove_water);
    Ok(rt
        .for_typed_int(&rt.types().ufast_type, spawned_q.count() as _)?
        .into())
}

/// For the parent `entity`, which contains a split mesh inside,
/// ensure that
/// (1) the parent has a Collider and is RigidBody::Static
/// (2) the parent reports an event when the player or other Gameplay entity
/// stops colliding
/// (3) all the children have Colliders and are RigidBody::Static
/// (4) when that collision-end event occurs, the children become RigidBody::Dynamic.
///
/// args: target: Uns64
///
/// returns: true if complete (i.e. found a mesh on `entity` to target), else `false`
pub(crate) fn make_children_physical_after_collision(
    In((entity, rt, args)): In<(Entity, RuntimeArc, Vec<ObjectPtr>)>,
    // ctx_p: ConvertContextParam,
    mut commands: Commands,
    child_q: Query<&Children>,
    parent_q: Query<&ChildOf>,
    mesh_q: Query<&Mesh3d>,
) -> ExecResult {
    if args.len() < 1 {
        return Err(RuntimeError::MissingArgument(0));
    }

    let target = decode_entity(&rt, &args[0])?;

    // Make sure the parent senses events, by adding sensors and colliders to its own meshes.
    let mut any = false;
    if entity != target {
        'outer: for parent_ent in child_q.iter_descendants(entity) {
            if !mesh_q.contains(parent_ent) {
                continue;
            }

            // Don't go inside nested target.
            let mut step = parent_ent;
            while let Ok(parent) = parent_q.get(step) {
                if parent.0 == target {
                    continue 'outer;
                }
                step = parent.0;
            }

            any = true;
            // Make sure we detect the player or other entity hitting us.
            // dbg!(entity, parent_ent);
            commands
                .entity(parent_ent)
                .insert((
                    CollisionEventsEnabled,
                    // Sensor,
                    ColliderConstructor::ConvexHullFromMesh,
                    CollisionLayers::new(
                        [GameLayer::Gameplay, GameLayer::World],
                        [GameLayer::Player, GameLayer::Gameplay],
                    ),
                ))
                .observe(
                    move |event: On<CollisionStart>,
                          mut commands: Commands,
                          child_q: Query<&Children>,
                          parent_q: Query<&ChildOf>,
                          vel_q: Query<&LinearVelocity>,
                          rigid_q: Query<&RigidBody>| {
                        let my_vel = if let Ok(vel) = vel_q.get(event.collider1) {
                            vel.0
                        } else if let Ok(vel) = vel_q.get(event.body1.unwrap_or(event.collider1)) {
                            vel.0
                        } else {
                            Vector::ZERO
                        };
                        let their_vel = if let Ok(vel) = vel_q.get(event.collider2) {
                            vel.0
                        } else if let Ok(vel) = vel_q.get(event.body2.unwrap_or(event.collider2)) {
                            vel.0
                        } else {
                            Vector::ZERO
                        };

                        if my_vel.dot(their_vel) > -100.0 {
                            // not head-on or powerful enough
                            return;
                        }

                        commands.entity(event.event_target()).insert((
                            // LinearVelocity::ZERO,
                            ShrinkAndDisappear::default().with_time_secs(0.25),
                        ));
                        commands
                            .entity(event.observer())
                            .insert((ShrinkAndDisappear::default().with_time_secs(0.25),));
                        commands
                            .entity(event.body1.unwrap_or(event.collider1))
                            .try_remove::<Script<UserScript>>()
                            .try_remove::<Script<GameScript>>();
                        commands
                            .entity(event.body2.unwrap_or(event.collider2))
                            .try_remove::<Script<UserScript>>()
                            .try_remove::<Script<GameScript>>();

                        // Transfer parent velocity
                        let vel = if let Ok(vel) = vel_q.get(event.collider1) {
                            vel.0
                        } else if let Ok(vel) = vel_q.get(event.body1.unwrap_or(event.collider1)) {
                            vel.0
                        } else {
                            Vector::ZERO
                        };

                        // Iterate the original `target`, not the collision.
                        let mut rng = rand::rng();
                        for kid in child_q.iter_descendants(target) {
                            if !rigid_q.contains(kid) {
                                continue;
                            }

                            // Remove parent rigid body so the pieces move independently
                            let mut step = kid;
                            while let Ok(parent) = parent_q.get(step) {
                                if rigid_q.contains(parent.0) {
                                    commands.entity(parent.0).try_remove::<RigidBody>();
                                    break;
                                }
                                step = parent.0;
                            }

                            commands.entity(kid).insert((
                                RigidBody::Dynamic,
                                LinearVelocity(vel),
                                DespawnAfter(Duration::from_secs(rng.random_range(25..35))),
                            ));
                        }
                    },
                );
        }

        if !any {
            return Ok(ObjectPtr::for_false().into());
        }
    }

    // Let Avian add colliders to children. This will take a frame or two.
    // After that, there will Colliders on each split child.
    commands
        .entity(target)
        .insert((ColliderConstructorHierarchy::new(
            ColliderConstructor::ConvexHullFromMesh,
        ),))
        .observe(on_colliders_ready_decorate_children);

    Ok(ObjectPtr::for_true().into())
}

fn on_colliders_ready_decorate_children(
    event: On<ColliderConstructorHierarchyReady>,
    mut commands: Commands,
    child_q: Query<&Children>,
    collider_q: Query<&Collider>,
    surface_q: Query<&SurfaceMaterial>,
    mass_q: Query<&Mass>,
) {
    let entity = dbg!(event.entity);

    let surface = surface_q.get(entity).unwrap_or(&SurfaceMaterial::Unknown);
    let mass = mass_q.get(entity).map_or(1.0, |v| v.0);
    let mut count = 0;
    for kid in child_q.iter_descendants(entity) {
        if collider_q.contains(kid) {
            count += 1;
            commands.entity(kid).insert((
                RigidBody::Static,
                Spawned,
                Restitution::new(0.25),
                LinearDamping(0.999),
                Friction::new(0.9),
                CollisionLayers::new(
                    [GameLayer::World],
                    [GameLayer::Player, GameLayer::World, GameLayer::Default],
                ),
                Mass(mass),
                surface.clone(),
            ));
        }
    }
    bevy::log::info!("{count} children");
}

pub(crate) fn spawn_water(
    In((_entity, _rt, args)): In<(Entity, RuntimeArc, Vec<ObjectPtr>)>,
    mut commands: Commands,
    ctx_p: ConvertContextParam,
) -> ExecResult {
    use bevy_water::*;
    let height = if let Some(arg) = args.get(0)
        && let Some(h) = RtNumber::try_from(&arg)
    {
        h.as_real() as f32
    } else {
        0.0
    };
    let ctx = ctx_p.as_ctx();
    let span = if let Some(arg) = args.get(1)
        && let Ok(vec) = convert_obj_to_value::<UVec2>(&ctx, arg)
    {
        vec
    } else {
        UVec2::new(4, 4)
    };

    commands.insert_resource(WaterSettings {
        height: height,
        amplitude: 0.25f32.min(height * 0.5),
        spawn_tiles: Some(span),
        ..default()
    });
    commands.run_system_cached(bevy_water::setup_water);
    Ok(default())
}
