mod action_handlers;
mod bevy_funcs;
mod damage_effect;
mod dialogue;
mod env;
mod firing;
mod gravity;
mod gravity_compute;
mod sound;

mod level_0;
// mod level_1;
// mod level_2;
// mod level_3;
// mod level_4;
// mod level_5;
// mod level_6;

use fedry_bevy_plugin::script_debug::ScriptDebugPlugin;

use std::time::Duration;
use strum::{EnumIter, VariantArray};

use bevy::asset::uuid::Uuid;
use bevy::color::palettes::tailwind;
use bevy::ecs::world::CommandQueue;
use bevy::mesh::*;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use fedry_bevy_plugin::prelude::{
    FedryScriptingPlugin, ScriptRoot, ScriptTypeBase, pause_scripting, register_script_key,
    unpause_scripting,
};

use eds_bevy_common::midi_synth::prelude::*;
use eds_bevy_common::physics::*;
use eds_bevy_common::prelude::*;

use crate::assets::GuiAssets;
use crate::assets::ModelAssets;
use crate::game::bevy_funcs::register_script_funcs;
use crate::game::damage_effect::DamageEffectPlugin;
use crate::game::dialogue::decorate_dialog_cues;
use crate::game::dialogue::spawn_dialogue_runner;
use crate::game::env::EnvPlugin;
use crate::game::gravity::GravityPlugin;
use crate::game::gravity_compute::GravityComputePlugin;
use crate::game::sound::SoundPlugin;
pub use action_handlers::*;

/// Start dialogue at the given node name.
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
#[type_path = "game"]
pub(crate) struct DialogueCue(pub String);

pub struct GamePlugin;

