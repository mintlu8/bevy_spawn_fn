use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    render::color::Color,
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_spawn_fn::*;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .run();
}

#[spawner_system]
fn startup() {
    spawn!(Camera2dBundle {});
    
    spawn!(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(2., 2., 2.),
            custom_size: @some [64., 64.],
        },
        texture: @load "circle.png",
        transform: Transform {
            translation: [20., 0., 0.]
        }
    });
}
