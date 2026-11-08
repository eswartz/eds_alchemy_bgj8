use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_seedling::prelude::AudioSample;
use eds_bevy_common::midi_synth::asset::SoundFont;
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
pub struct MusicAssets {
    #[asset(path = "sounds/Boychoir.sf2")]
    pub boy_choir: Handle<SoundFont>,
}

impl MusicAssets {}

#[derive(Resource, AssetCollection)]
#[allow(unused)]
pub struct FxAssets {
    #[asset(path = "sounds/412017__skymary__cat-meow-short.ogg")]
    pub cat_meow_0: Handle<AudioSample>,
    #[asset(path = "sounds/66518__freemaster2__meow9.ogg")]
    pub cat_meow_1: Handle<AudioSample>,
    #[asset(path = "sounds/656619__kinoton__angry-cat-1.ogg")]
    pub cat_anger_1a: Handle<AudioSample>,
    #[asset(path = "sounds/656619__kinoton__angry-cat-2.ogg")]
    pub cat_anger_1b: Handle<AudioSample>,
    #[asset(path = "sounds/656619__kinoton__angry-cat-3.ogg")]
    pub cat_anger_1c: Handle<AudioSample>,
    #[asset(path = "sounds/205650__myswat__funny-cat-always-grumpy-kitty-1.ogg")]
    pub cat_anger_2a: Handle<AudioSample>,
    #[asset(path = "sounds/205650__myswat__funny-cat-always-grumpy-kitty-2.ogg")]
    pub cat_anger_2b: Handle<AudioSample>,
    #[asset(path = "sounds/205650__myswat__funny-cat-always-grumpy-kitty-3.ogg")]
    pub cat_anger_2c: Handle<AudioSample>,
    #[asset(path = "sounds/205650__myswat__funny-cat-always-grumpy-kitty-4.ogg")]
    pub cat_anger_2d: Handle<AudioSample>,

    #[asset(path = "sounds/157283__samueljustice00__pug-1.ogg")]
    pub dog_1: Handle<AudioSample>,
    #[asset(path = "sounds/157283__samueljustice00__pug-2.ogg")]
    pub dog_2: Handle<AudioSample>,
    #[asset(path = "sounds/157283__samueljustice00__pug-3.ogg")]
    pub dog_3: Handle<AudioSample>,
    #[asset(path = "sounds/273688__richerlandtv__pug-woof-2.ogg")]
    pub dog_4: Handle<AudioSample>,

    #[asset(path = "sounds/clip-enter-532888__bricklover__water-splash-1.ogg")]
    pub water_enter_1: Handle<AudioSample>,
    #[asset(path = "sounds/524498__krizin__pool-water-enter-02.ogg")]
    pub water_enter_2: Handle<AudioSample>,
    #[asset(path = "sounds/clip-exit-532888__bricklover__water-splash-1.ogg")]
    pub water_exit_1: Handle<AudioSample>,
    #[asset(path = "sounds/524498__krizin__pool-water-exit-02.ogg")]
    pub water_exit_2: Handle<AudioSample>,

    #[asset(path = "sounds/140720__j1987__spinning-firecracker.ogg")]
    pub firecracker_1: Handle<AudioSample>,
    #[asset(path = "sounds/126565__ikbenraar__firecracker-tablet-bass.ogg")]
    pub firecracker_2: Handle<AudioSample>,
    #[asset(path = "sounds/328119__dsg__pop-9.ogg")]
    pub pop_1: Handle<AudioSample>,
    #[asset(path = "sounds/789793__quatricise__pop-4.ogg")]
    pub pop_2: Handle<AudioSample>,
    #[asset(path = "sounds/583310__fission9__tile-breaking.ogg")]
    pub break_1: Handle<AudioSample>,
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
