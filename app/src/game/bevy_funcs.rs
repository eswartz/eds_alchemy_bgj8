use bevy::prelude::*;
use eds_bevy_common::{
    layers::GameLayer,
    physics::{
        Collider, ColliderConstructor, ColliderConstructorHierarchy,
        ColliderConstructorHierarchyReady, CollisionEnd, CollisionEventsEnabled, CollisionLayers,
        CollisionMargin, CollisionStart, Restitution, RigidBody, Sensor,
    },
};
use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::*;

/// Register game-specific functions.
///
/// :see [fedry_bevy_plugin::bevy_funcs] for the common ones.
///
pub(crate) fn register_script_funcs(mut commands: Commands, runtime: If<Res<ScriptRuntime>>) {
    let rt = &runtime.rt;
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
        rt.to_atom("make_children_physical_after_collision"),
        commands.register_system(make_children_physical_after_collision),
    );
}

// pub(crate) fn check_collisions(mut reader: MessageReader<CollisionStart>) {
//     for ev in reader.read() {
//         dbg!(ev);
//     }
// }
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
            dbg!(entity, parent_ent);
            commands
                .entity(parent_ent)
                .insert((
                    CollisionEventsEnabled,
                    Sensor,
                    ColliderConstructor::ConvexHullFromMesh,
                    CollisionLayers::new(
                        [GameLayer::Gameplay, GameLayer::World],
                        [GameLayer::Player, GameLayer::Gameplay],
                    ),
                ))
                .observe(
                    move |event: On<CollisionEnd>,
                          mut commands: Commands,
                          child_q: Query<&Children>,
                          rigid_q: Query<&RigidBody>| {
                        dbg!(&event);
                        // iterate down from the split root
                        for kid in child_q.iter_descendants(target) {
                            if !rigid_q.contains(kid) {
                                continue;
                            }
                            commands.entity(kid).insert(RigidBody::Dynamic);
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
            // ColliderConstructor::Sphere { radius: 0.005 },
        ),))
        .observe(on_colliders_ready_decorate_children);

    Ok(ObjectPtr::for_true().into())
}

fn on_colliders_ready_decorate_children(
    event: On<ColliderConstructorHierarchyReady>,
    mut commands: Commands,
    child_q: Query<&Children>,
    collider_q: Query<&Collider>,
) {
    let entity = dbg!(event.entity);
    let mut count = 0;
    for kid in child_q.iter_descendants(entity) {
        if collider_q.contains(kid) {
            count += 1;
            commands.entity(kid).insert((
                RigidBody::Static,
                Restitution::new(0.25),
                CollisionLayers::new(
                    [GameLayer::World],
                    [GameLayer::Player, GameLayer::World, GameLayer::Default],
                ),
            ));
        }
    }
    dbg!(count);
}

// //////////////

// pub(crate) fn make_children_physical(
//     In((entity, _rt, _args)): In<(Entity, RuntimeArc, Vec<ObjectPtr>)>,
//     mut commands: Commands,
// ) -> ExecResult {
//     // Let Avian add colliders.
//     commands.entity(entity).insert((
//         ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
//         CollisionLayers::new(
//             [GameLayer::Gameplay],
//             [GameLayer::Player, GameLayer::Gameplay],
//         ),
//     ));
//     Ok(ObjectPtr::for_nil().into())
// }

// pub(crate) fn make_dynamic_after_collision(
//     In((entity, _rt, _args)): In<(Entity, RuntimeArc, Vec<ObjectPtr>)>,
//     mut commands: Commands,
//     child_q: Query<&Children>,
//     mesh_q: Query<&Mesh3d>,
// ) -> ExecResult {
//     // Avian may or may not have added colliders yet, so do this for each mesh inside.
//     for kid in child_q.iter_descendants(entity) {
//         if !mesh_q.contains(kid) {
//             continue;
//         }
//         dbg!(kid);
//         commands.entity(kid).insert((
//             RigidBody::Static,
//             Restitution::new(0.25),
//             CollisionLayers::new(
//                 [GameLayer::Gameplay],
//                 [GameLayer::Player, GameLayer::Gameplay, GameLayer::World],
//             ),
//         ));
//     }

//     // Top entity has the handler.
//     commands
//         .entity(entity)
//         .insert((CollisionEventsEnabled,))
//         .observe(
//             move |event: On<CollisionEnd>,
//                   mut commands: Commands,
//                   child_q: Query<&Children>,
//                   rigid_q: Query<&RigidBody>| {
//                 dbg!(&event);
//                 for kid in child_q.iter_descendants(entity) {
//                     if !rigid_q.contains(kid) {
//                         continue;
//                     }
//                     commands.entity(kid).insert(RigidBody::Dynamic);
//                 }
//             },
//         );

//     Ok(ObjectPtr::for_nil().into())
// }

/// Policy for the object(s) generated by the `spawn_cube` call.
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect)]
#[type_path = "game"]
pub(crate) enum Lifetime {
    /// Create as a temporary object, which will be despawned
    /// after a given amount of game time.
    Temporary { secs: f32 },
    /// Try to keep the object (default).
    /// This is typically bounded by the current level's end.
    #[default]
    Forever,
}