#[derive(Message, Debug)]
struct AddPlayerDamage {
    player: Entity,
    giver: Entity,
    amount: f32,
}

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
struct PlayerHealth {
    /// current
    value: f32,
    /// increase per second
    heal_rate: i32,
    /// max allowable
    max_health: i32,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity((9.8 * Vec3::NEG_Y).into()))
            //
            .add_message::<AddPlayerDamage>()
            .add_plugins(ActionHandlersPlugin)
            .add_plugins(EnvPlugin)
            .add_plugins(GravityPlugin)
            // .add_plugins(FiringPlugin)
            .add_plugins(SoundPlugin)
            .add_plugins(FedryScriptingPlugin)
            .add_systems(Startup, register_script_funcs)
            .add_plugins(ScriptDebugPlugin)
            .add_plugins(DetailNormalPlugin)
            .add_plugins(SplitIntoCubesPlugin)
            //
            .add_systems(
                PreUpdate,
                (
                    wake_up_spawned_if_floating.run_if(resource_changed::<Gravity>),
                    wake_up_spawned
                        .run_if(resource_changed::<Gravity>)
                        .run_if(not(is_in_menu)),
                )
                    .run_if(in_state(ProgramState::InGame)),
            )
            .insert_resource(GrabbingBehavior {
                ignore_mass: false,
                ..default()
            })
            .init_resource::<LevelDifficulty>()
            .add_plugins(DamageEffectPlugin)
            .add_plugins(level_0::LevelPlugin)
            // .add_plugins(level_1::LevelPlugin)
            // .add_plugins(level_2::LevelPlugin)
            // .add_plugins(level_3::LevelPlugin)
            // .add_plugins(level_4::LevelPlugin)
            // .add_plugins(level_5::LevelPlugin)
            // .add_plugins(level_6::LevelPlugin)
            .insert_resource(BaseEntity(Entity::PLACEHOLDER, Transform::IDENTITY))
            .add_observer(on_scene_ready)
            .add_systems(OnEnter(ProgramState::New), setup_styles)
            .add_systems(OnExit(ProgramState::New), ensure_levels)
            .add_systems(
                OnEnter(GameplayState::Setup),
                (level_spawn_started, spawn_level).chain(),
            )
            .add_systems(
                OnExit(GameplayState::Setup),
                (level_spawn_finished,).chain(),
            )
            .add_systems(
                FixedUpdate,
                (init_player_settings, spawn_player_on_start)
                    .chain()
                    .run_if(added_player_start) // <<< only once per session, in practice
                    .run_if(in_state(GameplayState::Playing)),
            )
            .add_systems(
                OnEnter(LevelState::LevelLoaded),
                (
                    |mut commands: Commands| {
                        commands.set_state(LevelState::Configuring);
                    },
                    setup_skybox,
                    unpause_scripting,
                    spawn_dialogue_runner,
                    decorate_dialog_cues,
                )
                    .chain(),
            )
            .add_systems(OnExit(LevelState::Playing), (pause_scripting,))
            .add_systems(
                Update,
                (report_raycast,)
                    .run_if(not(is_paused))
                    .run_if(not(is_in_menu))
                    .run_if(is_level_active)
                    .run_if(not(debug_gui_wants_direct_input))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame)),
            )
            // .add_systems(OnEnter(LevelState::Playing), show_power_bar)
            // .add_systems(OnExit(LevelState::Playing), remove_power_bar)
            // .add_systems(
            //     Update,
            //     update_power_bar
            //         .run_if(not(is_paused))
            //         .run_if(in_state(ProgramState::InGame))
            //         .run_if(in_state(GameplayState::Playing)),
            // )
            .add_systems(OnEnter(LevelState::Won), won_level)
            .add_systems(OnEnter(LevelState::Lost), lost_level)
            .add_systems(OnEnter(LevelState::Enter), start_game)
            .add_systems(OnEnter(LevelState::Advance), advance_level)
            .add_systems(
                FixedUpdate,
                (update_current_score,)
                    .run_if(not(is_in_menu))
                    .run_if(in_state(LevelState::Playing))
                    .run_if(in_state(ProgramState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (
                    check_next_level,
                    check_won_level
                        .run_if(in_state(LevelState::Won))
                        .run_if(not(is_paused)),
                    check_lost_level
                        .run_if(in_state(LevelState::Lost))
                        .run_if(not(is_paused)),
                )
                    .run_if(not(is_in_menu))
                    .run_if(in_state(ProgramState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (check_player_health,)
                    .run_if(not(is_paused))
                    .run_if(not(is_in_menu))
                    .run_if(in_state(ProgramState::InGame)),
            )
            .add_systems(
                First,
                spawn_midi_synths
                    .run_if(resource_exists::<CommonSoundFontAssets>)
                    .run_if(not(is_in_menu))
                    .run_if(in_state(ProgramState::InGame)),
            )
            .insert_resource(DepthMapStorage {
                orig_to_edited: default(),
            });

        register_script_key::<GameScript>(app);
    }
}

pub(crate) fn setup_styles(
    mut commands: Commands,
    gui_assets: Res<GuiAssets>,
    mut menu_style: ResMut<MenuStyle>,
) {
    let the_font = &gui_assets.main_font;

    *menu_style = menu_style
        .clone()
        .with_font(the_font.clone())
        .with_page_color(Color::BLACK.into()) // .
        .with_page_image(Some(gui_assets.menu_background_image.clone())) // .
    ;
    commands.insert_resource(InstructionFont(the_font.clone()));
    commands.insert_resource(crate::yarn_ui::AlchemyDialogFont(the_font.clone()));
}

/// Current difficulty.
#[derive(Resource, Default, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub struct LevelDifficulty(pub Difficulty);

#[derive(Resource, Debug, Clone, Reflect, Deref)]
#[reflect(Resource)]
#[type_path = "game"]
pub(crate) struct BoomMass(f32);

/// Marker for scripts driven by the game itself.
#[derive(Debug, Default, Clone, PartialEq, Hash, Reflect)]
#[type_path = "fedry"]
pub(crate) struct GameScript;

impl ScriptTypeBase for GameScript {}

/// Difficulty rating.
#[derive(
    Resource,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    Reflect,
    EnumIter,
    strum_macros::Display,
    VariantArray,
)]
#[reflect(Resource)]
#[type_path = "game"]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

/// The current score.
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource, Default)]
#[type_path = "game"]
pub struct CurrentScore {
    pub score: i32,
}

const END_LEVEL_DELAY_SECS: u64 = 3;

/// Countdown to next or same level.
#[derive(Resource, Reflect, Default)]
#[reflect(Resource, Default)]
#[type_path = "game"]
pub struct AutoEndLevelTimer {
    pub(crate) timer: Timer,
}

