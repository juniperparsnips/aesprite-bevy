use aseprite_bevy::aseprite::{AsepriteAnimation, AsepriteFrame, AsepriteLoader};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<State>()
        .init_asset::<AsepriteAnimation>()
        .init_asset_loader::<AsepriteLoader>()
        .add_systems(Startup, setup)
        // .add_systems(Update, print_on_load)
        .run();
}

#[derive(Resource, Default)]
struct State {
    handle: Handle<AsepriteAnimation>,
    printed: bool,
}

fn setup(mut state: ResMut<State>, asset_server: Res<AssetServer>) {
    // Recommended way to load an asset
    state.handle = asset_server.load("RH-Frontanim.json");

    // 2024-12-13T05:00:21.587601Z ERROR bevy_asset::server: Failed to load asset 'RH-Frontanim.json' with asset loader
    // 'aseprite_bevy::aseprite::AsepriteLoader': Could not parse JSON: expected value at line 2 column 13

    let thing = r#"{
      "filename": "RH-Frontanim 0.aseprite",
      "frame": {
        "x": 0,
        "y": 0,
        "w": 128,
        "h": 128
      },
      "rotated": false,
      "trimmed": false,
      "spriteSourceSize": {
        "x": 0,
        "y": 0,
        "w": 128,
        "h": 128
      },
      "sourceSize": {
        "w": 128,
        "h": 128
      },
      "duration": 66
    }"#;
    let parsed = serde_json::from_str::<AsepriteFrame>(thing);
    println!("{parsed:?}")
}

fn print_on_load(mut state: ResMut<State>, aseprite_assets: Res<Assets<AsepriteAnimation>>) {
    let aseprite_asset = aseprite_assets.get(&state.handle);

    // Can't print results if the assets aren't ready
    if state.printed {
        return;
    }

    if aseprite_asset.is_none() {
        info!("Aseprite Not Ready");
        return;
    }

    info!("Aseprite asset loaded: {:?}", aseprite_asset.unwrap());

    // Once printed, we won't print again
    state.printed = true;
}
