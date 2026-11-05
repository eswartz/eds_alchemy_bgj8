use bevy::prelude::*;
use bevy_yarnspinner::prelude::{DialogueRunner, YarnProject, YarnValue};
use eds_bevy_common::{
    physics::{CollisionEventsEnabled, CollisionStart, RigidBody},
    player_client::OurPlayer,
    states_sets::LevelState,
};
use fedry_bevy_plugin::{Scripting, script::ExecutionMode};
use fedry_runtime::{object_ptr::ObjectPtr, types::NativeReal};

use crate::{
    assets::ScriptAssets,
    game::{DialogueCue, GameScript},
};

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "game"]
pub(crate) struct ActiveDialogueRunner;

pub(crate) fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) -> Result {
    let mut dialogue_runner = project.create_dialogue_runner(&mut commands);
    dialogue_runner.commands_mut().add_command(
        "attach_script",
        commands.register_system(command_attach_script),
    );
    dialogue_runner.commands_mut().add_command(
        "dialogue_over",
        commands.register_system(command_dialogue_over),
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
    mut runner: Single<&mut DialogueRunner, With<ActiveDialogueRunner>>,
) {
    if player_q.get(event.collider2).is_ok()
        && let Ok(cue) = cue_q.get(event.collider1)
    {
        if !runner.is_running() {
            runner.start_node(&cue.0);
        }
    }
}

/// Dialogue is over; remove the runner.
fn command_dialogue_over(
    In(()): In<()>,
    mut commands: Commands,
    runner_q: Query<Entity, With<ActiveDialogueRunner>>,
) {
    let Ok(ent) = runner_q.single() else {
        warn!("no single ActiveDialogueRunner");
        return;
    };

    commands.entity(ent).remove::<ActiveDialogueRunner>();
}

/// Attach a [Script] with the given name (entry in [ScriptModule]) and a single argument.
fn command_attach_script(
    In((name, arg)): In<(String, YarnValue)>,
    mut commands: Commands,
    runner_q: Query<Entity, With<ActiveDialogueRunner>>,
    scripting: Scripting<GameScript>,
    script_assets: Res<ScriptAssets>,
) {
    dbg!(&name, &arg);

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

    let param_obj = match arg {
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
    };

    script
        .data()
        .map_mut()
        .insert(rt.to_atom("param"), param_obj);

    commands.entity(ent).insert(script);
}