impl AutoEndLevelTimer {
    pub fn new(delay: Duration) -> Self {
        Self {
            timer: Timer::new(delay, TimerMode::Once),
        }
    }
}

// World state

/// Our "base" object and its initial transform.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
#[type_path = "game"]
pub(crate) struct BaseEntity(pub Entity, pub Transform);

// Player state

/// Marker for an object (e.g. net) in the hand.
#[derive(Component)]
#[allow(unused)]
pub(crate) struct InHand;

fn on_scene_ready(
    ready: On<WorldInstanceReady>,
    children_q: Query<&Children>,
    meshes_q: Query<&Mesh3d, Without<CollisionLayers>>,
    mut commands: Commands,
) {
    for entity in children_q.iter_descendants(ready.entity) {
        if meshes_q.contains(entity) {
            commands.entity(entity).insert((CollisionLayers::new(
                GameLayer::World,
                [
                    GameLayer::Default,
                    GameLayer::World,
                    GameLayer::Player,
                    GameLayer::Projectiles,
                ],
            ),));
        }
    }
}

pub(crate) fn ensure_levels(mut level_list: ResMut<LevelList>) {
    level_list.0.sort_by(|a, b| a.id.cmp(&b.id));
}

pub(crate) fn level_spawn_started(mut commands: Commands, mut pause: ResMut<PauseState>) {
    commands.set_state(LevelState::Initializing);
    commands.set_state(OverlayState::Loading);

    // Prevent moving/interacting while loading UI is up.
    pause.set_menu_paused(true);
}

pub(crate) fn level_spawn_finished(
    mut commands: Commands,
    mut pause: ResMut<PauseState>,
    sensable_q: Query<Entity, With<DeathboxCollider>>,
) {
    for ent in sensable_q.iter() {
        commands
            .entity(ent)
            .insert((Sensor, CollisionEventsEnabled, CollidingEntities::default()));
    }

    commands.set_state(OverlayState::Hidden);
    commands.set_state(LevelState::LevelLoaded);

    // Go for it, user (unless they did set_user_paused)
    pause.set_menu_paused(false);
}

fn added_player_start(q: Query<&Transform, Added<PlayerStart>>, mut commands: Commands) -> bool {
    let Some(xfrm) = q.iter().next() else {
        return false;
    };

    commands.insert_resource(StationaryCameraTransform(*xfrm));

    true
}

pub(crate) fn spawn_player_on_start(world: &mut World) {
    let mut start_q = world.query_filtered::<&Transform, With<PlayerStart>>();
    let Some(xfrm) = start_q.iter(world).next() else {
        log::error!("no PlayerStart");
        return;
    };

    drop(start_q);
    let xfrm = xfrm.clone();

    // Make the player collision model and Player
    let player_ent = spawn_fps_player(
        world,
        Uuid::default(),
        Vec3::new(0.5, 1.5, 0.3),
        xfrm.clone(),
    );

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, world);

    // Put and orient the new Player where the PlayerStart is.
    commands.entity(player_ent).insert((
        PlayerLook {
            rotation: xfrm.rotation,
            ..default()
        },
        xfrm,
    ));

    // Set model.
    let models = world.resource::<ModelAssets>();
    commands
        .entity(player_ent)
        .insert(WorldAssetRoot(models.player.clone()));

    commands
        .spawn((
            ChildOf(player_ent),
            CollisionEventsEnabled,
            Sensor,
            Collider::sphere(0.2),
            Transform::from_xyz(0.0, 1.0, 0.0),
        ))
        .observe(bonk_on_the_head);

    commands.insert_resource(PlayerHealth {
        value: 100.0,
        heal_rate: 1,
        max_health: 100,
    });

    queue.apply(world);
}

pub(crate) fn setup_level(
    mut commands: Commands,
    level_list: &LevelList,
    level_index: &LevelIndex,
) {
    let index = level_index.0;
    if index >= level_list.0.len() {
        log::error!("no items in LevelList");
        commands.remove_resource::<CurrentLevel>();
        commands.set_state(ProgramState::Error);
        return;
    }

    let level = &level_list.0[level_index.0];
    commands.insert_resource(CurrentLevel {
        index,
        info: level.clone(),
    });
}

