#![allow(deprecated, reason = "old BEI")]
use bevy::prelude::*;
#[cfg(feature = "input_bei")]
use bevy_enhanced_input::prelude::*;
use eds_bevy_common::physics::*;
use eds_bevy_common::prelude::*;

pub(crate) struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "input_bei")]
        app.add_systems(Startup, create_input_map);

        app.add_systems(
            Update,
            (check_actions,)
                .run_if(not(is_paused))
                .run_if(not(is_in_menu))
                .run_if(is_level_active)
                .run_if(not(debug_gui_wants_direct_input))
                .run_if(in_state(LevelState::Playing))
                .run_if(in_state(ProgramState::InGame)),
        );
    }
}

#[cfg(feature = "input_bei")]
pub(crate) fn create_input_map(mut commands: Commands) {
    use bevy_enhanced_input::prelude::ActionOf;

    use crate::actions::assign_extra_actions;

    let menu_entity = commands.spawn((MenuContext, Name::new("MenuContext"))).id();
    let include = (ActionOf::<MenuContext>::new(menu_entity),);
    assign_stock_common_actions(commands.reborrow(), include.clone());
    assign_stock_menu_actions(commands.reborrow(), include.clone());

    ///////

    let player_entity = commands
        .spawn((PlayerContext, Name::new("PlayerContext")))
        .id();

    let include = (ActionOf::<PlayerContext>::new(player_entity),);
    assign_stock_common_actions(commands.reborrow(), include.clone());
    assign_stock_player_actions(commands.reborrow(), include.clone());
    assign_extra_actions(commands.reborrow(), include.clone());

    ///////

    let dialog_entity = commands
        .spawn((DialogContext, Name::new("DialogContext")))
        .id();

    let include = (ActionOf::<DialogContext>::new(dialog_entity),);
    assign_stock_common_actions(commands.reborrow(), include.clone());
    assign_stock_dialog_actions(commands.reborrow(), include.clone());
}

fn check_actions(_player_mode: ResMut<PlayerMode>) {}

#[cfg(feature = "input_bei")]
pub mod game_actions {}

#[cfg(feature = "input_bei")]
pub fn assign_extra_actions(mut commands: Commands, include: impl Bundle + Clone) {
    // We need this when mod keys distinguish actions.
    let consume = ActionSettings {
        consume_input: true,
        ..default()
    };

    ////////

    commands.spawn((
        include.clone(),
        Action::<actions::TogglePhysics>::new(),
        bindings![KeyCode::ScrollLock,],
    ));
    commands.spawn((
        include.clone(),
        Action::<actions::TogglePhysicsGizmos>::new(),
        consume.clone(),
        bindings![
            KeyCode::ScrollLock.with_mod_keys(MOD_CTRL_COMMAND | ModKeys::ALT),
            KeyCode::KeyP.with_mod_keys(MOD_CTRL_COMMAND | ModKeys::ALT),
        ],
    ));

    commands.spawn((
        include.clone(),
        Action::<actions::ChangeCamera>::new(),
        consume.clone(),
        bindings![KeyCode::KeyV.with_mod_keys(MOD_CTRL_COMMAND),],
    ));

    commands.spawn((
        include.clone(),
        Action::<actions::Zoom>::new(),
        ExponentialCurve::splat(1.25),
        Negate::y(),
        Bindings::spawn((
            Spawn((
                Binding::mouse_wheel(),
                SwizzleAxis::YYY,
                default_mouse_wheel_scale(1.0),
            )),
            Bidirectional::new(KeyCode::ArrowUp, KeyCode::ArrowDown),
            Bidirectional::new(GamepadButton::RightTrigger2, GamepadButton::LeftTrigger2),
        )),
    ));
}
