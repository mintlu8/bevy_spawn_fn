# bevy_spawn_fn

Awesome spawning experience for bevy.

## Getting Started

Annotate your system with `#[spawner_system]`, then use the `spawn!` macro.

```rust
#[spawner_system]
pub fn particle_emitter(emitter: Res<ParticleEmitter>) {
    if emitter.should_spawn() {
        spawn! {
            ParticleBundle {
                color: Color::Green,
                size: 10.0,
                texture: @load "images/my_image.png"
            }
        }
    }
}
```

If the function not a system, use the `#[spawner_fn]` macro,
which takes less liberty in rewriting the function.

## The `spawn!` macro

`spawn!` spawns a `IntoSpawnable` and return an `Entity`.

The macro uses the `infer_construct!` macro from
the [`default_constructor`](https://docs.rs/default-constructor) crate under the hood,
which uses the `InferInto` trait for conversion.

Additionally effect `@load` can be used to load `Handle<T>` from
a string path and `@asset` can be used to convert `impl Into<T>` to `Handle<T>`
via `AssetServer`.

## The `Spawnable` Trait

`Spawnable` is a superset of `Bundle` that can be implemented to spawn
heterogenous bundles and children.

`IntoSpawnable` is free ergonomics on top of `Spawnable`!

## Versions

| bevy | bevy_spawn_fn      |
|------|--------------------|
| 0.13 | latest             |

## License

License under either of

Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)
at your option.

## Contribution

Contributions are welcome!

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