pub(crate) fn spawn_level(
    mut commands: Commands,
    level_list: Res<LevelList>,
    level_index: Res<LevelIndex>,
    world: Res<WorldMarkerEntity>,
    script_root: Option<ResMut<ScriptRoot>>,
) {
    setup_level(commands.reborrow(), &level_list, &level_index);

    if level_index.0 >= level_list.0.len() {
        return;
    }

    let level = &level_list.0[level_index.0];
    log::info!("Entering level {}", level.label);

    // Put any script content here.
    if let Some(mut script_root) = script_root {
        script_root.0 = world.0;
    }

    commands
        .spawn((
            DespawnOnExit(GameplayState::Playing),
            WorldAssetRoot(level.scene.clone()),
            ChildOf(world.0),
        ))
        .observe(|_event: On<WorldInstanceReady>, mut commands: Commands| {
            commands.set_state(GameplayState::Playing);
        });

    commands.insert_resource(CurrentScore::default());
}

fn init_player_settings(
    move_q: Query<&PlayerCameraMode, With<LevelRoot>>,
    mut commands: Commands,
    mut settings: ResMut<PlayerInputSettings>,
) {
    if let Ok(mode) = move_q.single() {
        match mode.0 {
            PlayerMode::Fps => *settings = PlayerInputSettings::for_fps(),
            PlayerMode::Space => *settings = PlayerInputSettings::for_space(),
        }
        commands.insert_resource(mode.0);
    } else {
        log::warn!("no PlayerCameraMode in LevelRoot");
    }
}

pub(crate) fn start_game(mut commands: Commands) {
    commands.set_state(OverlayState::Loading);
    commands.set_state(ProgramState::InGame);
    commands.set_state(GameplayState::AssetsLoaded);
    // commands.insert_resource(ConnectToServer);
    commands.set_state(LevelState::Advance);
}

pub(crate) fn advance_level(mut commands: Commands) {
    commands.set_state(OverlayState::Loading);
    commands.set_state(GameplayState::Setup);
}

fn update_current_score(
    mut commands: Commands,
    level_state: Res<State<LevelState>>,
    score: Option<Res<CurrentScore>>,
    gui_area: GuiAreaMarkerLocator,
    // mut score_q: Single<(&mut Text, &mut TextColor), With<ScoreArea>>,
    mut score_q: Query<(&mut Text, &mut TextColor)>,
) {
    gui_area.with_first(GuiAreaMarker::ScoreArea, |ent| {
        let Ok((ref mut text, ref mut color)) = score_q.get_mut(ent) else {
            return;
        };
        if score.is_some() {
            if *level_state == LevelState::Playing {
                // let won = score.score >= goal.goal as _;
                // let lost = score.score <= goal.lose;

                let won = false;
                let lost = false;
                text.0 = String::new();
                color.0 = Color::Srgba(if won {
                    tailwind::LIME_300
                } else if lost {
                    tailwind::RED_700
                } else {
                    tailwind::GRAY_100
                });

                if won {
                    commands.set_state(LevelState::Won);
                } else if lost {
                    commands.set_state(LevelState::Lost);
                }
            }
        } else {
            text.0.clear();
        }
    });
}

/// Apply the [NextLevelIndex] value, if set.
fn check_next_level(
    mut level_index: ResMut<LevelIndex>,
    next_level_index_opt: Option<ResMut<NextLevelIndex>>,
    mut commands: Commands,
) {
    next_level_index_opt.map(|next_level| {
        commands.remove_resource::<NextLevelIndex>();
        *level_index = LevelIndex(next_level.0);
        commands.set_state(ProgramState::InGame);
    });
}

fn won_level(
    mut commands: Commands,
    gui_area: GuiAreaMarkerLocator,
    mut score_q: Query<(&mut Text, &mut TextColor)>,
) {
    gui_area.with_first(GuiAreaMarker::ScoreArea, |ent| {
        let Ok((ref mut text, ref mut color)) = score_q.get_mut(ent) else {
            return;
        };
        text.0 = "Passed!".to_string();
        color.0 = Color::Srgba(tailwind::LIME_300);
    });
    commands.insert_resource(AutoEndLevelTimer::new(Duration::from_secs(
        END_LEVEL_DELAY_SECS,
    )));
}

