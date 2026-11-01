//! A simple example dialogue view for Yarn Spinner.
//! A dialogue view is a plugin that handles presenting lines and options to the user and advances the dialogue on user input.
//! This one shows text in a dialogue box inspired by Legend of Zelda: Breath of the Wild.
//!
//! ## Demo
//!
//! The [Yarn Spinner for Rust Demo](https://janhohenheim.itch.io/yarnspinner-rust-demo) uses this dialogue view, so you can play that in the browser if you
//! want to see it in action. Additionally, all [Bevy Yarn Spinner examples](https://github.com/YarnSpinnerTool/YarnSpinner-Rust/tree/main/examples/bevy_yarnspinner/src/bin) use
//! this dialogue view as well.
//!
//! ## Usage
//!
//! It's enough to simply register [`AlchemyDialogViewPlugin`] alongside [`YarnSpinnerPlugin`]:
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_yarnspinner::*;
//! use bevy_yarnspinner::prelude::YarnSpinnerPlugin;
//! use bevy_yarnspinner_example_dialogue_view::prelude::*;
//!
//! App::new()
//!    .add_plugins(DefaultPlugins)
//!    .add_plugins(YarnSpinnerPlugin::new())
//!    .add_plugins(AlchemyDialogViewPlugin::new());
//! ```
//!
//! This crate also exposes the [`SpeakerChangeEvent`] which you can use to animate characters while they are speaking,
//! as the text is written out over a few seconds.
//!
//! ## Inputs
//!
//! - Advance the dialogue: press the space bar, enter key, left click or tap the screen after the text is done typing.
//! - Type out the text faster: Same as above, but hold press before the text is done typing.
//! - Select an option: press the number key corresponding to the option you want to select or click/tap the option.
//!
//! ## Limitations
//!
//! This dialogue view expects only a single instance of [`DialogueRunner`](bevy_yarnspinner::prelude::DialogueRunner) to be running.
//! Its behavior is otherwise undefined.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(missing_docs, missing_debug_implementations)]

use bevy::prelude::*;
use bevy_yarnspinner::prelude::YarnSpinnerPlugin;

/// The plugin registering all systems of the dialogue view.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct AlchemyDialogViewPlugin;

/// The [`SystemSet`] containing all systems added by the [`AlchemyDialogViewPlugin`].
/// Is run after the [`YarnSpinnerSystemSet`](bevy_yarnspinner::prelude::YarnSpinnerSystemSet).
#[derive(Debug, Default, Clone, Copy, SystemSet, Eq, PartialEq, Hash)]
pub struct AlchemyDialogViewSystemSet;

impl AlchemyDialogViewPlugin {
    /// Creates a new example dialogue view
    pub fn new() -> Self {
        Self::default()
    }
}

/// Set the font that appears in dialogs.
#[derive(Resource)]
pub struct AlchemyDialogFont(pub Handle<Font>);

/// Marker for the [`Node`] that is the root of the dialog UI.
#[derive(Debug, Default, Component)]
pub struct DialogueRootNode;

mod option_selection;
mod setup;
mod typewriter;
mod updating;

impl Plugin for AlchemyDialogViewPlugin {
    fn build(&self, app: &mut App) {
        assert!(
            app.is_plugin_added::<YarnSpinnerPlugin>(),
            "YarnSpinnerPlugin must be added before AlchemyDialogViewPlugin"
        );
        app.add_plugins(setup::ui_setup_plugin)
            .add_plugins(updating::ui_updating_plugin)
            .add_plugins(typewriter::typewriter_plugin)
            .add_plugins(option_selection::option_selection_plugin);
    }
}
