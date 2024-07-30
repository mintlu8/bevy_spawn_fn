use bevy::{
    app::{App, Startup},
    color::Color,
    core_pipeline::core_2d::Camera2dBundle,
    math::{Vec2, Vec3},
    prelude::{TransformBundle, VisibilityBundle},
    render::texture::Image,
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_asset::Handle;
use bevy_spawn_fn::*;
use default_constructor::infer_construct;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .run();
}

#[derive(Debug, Default)]
pub struct ManySprites {
    color: Color,
    size: Vec2,
    texture: Handle<Image>,
    root_pos: Vec3,
    positions: Vec<Vec3>,
}

impl Spawnable for ManySprites {
    fn into_bundle(self) -> impl bevy::prelude::Bundle {
        TransformBundle {
            local: Transform {
                translation: self.root_pos,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn spawn_children(&mut self, spawner: &mut Spawner) {
        for pos in self.positions.iter().copied() {
            spawner.spawn_bundle(infer_construct! {
                SpriteBundle {
                    sprite: Sprite {
                        color: self.color,
                        custom_size: @some self.size,
                    },
                    texture: self.texture.clone(),
                    transform: Transform {
                        translation: pos
                    }
                }
            });
        }
    }
}

#[spawner_system]
fn startup() {
    spawn!(Camera2dBundle);

    spawn!(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(2., 2., 2.),
            custom_size: @some [64., 64.],
        },
        texture: @load "circle.png",
        transform: Transform {
            translation: [20., 0., 0.]
        }
    });

    spawn!(
        Sprite {
            color: Color::srgb(2., 2., 2.),
            custom_size: @some [64., 64.],
        },
        (__spawn_asset_server.load::<Image>("circle.png")),
        TransformBundle {
            local: Transform {
                translation: [-20., 0., 0.]
            }
        },
        VisibilityBundle,
    );

    spawn!(ManySprites {
        color: Color::srgb(3., 3., 3.),
        size: [40., 40.],
        texture: @load "circle.png",
        root_pos: [-20., 0., 0.],
        positions: @arr [
            [10., 0., 1.],
            [-5., -5., 2.],
            [-5., 5., 3.],
        ]
    });
}
