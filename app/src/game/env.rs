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
        app.add_systems(
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
    mut commands: Commands,
    mut forces_q: Query<(&GlobalTransform, &ColliderAabb, &Mass, Forces), With<Floating>>,
    water_q: Query<&ColliderAabb, With<Water>>,
    time: Res<Time>,
) {
    let mut rng = rand::rng();

    let Ok(water_aabb) = water_q.single() else {
        return;
    };
    let top_y = water_aabb.max.y;

    for (gxfrm, my_aabb, mass, mut forces) in forces_q.iter_mut() {
        // let my_y = gxfrm.translation().y;
        let half_height = my_aabb.size().y / 2.0;
        let my_y = my_aabb.min.y + half_height / 2.0;
        // dbg!(top_y, my_y);

        // Fake wave
        let x = (gxfrm.translation().x % 5.0) / 5.0 * std::f32::consts::TAU;
        let z = (gxfrm.translation().z % 5.0) / 5.0 * std::f32::consts::TAU;
        let sx = (x + time.elapsed_secs() / 4.0).cos();
        let sz = (z + time.elapsed_secs() / 4.0).sin();

        // let top_y = top_y + (sx + sz) * half_height;
        let pert = (sx * sz).abs() * half_height / 2.0;

        let dist = my_y - top_y + pert;
        if dist > half_height {
            forces.apply_linear_impulse(Vec3::NEG_Y * dist * 0.5 * mass.0);
        } else if dist < -half_height {
            forces.apply_linear_impulse(Vec3::Y * -dist * 0.5 * mass.0);
        }
    }
}
