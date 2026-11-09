use bevy::prelude::*;
use bevy_yarnspinner::prelude::{DialogueRunner, YarnProject, YarnValue};
use eds_bevy_common::physics::*;
use eds_bevy_common::prelude::*;
use fedry_bevy_plugin::{Scripting, script::ExecutionMode};
use fedry_runtime::{
    common::RuntimeResult, object_ptr::ObjectPtr, runtime::RuntimeArc, types::NativeReal,
};

use crate::{
    assets::ScriptAssets,
    game::{DialogueCue, GameScript},
};

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "game"]
pub(crate) struct ActiveDialogueRunner;

pub(crate) fn spawn_dialogue_runner(
    mut commands: Commands,
    project: Option<Res<YarnProject>>,
) -> Result {
    let Some(project) = project else {
        commands.set_state(OverlayState::ErrorScreen);
        return Ok(());
    };
    let mut dialogue_runner = project.create_dialogue_runner(&mut commands);
    dialogue_runner.commands_mut().add_command(
        "dialogue_over",
        commands.register_system(command_dialogue_over),
    );

    dialogue_runner.commands_mut().add_command(
        "dialogue_stop",
        commands.register_system(command_dialogue_stop),
    );

    dialogue_runner.commands_mut().add_command(
        "attach_script_1",
        commands.register_system(command_attach_script_1),
    );
    dialogue_runner.commands_mut().add_command(
        "attach_script_2",
        commands.register_system(command_attach_script_2),
    );
    dialogue_runner.commands_mut().add_command(
        "attach_script_3",
        commands.register_system(command_attach_script_3),
    );
    dialogue_runner.commands_mut().add_command(
        "attach_script_4",
        commands.register_system(command_attach_script_4),
    );
    dialogue_runner.commands_mut().add_command(
        "attach_script_5",
        commands.register_system(command_attach_script_5),
    );

    commands.spawn((
        DespawnOnExit(LevelState::Playing),
        Name::new("DIALOGUE"),
        dialogue_runner,
        ActiveDialogueRunner,
    ));

    Ok(())
}

pub(crate) fn decorate_dialog_cues(
    mut commands: Commands,
    cue_q: Query<Entity, With<DialogueCue>>,
    reqs_q: Query<(&RigidBody, &CollisionEventsEnabled)>,
) {
    for ent in cue_q.iter() {
        if reqs_q.get(ent).is_err() {
            error!("DialogCue on entity {ent} has no RigidBody, CollisionEventsEnabled");
            continue;
        }
        commands.entity(ent).observe(on_enter_dialog_cue);
    }
}

pub(crate) fn on_enter_dialog_cue(
    event: On<CollisionStart>,
    player_q: Query<&OurPlayer>,
    cue_q: Query<&DialogueCue>,
    // look_q: Query<&PlayerLook>,
    // gxfrm_q: Query<&GlobalTransform>,
    mut runner: Query<&mut DialogueRunner, With<ActiveDialogueRunner>>,
) {
    if let Some(mut runner) = runner.iter_mut().next()
        && player_q.get(event.collider2).is_ok()
        && let Ok(cue) = cue_q.get(event.collider1)
    {
        // let Ok(gxfrm1) = gxfrm_q.get(event.body1.unwrap_or(event.collider1)) else {
        //     return;
        // };
        // let Ok(gxfrm2) = gxfrm_q.get(event.body2.unwrap_or(event.collider2)) else {
        //     return;
        // };
        // let offs = gxfrm1.translation() - gxfrm2.translation();
        // let y_ang = gxfrm1.rotation().to_euler(EulerRot::YXZ).0;
        // let dot = gxfrm1.translation().dot(gxfrm2.translation());
        // dbg!(offs, y_ang, dot);

        if !runner.is_running() {
            runner.start_node(&cue.0);
        }
    }
}