fn lost_level(
    mut commands: Commands,
    gui_area: GuiAreaMarkerLocator,
    mut score_q: Query<(&mut Text, &mut TextColor)>,
) {
    gui_area.with_first(GuiAreaMarker::GameStatusArea, |ent| {
        let Ok((ref mut text, ref mut color)) = score_q.get_mut(ent) else {
            return;
        };
        text.0 = "Failed...\nTry again!".to_string();
        color.0 = Color::Srgba(tailwind::RED_700);
    });
    commands.insert_resource(AutoEndLevelTimer::new(Duration::from_secs(
        END_LEVEL_DELAY_SECS,
    )));
}

fn check_won_level(
    mut commands: Commands,
    mut end_timer: ResMut<AutoEndLevelTimer>,
    time: Res<Time>,
    level_index: ResMut<LevelIndex>,
    level_list: Res<LevelList>,
) {
    if !end_timer.timer.tick(time.delta()).is_finished() {
        return;
    }

    let next_index = level_index.0 + 1;
    if next_index >= level_list.0.len() {
        commands.set_state(ProgramState::Completed);
        commands.set_state(LevelState::Initializing);
        commands.set_state(GameplayState::Done);
        commands.set_state(OverlayState::GameOverScreen);
        // Next time we restart, be at level 0.
        commands.insert_resource(LevelIndex(0));
    } else {
        commands.insert_resource(NextLevelIndex(next_index));
        commands.set_state(LevelState::Advance);
    }
}

fn check_lost_level(
    mut commands: Commands,
    mut end_timer: ResMut<AutoEndLevelTimer>,
    time: Res<Time>,
) {
    if !end_timer.timer.tick(time.delta()).is_finished() {
        return;
    }

    // Restarts level.
    commands.set_state(LevelState::Enter);
}

fn setup_skybox(
    mut commands: Commands,
    skybox_q: Query<Entity, (With<SkyboxModel>,)>,
    cam_q: Query<Entity, (With<Camera3d>, With<WorldCamera>)>,
    skyboxes: If<Res<CommonSkyboxAssets>>,
) {
    let Ok(cam) = cam_q.single() else { return };

    // If there isn't one in the level, add a default.
    if skybox_q.is_empty() {
        //let with_reflection_probe = Some(SkyboxReflectionProbeModel::default());  // looks ... not so good when real lights are present
        let with_reflection_probe = None;

        commands.entity(cam).insert(SkyboxModel {
            image: Some(skyboxes.starmap_2020.clone()),
            brightness: bevy::prelude::light_consts::lux::HALLWAY,
            mapping: CubemapMapping::PxNxPyFxFyNyFxFyPzNz,
            with_reflection_probe,
            ..default()
        });
    }
    commands.insert_resource(SkyboxSetup::WaitingSkybox);
}

/// When placed on an entity, ensure a [MidiSynth] is wired up
/// and can process events.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct OurMidiSynth;

pub(crate) fn spawn_midi_synths(
    mut commands: Commands,
    sf_assets: Res<CommonSoundFontAssets>,
    muted: Res<MidiSynthsPaused>,
    synth_q: Query<Entity, (With<OurMidiSynth>, Without<MidiSynth>)>,
    mut synth_map: ResMut<SynthProxyMap>,
) -> Result<()> {
    let params = MidiSynthParams::default().is_world_positioned(true);

    for ent in synth_q.iter() {
        let (sample_sender, sample_receiver) = crossbeam_channel::unbounded();
        let synth = MidiSynth::new(
            params.clone(),
            sf_assets.timgm6mb.clone(),
            muted.0.clone(),
            ent,
            sample_sender,
            sample_receiver,
        )?;
        commands.entity(ent).insert((synth, Sfx));

        synth_map.register_synth(ent);

        commands.write_message(SynthMessage::new(ent, SynthCommand::Reset));
    }

    Ok(())
}

/// Used to save off [GravityScale] for purposes of inducing sleeping.
#[derive(Component, Debug, Reflect, Clone, PartialEq)]
#[reflect(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct OrigGravityScale {
    pub orig_scale: Scalar,
    pub restore_next_time: bool,
}

