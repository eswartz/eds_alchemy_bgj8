use bevy::camera::visibility::RenderLayers;
use bevy_tweening::{EaseMethod, Tween, TweenAnim};
use std::time::Duration;

use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use eds_bevy_common::prelude::*;

use crate::game::PlayerHealth;

pub struct DamageEffectPlugin;

impl Plugin for DamageEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(LevelState::Playing), add_damage_effect)
            .add_systems(OnExit(LevelState::Playing), remove_damage_effect)
            .add_systems(OnEnter(ProgramState::LaunchMenu), remove_damage_effect)
            .add_systems(
                Update,
                update_damage_effect
                    .run_if(not(is_paused))
                    .run_if(in_state(LevelState::Playing)),
            );
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct DamageEffect;

pub(crate) fn add_damage_effect(mut commands: Commands) {
    commands.spawn((
        DamageEffect,
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(tailwind::RED_500.with_alpha(0.0).into()),
        RenderLayers::from_layers(&[RENDER_LAYER_UI]),
    ));
}

pub(crate) fn remove_damage_effect(
    mut commands: Commands,
    damage_q: Single<Entity, With<DamageEffect>>,
) {
    commands.entity(*damage_q).try_despawn();
}

pub(crate) fn update_damage_effect(
    player_q: Single<&PlayerHealth>,
    effects: Single<(Entity, &DamageEffect, &mut BackgroundColor)>,
    mut commands: Commands,
    mut last_health: Local<f32>,
) {
    let (ent, _effect, _bg) = effects.into_inner();
    let pstate = *player_q;
    let (cur_health, max_health) = (pstate.value, pstate.max_health as f32);
    let diff = cur_health - *last_health;
    if diff < 0. {
        let pain_ranking = (max_health - cur_health) / max_health;
        let strength = (-diff / (cur_health + 0.01)).clamp(0.0, 1.0);
        let time = pain_ranking / 1.0;
        let lens = BackgroundColorLens {
            start: tailwind::RED_500.with_alpha(strength).into(),
            end: tailwind::RED_500
                .with_alpha((pain_ranking - 0.5).max(0.0))
                .into(),
        };
        let bg_color_tween = Tween::new(
            EaseMethod::EaseFunction(EaseFunction::BackOut),
            Duration::from_secs_f32(time),
            lens,
        );
        commands.entity(ent).insert(TweenAnim::new(bg_color_tween));
    }

    *last_health = cur_health;
}
