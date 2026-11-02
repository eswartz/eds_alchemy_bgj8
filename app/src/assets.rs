use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_seedling::prelude::AudioSample;
use fedry_bevy_plugin::asset::ScriptModule;

#[derive(Resource, AssetCollection)]
pub struct GuiAssets {
    #[asset(path = "fonts/UncialAntiqua-Regular.ttf")]
    pub uncial_antiqua_font: Handle<Font>,
    #[asset(path = "fonts/SymbolsNerdFont-Regular.ttf")]
    pub symbols_nerd_font: Handle<Font>,

    #[asset(path = "textures/background.webp")]
    pub menu_background_image: Handle<Image>,
}

#[derive(Resource, AssetCollection)]
#[allow(unused)]
pub struct MusicAssets {}

impl MusicAssets {}

#[derive(Resource, AssetCollection)]
#[allow(unused)]
pub struct FxAssets {}

#[derive(Resource, AssetCollection)]
pub struct MapAssets {
    #[asset(path = "maps/level_0.glb#Scene0")]
    pub level_0: Handle<WorldAsset>,
}

#[derive(Resource, AssetCollection)]
pub struct ModelAssets {}

#[derive(Resource, AssetCollection)]
pub struct ScriptAssets {}
