#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]
use bevy_asset::{meta::Settings, Asset, AssetPath, Assets, Handle, UntypedHandle};
use bevy_ecs::{
    bundle::Bundle,
    component::{Component, ComponentHooks, StorageType},
    entity::Entity,
    system::EntityCommands,
    world::EntityWorldMut,
};
use bevy_hierarchy::{BuildChildren, BuildWorldChildren};
pub use default_constructor::InferInto;
use scoped_tls_hkt::scoped_thread_local;
use std::{borrow::Cow, cell::Cell, marker::PhantomData, mem, ptr::null_mut};

mod spawnable;
pub use spawnable::*;

#[doc(hidden)]
pub use bevy_asset::AssetServer;
#[doc(hidden)]
pub use bevy_ecs::system::{Commands, Res};
pub use bevy_spawn_fn_derive::*;
#[doc(hidden)]
pub use default_constructor;

/// Convert an item to a handle by registering using [`AssetServer::add`].
#[doc(hidden)]
pub fn asset<T: Asset>(a: T) -> Handle<T> {
    ASSET_SERVER.with(|s| s.add(a))
}

/// Convert a [`AssetPath`] to a handle by loading using [`AssetServer::load`].
#[doc(hidden)]
pub fn load<T: Asset>(a: AssetPath<'static>) -> Handle<T> {
    ASSET_SERVER.with(|s| s.load(a))
}

// A reference to the spawner scope.
thread_local! {static SPAWNER: Cell<*mut Spawner<'static, 'static, 'static>> = const { Cell::new(null_mut()) } }
scoped_thread_local!(static ASSET_SERVER: AssetServer);

/// Spawn a [`IntoSpawnable`] using a thread local spawner, returns [`Entity`].
///
/// This can be manually created via [`spawner_scope`] or used inside an system or function annotated with
/// [`spawner_fn`] or [`spawner_system`].
///
/// # Syntax
///
/// See [`infer_construct!`] and module level documentation of [`default_constructor`].
#[macro_export]
macro_rules! spawn {
    ($($tt: tt)*) => {
        {
            #[allow(unused)]
            use $crate::default_constructor::effects::*;
            #[allow(unused)]
            use $crate::{asset, load};
            $crate::spawn(
                $crate::default_constructor::meta_default_constructor! {
                    [$crate::default_constructor::infer_into]
                    $($tt)*
                }
            )
        }
    };
}

struct Reset(*mut Spawner<'static, 'static, 'static>);

impl Drop for Reset {
    fn drop(&mut self) {
        SPAWNER.set(self.0);
    }
}

/// Push a [`Spawner`] onto thread local storage in a scope.
pub fn spawner_scope<'a, 'b: 'a, 'c: 'a, T>(
    spawner: &'a mut impl AsSpawner<'a, 'b, 'c>,
    f: impl FnOnce() -> T,
) -> T {
    let mut spawner = spawner.as_spawner();
    let prev = SPAWNER.replace((&mut spawner as *mut Spawner).cast());
    // for panic safety, this will reset the spawner during unwinding.
    let _reset = Reset(prev);
    f()
}

/// Push a [`AssetServer`] onto thread local storage in a scope.
pub fn asset_server_scope<T>(asset_server: &AssetServer, f: impl FnOnce() -> T) -> T {
    ASSET_SERVER.set(asset_server, f)
}

/// Spawn a [`IntoSpawnable`] using the current thread local [`spawner_scope`].
pub fn spawn(spawned: impl IntoSpawnable) -> Entity {
    let ptr = SPAWNER.replace(null_mut());
    // for panic safety, this will reset the spawner during unwinding.
    let __reset = Reset(ptr);
    // Safety: `SPAWNER` is only set by `spawner_scope` and
    // exclusively accessed in `spawn`.
    let spawner = unsafe { ptr.as_mut().expect("Must be called in a spawner scope.") };
    spawner.spawn(spawned)
}

/// A type that can be converted into a [`Bundle`].
pub trait IntoBundle {
    /// Convert to a [`Bundle`].
    fn into_bundle(self) -> impl Bundle;
}

/// A type that can be spawned as an entity.
pub trait Spawnable {
    /// Collects a static bundle of a concrete type.
    fn into_bundle(self) -> impl Bundle;
    /// Collect heterogenous components or bundles from a mutable reference of self.
    ///
    /// A common thing this might do is [`Option::take`] optional bundles and insert them.
    fn spawn_mut<'t>(&mut self, spawner: &'t mut Spawner) -> EntityMutSpawner<'t> {
        spawner.spawn_empty()
    }
    /// Spawn children.
    #[allow(unused_variables)]
    fn spawn_children(&mut self, spawner: &mut Spawner) {}
}

/// A type that can be converted to a [`Spawnable`].
pub trait IntoSpawnable {
    /// Convert to a [`Spawnable`].
    fn into_spawnable(self) -> impl Spawnable;
}

impl<T> IntoBundle for T
where
    T: Bundle,
{
    fn into_bundle(self) -> impl Bundle {
        self
    }
}

impl<T> Spawnable for T
where
    T: IntoBundle,
{
    fn into_bundle(self) -> impl Bundle {
        IntoBundle::into_bundle(self)
    }
}

