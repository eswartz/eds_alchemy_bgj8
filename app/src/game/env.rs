use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;
use eds_bevy_common::physics::*;
use eds_bevy_common::prelude::*;
use firewheel::Volume;
use rand::RngExt as _;
use rand::seq::IndexedRandom as _;

use crate::assets::FxAssets;

pub(crate) struct EnvPlugin;

impl Plugin for EnvPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_water::WaterPlugin) //
            .insert_resource(bevy_water::WaterSettings {
                spawn_tiles: None,
                amplitude: 0.0,
                ..default()
            }) //
            .add_systems(OnEnter(LevelState::Playing), (remove_water,))
            .add_systems(OnExit(LevelState::Playing), (remove_water,))
            .add_systems(
                FixedPreUpdate,
                (check_water_entry_exit, float_on_water)
                    .run_if(not(is_paused))
                    .run_if(in_state(LevelState::Playing)),
            );
    }
}

fn check_water_entry_exit(
    mut commands: Commands,
    mut reader: MessageReader<AreaChanged>,
    assets: Res<FxAssets>,
) {
    let mut rng = rand::rng();

    for msg in reader.read() {
        if msg.from_area == AreaContent::Water && msg.to_area == AreaContent::Air {
            commands.spawn((
                ChildOf(msg.entity),
                Sfx,
                SamplePlayer::new(
                    (*[&assets.water_exit_1, &assets.water_exit_2]
                        .choose(&mut rand::rng())
                        .unwrap())
                    .clone(),
                )
                .with_volume(Volume::Linear(rng.random_range(0.25..0.5))),
                bevy_seedling::prelude::PlaybackSettings::default()
                    .despawn()
                    .with_playback(true)
                    .with_speed(rng.random_range(0.8..1.2)),
            ));
        } else if msg.from_area == AreaContent::Air && msg.to_area == AreaContent::Water {
            commands.spawn((
                ChildOf(msg.entity),
                Sfx,
                SamplePlayer::new(
                    (*[&assets.water_enter_1, &assets.water_enter_2]
                        .choose(&mut rand::rng())
                        .unwrap())
                    .clone(),
                )
                .with_volume(Volume::Linear(rng.random_range(0.25..0.5))),
                bevy_seedling::prelude::PlaybackSettings::default()
                    .despawn()
                    .with_playback(true)
                    .with_speed(rng.random_range(0.8..1.2)),
            ));
        }
    }
}

/// Marks floating.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Clone, Default)]
#[type_path = "game"]
pub struct Floating;

fn float_on_water(
    mut forces_q: Query<
        (
            &GlobalTransform,
            &ColliderAabb,
            &Mass,
            Forces,
            Has<OurPlayer>,
        ),
        Or<(With<Floating>, With<OurPlayer>)>,
    >,
    water_q: Query<&ColliderAabb, With<Water>>,
    water: bevy_water::WaterParam,
) {
    if water_q.is_empty() {
        return;
    };

    for (gxfrm, my_aabb, mass, mut forces, is_player) in forces_q.iter_mut() {
        let top_y = water.wave_point(gxfrm.translation()).y;
        let half_height = my_aabb.size().y / 2.0;
        let my_y = my_aabb.min.y + half_height / 2.0;

        let dist = my_y - top_y;
        if dist > half_height && !is_player {
            forces.apply_linear_impulse(Vec3::NEG_Y * dist * 0.5 * mass.0);
        } else if dist < -half_height {
            forces.apply_linear_impulse(Vec3::Y * -dist * 0.5 * mass.0);
        }
    }
}

pub(crate) fn remove_water(mut water: If<ResMut<bevy_water::WaterSettings>>) {
    water.amplitude = 0.;
    water.spawn_tiles = None;
    water.height = -100.;
}
