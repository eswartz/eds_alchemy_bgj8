use crate::assets::*;
use crate::game::BoomMass;
use crate::game::GameScript;
use eds_bevy_common::prelude::*;

use bevy::prelude::*;
use fedry_bevy_plugin::Scripting;

use fedry_bevy_plugin::prelude::*;

pub(crate) const ID: &str = "level0";
pub(crate) const NAME: &str = "Level 0";

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

fn on_level_loaded(
    mut commands: Commands,
    world: Res<WorldMarkerEntity>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    scripting: Scripting<GameScript>,
    assets: Res<AssetServer>,
    script_assets: Res<ScriptAssets>,
    model_assets: Res<ModelAssets>,
    ctx_p: ConvertContextParam,
) -> Result {
    let ctx = ctx_p.as_ctx();

    commands.insert_resource(InstructionText(
        r#"
        DON'T DIE...?
        "#
        .to_string(),
    ));

    commands.insert_resource(BoomMass(100.0));
    // let script =
    //     scripting.new_script_from_module_id(script_assets.level_0.id(), ExecutionMode::Async)?;
    // let runtime = scripting.runtime;

    // let script_module = script.module();

    Ok(())
}