fn report_raycast(
    gui_area: GuiAreaMarkerLocator,
    mut info_q: Query<(&mut Text, &mut TextColor, &mut Visibility)>,
    highlighting_mode: Option<Res<HighlightingMode>>,
    crosshair_target: If<Res<CrosshairTargets>>,
    names_q: Query<Option<&Name>>,
    gui_state: Res<GuiState>,
    mut last_target_desc: Local<(CrosshairTargets, String)>,
) {
    if !dev_tools_enabled() {
        return;
    }

    gui_area.with_first(GuiAreaMarker::InfoArea, |ent| {
        let Ok((ref mut text, ref mut color, ref mut visibility)) = info_q.get_mut(ent) else {
            return;
        };
        if highlighting_mode
            .as_ref()
            .is_none_or(|highlighting_mode| !highlighting_mode.is_disabled())
            && gui_state.enabled
            && !crosshair_target.targets.is_empty()
            && let Some(message) = if last_target_desc.0 == **crosshair_target {
                // Same as last tick.
                Some(last_target_desc.1.clone())
            } else {
                // Recompute.
                if let Some(message) = report_crosshair_targets(&crosshair_target, &names_q) {
                    *last_target_desc = (crosshair_target.clone(), message.clone());
                    Some(message)
                } else {
                    None
                }
            }
        {
            visibility.set_if_neq(Visibility::Inherited);
            text.0 = message;
            color.0 = Color::Srgba(tailwind::GRAY_100);
        } else {
            visibility.set_if_neq(Visibility::Hidden);
        }
    });
}

/// Avian doesn't reliably wake up (or cause [RigidBody]s to move)
/// when changing Gravity. Maybe that's intentional, maybe a bug?
fn wake_up_spawned_if_floating(
    mut commands: Commands,
    collisions: Res<ContactGraph>,
    sleep_q: Query<Entity, (With<Sleeping>, With<Spawned>, Without<Player>)>,
) {
    for ent in sleep_q.iter() {
        if collisions.entities_colliding_with(ent).next().is_none() {
            commands.entity(ent).remove::<Sleeping>();
        }
    }
}

/// Avian doesn't reliably wake up (or cause [RigidBody]s to move)
/// when changing Gravity. Maybe that's intentional, maybe a bug?
fn wake_up_spawned(
    mut commands: Commands,
    sleep_q: Query<Entity, (With<Sleeping>, With<Spawned>, Without<Player>)>,
) {
    for ent in sleep_q.iter() {
        commands.entity(ent).remove::<Sleeping>();
    }
}

fn bonk_on_the_head(
    event: On<CollisionStart>,
    vel_q: Query<&LinearVelocity>,
    mass_q: Query<&Mass>,
    mut commands: Commands,
) {
    let Some(player) = &event.body1 else {
        return;
    };
    let Ok(player_vel) = vel_q.get(*player) else {
        return;
    };
    let Ok(other_vel) = vel_q.get(event.collider2) else {
        return;
    };
    let player_mass = if let Ok(mass) = mass_q.get(event.collider2) {
        mass.0
    } else {
        10.0
    };
    let other_mass = if let Ok(mass) = mass_q.get(event.collider2) {
        mass.0
    } else {
        1.0
    };
    let offs = other_vel.0 - player_vel.0;
    if offs.y < -1.0 {
        commands.write_message(AddPlayerDamage {
            player: *player,
            giver: event.body2.unwrap_or(event.collider2),
            amount: offs.y.abs() * other_mass / player_mass,
        });
    }
}

fn check_player_health(
    mut commands: Commands,
    mut health: If<ResMut<PlayerHealth>>,
    mut reader: MessageReader<AddPlayerDamage>,
    time: Res<Time>,
) {
    if health.value <= 0.0 {
        return;
    };

    let mut damage_amount = 0.0f32;
    for msg in reader.read() {
        damage_amount += msg.amount;
    }

    let heal_amount = time.delta_secs() * health.heal_rate as f32;
    let new_value =
        (health.value + heal_amount - damage_amount).clamp(0.0, health.max_health as f32);

    if health.value != new_value {
        debug!("health: {}", new_value);
        health.value = new_value;

        if new_value <= 0. {
            commands.set_state(LevelState::Lost);
        }
    }
}