/// Stop dialog for now.
fn command_dialogue_stop(
    In(()): In<()>,
    // mut commands: Commands,
    mut runner_q: Query<&mut DialogueRunner>,
) {
    let Ok(mut runner) = runner_q.single_mut() else {
        warn!("no single ActiveDialogueRunner");
        return;
    };
    runner.stop();
}

/// Dialogue is over; remove the runner.
fn command_dialogue_over(
    In(()): In<()>,
    mut commands: Commands,
    runner_q: Query<Entity, With<ActiveDialogueRunner>>,
) {
    let Some(ent) = runner_q.iter().next() else {
        warn!("no single ActiveDialogueRunner");
        return;
    };

    commands.entity(ent).remove::<ActiveDialogueRunner>();
}

fn fetch_object_from_yarn(rt: &RuntimeArc, arg: YarnValue) -> RuntimeResult<ObjectPtr> {
    Ok(match arg {
        YarnValue::String(s) => {
            if let Ok(v) = s.parse::<f64>() {
                rt.for_typed_real(&rt.types().rfast_type, v as NativeReal)
                    .unwrap()
            } else if s == "false" {
                ObjectPtr::for_false()
            } else if s == "true" {
                ObjectPtr::for_true()
            } else {
                rt.for_string(s).unwrap()
            }
        }
        YarnValue::Number(n) => rt
            .for_typed_real(&rt.types().rfast_type, n as NativeReal)
            .unwrap(),
        YarnValue::Boolean(b) => ObjectPtr::for_bool(b),
    })
}

/// Attach a [Script] with the given name (entry in [ScriptModule]/entries.dry) and a single argument.
///
/// This populates the data.task and data.param fields for the dialogue runner to detect.
fn command_attach_script_1(
    In((name, arg)): In<(String, YarnValue)>,
    mut commands: Commands,
    runner_q: Query<Entity, With<ActiveDialogueRunner>>,
    scripting: Scripting<GameScript>,
    script_assets: Res<ScriptAssets>,
) {
    let script = scripting
        .new_script_from_module_id(script_assets.driver.id(), ExecutionMode::Async)
        .unwrap();

    let Ok(ent) = runner_q.single() else {
        warn!("no single ActiveDialogueRunner for '{name}'");
        return;
    };
    let rt = &scripting.runtime.rt;

    script
        .data()
        .map_mut()
        .insert(rt.to_atom("task"), rt.for_string(name).unwrap());

    let param_obj = fetch_object_from_yarn(&rt, arg).unwrap();

    script
        .data()
        .map_mut()
        .insert(rt.to_atom("param"), param_obj);

    commands.entity(ent).insert(script);
}

fn command_attach_script_2(
    In((name, arg1, arg2)): In<(String, YarnValue, YarnValue)>,
    mut commands: Commands,
    runner_q: Query<Entity, With<ActiveDialogueRunner>>,
    scripting: Scripting<GameScript>,
    script_assets: Res<ScriptAssets>,
) {
    let script = scripting
        .new_script_from_module_id(script_assets.driver.id(), ExecutionMode::Async)
        .unwrap();

    let Ok(ent) = runner_q.single() else {
        warn!("no single ActiveDialogueRunner for '{name}'");
        return;
    };
    let rt = &scripting.runtime.rt;

    script
        .data()
        .map_mut()
        .insert(rt.to_atom("task"), rt.for_string(name).unwrap());

    let param1_obj = fetch_object_from_yarn(&rt, arg1).unwrap();
    let param2_obj = fetch_object_from_yarn(&rt, arg2).unwrap();

    let tup_obj = rt.for_tuple(vec![param1_obj, param2_obj]).unwrap();

    script.data().map_mut().insert(rt.to_atom("param"), tup_obj);

    commands.entity(ent).insert(script);
}

