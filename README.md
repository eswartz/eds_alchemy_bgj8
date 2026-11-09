# What a Mess!

(Called `eds-alchemy` on disk)

This is my game jam entry for Bevy Game Jam #8.

The theme is ALCHEMY.

# How to Play

Just go to the metal object and engage in conversation.
Try the options multiple times.

# Bugs?

If on startup, you get a big red error message and a "ran out of fuel"
message on the console, try again...

# Controls

Typical WASD navigation plus controller support. 

* `WASD`: movement
* mouse movement: look
* `Backspace`: turn around
* `Space`/`c`: jump/crouch
* `Pause` / `Alt-P` (Win/Lin) / `Ctrl-P` (macOS): pause
* `F12` / `Alt-M` (Win/Lin) / `Ctrl-M` (macOS): mute
* `F11` / `Alt-Enter`: toggle fullscreen
* `Esc`: menu

## Menus

* `up/down`: select item
* `left/right`: decrease/increase value/enum
* `mouse drag`: decrease/increase value/enum
* `backspace`: reset value to default

# Major Crates Used

Although I did the jam solo, hundreds of other people did the hard work!

- Rust nightly -- whoo!
- `bevy` 0.20
  - I use a few custom tricks to make webp textures work in glTF files
- `skein` (Blender/glTF integration) -- patched to let unit struct components work
- `avian` (physics) -- this entry exercises this a lot! No patches!
- `bevy_seedling` + `Firewheel` (audio) -- another worthy set of crates, patched locally for Bevy 0.20
- `bevy_enhanced_input` -- as-is
- `bevy-inspector-egui` -- as-is
- `bevy_yarn_spinner` (dialogue) -- also patched locally for 0.20
- `bevy_water` (water effect) -- thrown in at the last minute
- `bevy_app_compute` (compute shaders) -- two levels of patching (PR + bevy 0.20 migration)
- `bevy_mod_outline`, `bevy_tweening` -- used but not really exercised here
- 
# Patches

```
bevy_seedling = { git = "https://github.com/eswartz/bevy_seedling-fork.git", branch = "bevy-0.20" }
firewheel = { git = "https://github.com/eswartz/Firewheel-fork.git", branch = "bevy-0.20" }
firewheel-core = { git = "https://github.com/eswartz/Firewheel-fork.git", branch = "bevy-0.20" }
firewheel-ircam-hrtf = { git = "https://github.com/eswartz/bevy_seedling-fork.git", branch = "bevy-0.20" }
firewheel-web-audio = { git = "https://github.com/eswartz/bevy_seedling-fork.git", branch = "bevy-0.20" }
bevy_app_compute = { git = "https://github.com/eswartz/bevy_app_compute-fork.git" }
bevy_skein = { git = "https://github.com/eswartz/skein-fork.git", branch = "main" }
bevy_yarnspinner = { git = "https://github.com/eswartz/YarnSpinner-Rust-fork.git", branch = "bevy-0.20" }
eds_bevy_common = { git = "https://github.com/eswartz/eds_bevy_common.git", branch = "bevy-20" }
```

# Personal Sources

- `src` -- own work (though some bits were stolen from other unpublished experiments of mine)
- `eds_bevy_common` (framework) -- my long-baking framework which saved most of the effort, though I spent a bit of time again in Bevy 0.20 migration
- `fedry` / `fedry_bevy` (scripting) -- my personal scripting effort, private for now. This entry was testing this quite a bit.

# Hacking

* Set `DEBUG=1` in the environment to get access to the "`" key to get debug UI.

# Assets

- `assets/maps/*`: own work (made in Blender)
- `assets/dialogue/*`: own work
- `assets/scripts/*`: own work
- `assets/shaders/*`: own work
- `assets/textures/*`: own work (made in Krita)
- `assets/models/cat*.glb` + `assets/models/pug*.glb`: work derived from free models via https://www.cgtrader.com/designers/pixelcgstudio2025?utm_source=credit
  These were adapted using techniques in:
  https://blender.stackexchange.com/questions/258583/how-can-you-fill-one-meshs-volume-with-as-many-of-another-mesh-that-will-fit
  plus a lot of Blender effort!
- `assets/sounds/*`: from freesound.org (name prefixes retained), edited in ocenaudio
- `assets/fonts/*`: from egui, Google Fonts (OFL, Apache, MIT -- see accompanying licenses)
