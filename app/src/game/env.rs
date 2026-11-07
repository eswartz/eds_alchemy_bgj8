use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;
use eds_bevy_common::prelude::*;
use firewheel::Volume;
use rand::RngExt as _;
use rand::seq::IndexedRandom as _;

use crate::assets::FxAssets;

pub(crate) struct EnvPlugin;

impl Plugin for EnvPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            check_water_entry_exit
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