fn command_attach_script_3(
    In((name, arg1, arg2, arg3)): In<(String, YarnValue, YarnValue, YarnValue)>,
    mut commands: Commands,
    runner_q: Query<Entity, With<ActiveDialogueRunner>>,
    scripting: Scripting<GameScript>,
    script_assets: Res<ScriptAssets>,
) {
    let script = scripting
        .new_script_from_module_id(script_assets.driver.id(), ExecutionMode::Async)
        .unwrap();

    let Ok(ent) = runner_q.single() else {
        warn!("no single ActiveDialogueRunner for '{name}'");
        return;
    };
    let rt = &scripting.runtime.rt;

    script
        .data()
        .map_mut()
        .insert(rt.to_atom("task"), rt.for_string(name).unwrap());

    let param1_obj = fetch_object_from_yarn(&rt, arg1).unwrap();
    let param2_obj = fetch_object_from_yarn(&rt, arg2).unwrap();
    let param3_obj = fetch_object_from_yarn(&rt, arg3).unwrap();

    let tup_obj = rt
        .for_tuple(vec![param1_obj, param2_obj, param3_obj])
        .unwrap();

    script.data().map_mut().insert(rt.to_atom("param"), tup_obj);

    commands.entity(ent).insert(script);
}

fn command_attach_script_4(
    In((name, arg1, arg2, arg3, arg4)): In<(String, YarnValue, YarnValue, YarnValue, YarnValue)>,
    mut commands: Commands,
    runner_q: Query<Entity, With<ActiveDialogueRunner>>,
    scripting: Scripting<GameScript>,
    script_assets: Res<ScriptAssets>,
) {
    let script = scripting
        .new_script_from_module_id(script_assets.driver.id(), ExecutionMode::Async)
        .unwrap();

    let Ok(ent) = runner_q.single() else {
        warn!("no single ActiveDialogueRunner for '{name}'");
        return;
    };
    let rt = &scripting.runtime.rt;

    script
        .data()
        .map_mut()
        .insert(rt.to_atom("task"), rt.for_string(name).unwrap());

    let param1_obj = fetch_object_from_yarn(&rt, arg1).unwrap();
    let param2_obj = fetch_object_from_yarn(&rt, arg2).unwrap();
    let param3_obj = fetch_object_from_yarn(&rt, arg3).unwrap();
    let param4_obj = fetch_object_from_yarn(&rt, arg4).unwrap();

    let tup_obj = rt
        .for_tuple(vec![param1_obj, param2_obj, param3_obj, param4_obj])
        .unwrap();

    script.data().map_mut().insert(rt.to_atom("param"), tup_obj);

    commands.entity(ent).insert(script);
}

fn command_attach_script_5(
    In((name, arg1, arg2, arg3, arg4, arg5)): In<(
        String,
        YarnValue,
        YarnValue,
        YarnValue,
        YarnValue,
        YarnValue,
    )>,
    mut commands: Commands,
    runner_q: Query<Entity, With<ActiveDialogueRunner>>,
    scripting: Scripting<GameScript>,
    script_assets: Res<ScriptAssets>,
) {
    let script = scripting
        .new_script_from_module_id(script_assets.driver.id(), ExecutionMode::Async)
        .unwrap();

    let Ok(ent) = runner_q.single() else {
        warn!("no single ActiveDialogueRunner for '{name}'");
        return;
    };
    let rt = &scripting.runtime.rt;

    script
        .data()
        .map_mut()
        .insert(rt.to_atom("task"), rt.for_string(name).unwrap());

    let param1_obj = fetch_object_from_yarn(&rt, arg1).unwrap();
    let param2_obj = fetch_object_from_yarn(&rt, arg2).unwrap();
    let param3_obj = fetch_object_from_yarn(&rt, arg3).unwrap();
    let param4_obj = fetch_object_from_yarn(&rt, arg4).unwrap();
    let param5_obj = fetch_object_from_yarn(&rt, arg5).unwrap();

    let tup_obj = rt
        .for_tuple(vec![
            param1_obj, param2_obj, param3_obj, param4_obj, param5_obj,
        ])
        .unwrap();

    script.data().map_mut().insert(rt.to_atom("param"), tup_obj);

    commands.entity(ent).insert(script);
}
