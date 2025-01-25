use aseprite_bevy::aseprite::{AsepriteAnimation, AsepriteLoader};
use bevy::{asset::WaitForAssetError, prelude::*, window::WindowResolution};
use dynastes::{AnimationStateMachine, Dynastes, DynastesPlugin};
use thiserror::Error;

fn main() {
    let scale = 6.0;
    let mut res = WindowResolution::new(480.0 * scale, 270.0 * scale);
    res.set_scale_factor_override(Some(scale));

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: res,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(DynastesPlugin::<AsepriteAnimation>::default())
        .init_asset::<AsepriteAnimation>()
        .init_asset_loader::<AsepriteLoader>()
        .add_systems(Startup, setup)
        .run();
}

#[derive(Debug, Error)]
enum SetupError {
    #[error(transparent)]
    WaitForAsset(#[from] WaitForAssetError),
}

fn setup(mut commands: Commands<'_, '_>, asset_server: Res<'_, AssetServer>) {
    let animation_handle: Handle<AnimationStateMachine<AsepriteAnimation>> =
        asset_server.load("RH-Frontanim.dyn");

    commands.spawn(Camera2d);
    commands.spawn((
        Dynastes(animation_handle),
        Transform::from_scale(Vec3::splat(1.0)),
    ));
}
