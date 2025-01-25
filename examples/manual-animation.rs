#![feature(random)]

use std::{
    random::{DefaultRandomSource, Random},
    time::Duration,
};

use aseprite_bevy::{AsepriteAnimation, AsepritePlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AsepritePlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .add_systems(Update, render_on_load)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let animation_handle: Handle<AsepriteAnimation> = asset_server.load("RH-Frontanim.json");

    commands.spawn(Camera2d);
    commands.spawn(FakeDynastes(animation_handle));
}

#[derive(Component)]
struct FakeDynastes(Handle<AsepriteAnimation>);

#[derive(Component)]
struct AnimationState(String);

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &FakeDynastes,
        &mut AnimationState,
        &mut AnimationTimer,
        &mut Sprite,
    )>,
    aseprite_assets: Res<Assets<AsepriteAnimation>>,
) {
    for (dynastes, mut state_name, mut timer, mut sprite) in &mut query {
        let Some(animation) = aseprite_assets.get(&dynastes.0) else {
            println!("This shouldn't happen?");
            continue;
        };

        let Some(state) = animation.states.get(&state_name.0) else {
            println!("Neither should this.");
            continue;
        };

        timer.tick(time.delta());

        let mut should_swap_state = false;

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index == state.last {
                    should_swap_state = true;
                } else {
                    atlas.index = atlas.index + 1;

                    let Some(duration) = state.durations.get(atlas.index - state.first) else {
                        println!("No frames in state");
                        continue;
                    };

                    if timer.times_finished_this_tick() > 1 {
                        println!(
                            "lag experienced. {} frames missed",
                            timer.times_finished_this_tick()
                        );
                    }

                    timer.set_duration(Duration::from_millis(*duration as u64));
                };
            }
        }

        if should_swap_state {
            // Very inefficiently select a new random state
            let ordered = animation.states.iter().collect::<Vec<_>>();
            let Some((new_state_name, new_state)) =
                ordered.get(usize::random(&mut DefaultRandomSource) % ordered.len())
            else {
                println!("No states!");
                continue;
            };

            println!("new state: {new_state_name}");

            let Some(first_duration) = new_state.durations.get(0) else {
                println!("No frames in state");
                continue;
            };

            sprite.texture_atlas = Some(new_state.atlas.clone());
            state_name.0 = new_state_name.to_string();
            timer.0 = Timer::new(
                Duration::from_millis(*first_duration as u64),
                TimerMode::Repeating,
            )
        }
    }
}

fn render_on_load(
    mut commands: Commands,
    mut unloaded: Query<(Entity, &FakeDynastes), Without<Sprite>>,
    aseprite_assets: Res<Assets<AsepriteAnimation>>,
) {
    for (entity, dynastes) in &mut unloaded {
        let Some(animation) = aseprite_assets.get(&dynastes.0) else {
            continue;
        };

        // Get an arbitrary first state
        let Some((state_name, state)) = animation.states.iter().next() else {
            println!("No states!");
            continue;
        };

        let Some(first_duration) = state.durations.get(0) else {
            println!("No frames in state");
            continue;
        };

        commands.entity(entity).insert((
            Sprite::from_atlas_image(animation.image.clone(), state.atlas.clone()),
            Transform::from_scale(Vec3::splat(2.0)),
            AnimationState(state_name.clone()),
            AnimationTimer(Timer::new(
                Duration::from_millis(*first_duration as u64),
                TimerMode::Repeating,
            )),
        ));
    }
}
