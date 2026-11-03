use crate::assets::*;
use bevy::prelude::*;
use eds_bevy_common::prelude::*;

pub(crate) const INSTRUCTIONS: &str = r#"
This game has no end.
"#;

pub(crate) const ID: &str = "level0";
pub(crate) const NAME: &str = "Chapter 0";

fn register_level(mut list: ResMut<LevelList>, maps: Res<MapAssets>) {
    list.0.push(LevelInfo {
        id: ID.to_string(),
        label: NAME.to_string(),
        scene: maps.level_0.clone(),
    });
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ProgramState::New), register_level)
            .add_systems(
                OnEnter(LevelState::LevelLoaded),
                on_level_loaded.run_if(is_in_level(ID)),
            );
    }
}

fn on_level_loaded(mut commands: Commands) {
    commands.insert_resource(InstructionText(INSTRUCTIONS.to_string()));
}
