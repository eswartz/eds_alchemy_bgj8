use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_seedling::prelude::AudioSample;
use fedry_bevy_plugin::asset::ScriptModule;

#[derive(Resource, AssetCollection)]
pub struct GuiAssets {
    // #[asset(path = "fonts/UncialAntiqua-Regular.ttf")]
    #[asset(path = "fonts/IrishGrover-Regular.ttf")]
    pub main_font: Handle<Font>,
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
pub struct FxAssets {
    #[asset(path = "sounds/412017__skymary__cat-meow-short.ogg")]
    pub cat_meow_0: Handle<AudioSample>,
    #[asset(path = "sounds/66518__freemaster2__meow9.ogg")]
    pub cat_meow_1: Handle<AudioSample>,
    #[asset(path = "sounds/157283__samueljustice00__pug-1.ogg")]
    pub dog_1: Handle<AudioSample>,
    #[asset(path = "sounds/157283__samueljustice00__pug-2.ogg")]
    pub dog_2: Handle<AudioSample>,
    #[asset(path = "sounds/157283__samueljustice00__pug-3.ogg")]
    pub dog_3: Handle<AudioSample>,
    #[asset(path = "sounds/273688__richerlandtv__pug-woof-2.ogg")]
    pub dog_4: Handle<AudioSample>,
}

#[derive(Resource, AssetCollection)]
pub struct MapAssets {
    #[asset(path = "maps/level_0.glb#Scene0")]
    pub level_0: Handle<WorldAsset>,
}

#[derive(Resource, AssetCollection)]
pub struct ModelAssets {
    #[asset(path = "models/player.glb#Scene0")]
    pub player: Handle<WorldAsset>,
}

#[derive(Resource, AssetCollection)]
pub struct ScriptAssets {
    #[asset(path = "scripts/driver.dry")]
    pub driver: Handle<ScriptModule>,
    #[asset(path = "scripts/entries.dry")]
    pub entries: Handle<ScriptModule>,
}
