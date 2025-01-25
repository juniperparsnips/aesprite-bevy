mod aseprite;

pub use aseprite::*;
use bevy::{app::Plugin, asset::AssetApp};

/// Re-export of dynastes
pub use dynastes;

#[derive(Default)]
pub struct AsepritePlugin;

impl Plugin for AsepritePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<AsepriteAnimation>()
            .init_asset_loader::<AsepriteLoader>();
    }
}