impl<T> IntoSpawnable for T
where
    T: Spawnable,
{
    fn into_spawnable(self) -> impl Spawnable {
        self
    }
}

/// Create a function scope that can use [`spawn!`] to create children.
pub trait SpawnChildScope {
    /// Create a function scope that can use [`spawn!`] to create children.
    fn spawn_child_scope(&mut self, f: impl FnOnce()) -> &mut Self;
}

impl SpawnChildScope for EntityCommands<'_> {
    fn spawn_child_scope(&mut self, f: impl FnOnce()) -> &mut Self {
        self.with_children(|spawner| spawner_scope(spawner, f))
    }
}

impl SpawnChildScope for EntityWorldMut<'_> {
    fn spawn_child_scope(&mut self, f: impl FnOnce()) -> &mut Self {
        self.with_children(|spawner| spawner_scope(spawner, f))
    }
}

/// [`Component`] that immediately removes itself, adds the underlying
/// value to [`Assets<T>`] and inserts a [`Handle<T>`].
#[derive(Debug)]
pub struct AddMe<T: Asset>(Option<T>);

impl<T: Asset> AddMe<T> {
    pub const fn new(item: T) -> Self {
        AddMe(Some(item))
    }
}

impl<T: Asset> Component for AddMe<T> {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, entity, _| {
            (|| {
                let item = world.get_entity_mut(entity)?.get_mut::<Self>()?.0.take()?;
                let handle = world.resource_mut::<Assets<T>>().add(item);
                world
                    .commands()
                    .entity(entity)
                    .remove::<Self>()
                    .insert(handle);
                Some(())
            })();
        });
    }
}

/// [`Component`] that immediately removes itself, loads the underlying path
/// and inserts a [`Handle<T>`].
pub struct LoadMe<T: Asset> {
    name: Cow<'static, str>,
    settings: Option<Box<dyn FnOnce(&AssetServer, Cow<str>) -> UntypedHandle + Send + Sync>>,
    p: PhantomData<T>,
}

impl<T: Asset> Default for LoadMe<T> {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(""),
            settings: None,
            p: PhantomData,
        }
    }
}

impl<T: Asset> LoadMe<T> {
    pub const fn new_static(path: &'static str) -> Self {
        LoadMe {
            name: Cow::Borrowed(path),
            settings: None,
            p: PhantomData,
        }
    }

    pub fn new(path: impl Into<String>) -> Self {
        LoadMe {
            name: Cow::Owned(path.into()),
            settings: None,
            p: PhantomData,
        }
    }

    pub fn new_with_settings<S: Settings>(
        path: impl Into<String>,
        settings: impl Fn(&mut S) + Send + Sync + 'static,
    ) -> Self {
        LoadMe {
            name: Cow::Owned(path.into()),
            settings: Some(Box::new(|assets, name| {
                UntypedHandle::from(assets.load_with_settings::<T, _>(name.into_owned(), settings))
            })),
            p: PhantomData,
        }
    }
}

impl<T: Asset> Component for LoadMe<T> {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, entity, _| {
            (|| {
                let item = mem::take(world.get_entity_mut(entity)?.get_mut::<Self>()?.as_mut());
                let handle: Handle<T> = if let Some(loader) = item.settings {
                    loader(world.resource::<AssetServer>(), item.name)
                        .try_into()
                        .expect("Expected Handle<T>")
                } else {
                    world.resource::<AssetServer>().load(item.name.into_owned())
                };
                world
                    .commands()
                    .entity(entity)
                    .remove::<Self>()
                    .insert(handle);
                Some(())
            })();
        });
    }
}

#[cfg(test)]
mod test {
    use bevy::app::App;
    use bevy_asset::AssetPlugin;
    use bevy_ecs::{bundle::Bundle, component::Component, system::RunSystemOnce, world::World};
    use bevy_hierarchy::WorldChildBuilder;
    use bevy_spawn_fn_derive::{spawner_fn, spawner_system};

    use crate::IntoBundle;

    #[derive(Component)]
    pub struct A;
    #[derive(Component)]
    pub struct B;

    #[derive(Component)]
    pub struct C;

    #[derive(Bundle)]
    pub struct Abc {
        a: A,
        b: B,
        c: C,
    }

    #[derive(Debug, Default)]
    pub struct IntoAbc {
        a: f32,
        b: String,
        c: char,
    }

    impl IntoBundle for IntoAbc {
        fn into_bundle(self) -> impl Bundle {
            Abc { a: A, b: B, c: C }
        }
    }

    #[spawner_fn]
    fn test1(spawner: &mut World) {
        spawn!(IntoAbc {
            a: 4,
            b: "Ferris",
            c: '\0'
        });
    }

    #[spawner_fn]
    fn test2(spawner: &mut WorldChildBuilder) {
        spawn!(IntoAbc {
            a: 4,
            b: "Ferris",
            c: '\0'
        });
    }

    #[spawner_system]
    fn test3() {
        spawn!(IntoAbc {
            a: 4,
            b: "Ferris",
            c: '\0'
        });
    }

    #[test]
    fn miri_test() {
        let mut world = App::new();
        world.add_plugins(AssetPlugin::default());
        world.world_mut().run_system_once(test3);
    }
}
