use bevy::prelude::*;
use bevy_yarnspinner::prelude::{DialogueRunner, YarnProject};
use eds_bevy_common::{
    layers::GameLayer,
    physics::{
        ColliderConstructor, ColliderConstructorHierarchy, CollisionEventsEnabled, CollisionLayers,
        CollisionStart, RigidBody, Sensor,
    },
    states_sets::LevelState,
};

use crate::game::DialogueCue;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
#[type_path = "game"]
pub(crate) struct MyDialogueRunner;

pub(crate) fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    let dialogue_runner = project.create_dialogue_runner(&mut commands);
    commands.spawn((
        DespawnOnExit(LevelState::Playing),
        Name::new("DIALOGUE"),
        dialogue_runner,
        MyDialogueRunner,
    ));
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
    cue_q: Query<&DialogueCue>,
    mut runner: Single<&mut DialogueRunner, With<MyDialogueRunner>>,
) {
    if let Ok(cue) = cue_q.get(event.collider1) {
        if !runner.is_running() {
            runner.start_node(&cue.0);
        }
    }
}
