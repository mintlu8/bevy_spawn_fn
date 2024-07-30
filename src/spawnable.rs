use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    system::{Commands, EntityCommands},
    world::{EntityWorldMut, World},
};
use bevy_hierarchy::{BuildChildren, BuildWorldChildren, ChildBuilder, WorldChildBuilder};

use crate::{IntoSpawnable, SpawnChildScope, Spawnable};

/// A type that can spawn [`Bundle`]s.
pub trait AsSpawner<'t, 'a, 'b> {
    /// Convert to a [`Spawner`].
    fn as_spawner(&'t mut self) -> Spawner<'t, 'a, 'b>;
}

impl<'t> AsSpawner<'t, 't, 't> for World {
    fn as_spawner(&'t mut self) -> Spawner<'t, 't, 't> {
        Spawner::World(self)
    }
}

impl<'t, 'a, 'b> AsSpawner<'t, 'a, 'b> for Commands<'a, 'b> {
    fn as_spawner(&'t mut self) -> Spawner<'t, 'a, 'b> {
        Spawner::Commands(self)
    }
}

impl<'t, 'a> AsSpawner<'t, 'a, 'a> for ChildBuilder<'a> {
    fn as_spawner(&'t mut self) -> Spawner<'t, 'a, 'a> {
        Spawner::ChildBuilder(self)
    }
}

impl<'t, 'a> AsSpawner<'t, 'a, 'a> for WorldChildBuilder<'a> {
    fn as_spawner(&'t mut self) -> Spawner<'t, 'a, 'a> {
        Spawner::WorldChildBuilder(self)
    }
}

/// All types that can spawn [`Bundle`]s.
pub enum Spawner<'t, 'a, 'b> {
    World(&'t mut World),
    Commands(&'t mut Commands<'a, 'b>),
    ChildBuilder(&'t mut ChildBuilder<'a>),
    WorldChildBuilder(&'t mut WorldChildBuilder<'a>),
    Scoped(Box<dyn ScopedSpawner>),
}

/// Mutable reference to an [`Entity`].
pub enum EntityMutSpawner<'a> {
    EntityWorldMut(EntityWorldMut<'a>),
    EntityCommands(EntityCommands<'a>),
    Scoped(Box<dyn ScopedEntityMut>),
}

impl<'t> EntityMutSpawner<'t> {
    #[inline]
    pub fn insert<B: Bundle>(&mut self, bundle: B) {
        match self {
            EntityMutSpawner::EntityWorldMut(x) => {
                x.insert(bundle);
            }
            EntityMutSpawner::EntityCommands(x) => {
                x.insert(bundle);
            }
            EntityMutSpawner::Scoped(x) => {
                let mut once = Some(bundle);
                x.entity_mut_scope(&mut |x| x.insert(once.take().unwrap()));
            }
        }
    }

    pub fn spawn_children(&mut self, f: impl FnOnce(Spawner)) {
        match self {
            EntityMutSpawner::EntityWorldMut(x) => {
                x.with_children(|x| f(Spawner::WorldChildBuilder(x)));
            }
            EntityMutSpawner::EntityCommands(x) => {
                x.with_children(|x| f(Spawner::ChildBuilder(x)));
            }
            EntityMutSpawner::Scoped(x) => {
                let mut once = Some(f);
                x.entity_mut_scope(&mut |x| x.spawn_children(once.take().unwrap()));
            }
        }
    }

    pub fn id(&self) -> Entity {
        match self {
            EntityMutSpawner::EntityWorldMut(x) => x.id(),
            EntityMutSpawner::EntityCommands(x) => x.id(),
            EntityMutSpawner::Scoped(x) => x.id(),
        }
    }

    /// Create a function scope that can use [`spawn!`](crate::spawn!) to create children.
    pub fn spawn_child_scope(&mut self, f: impl FnOnce()) {
        match self {
            EntityMutSpawner::EntityWorldMut(x) => {
                x.spawn_child_scope(f);
            }
            EntityMutSpawner::EntityCommands(x) => {
                x.spawn_child_scope(f);
            }
            EntityMutSpawner::Scoped(x) => {
                let mut once = Some(f);
                x.entity_mut_scope(&mut move |e| e.spawn_child_scope(once.take().unwrap()))
            }
        }
    }
}

impl Spawner<'_, '_, '_> {
    /// Spawn a empty [`Entity`] with a spawner.
    pub fn spawn_empty(&mut self) -> EntityMutSpawner {
        match self {
            Spawner::World(w) => EntityMutSpawner::EntityWorldMut(w.spawn_empty()),
            Spawner::Commands(w) => EntityMutSpawner::EntityCommands(w.spawn_empty()),
            Spawner::ChildBuilder(w) => EntityMutSpawner::EntityCommands(w.spawn_empty()),
            Spawner::WorldChildBuilder(w) => EntityMutSpawner::EntityWorldMut(w.spawn_empty()),
            Spawner::Scoped(w) => w.spawner_scope(&mut |w| w.spawn_empty().id()),
        }
    }

    /// Spawn a [`Bundle`] with a spawner.
    pub fn spawn_bundle<B: Bundle>(&mut self, bundle: B) -> EntityMutSpawner {
        match self {
            Spawner::World(w) => EntityMutSpawner::EntityWorldMut(w.spawn(bundle)),
            Spawner::Commands(w) => EntityMutSpawner::EntityCommands(w.spawn(bundle)),
            Spawner::ChildBuilder(w) => EntityMutSpawner::EntityCommands(w.spawn(bundle)),
            Spawner::WorldChildBuilder(w) => EntityMutSpawner::EntityWorldMut(w.spawn(bundle)),
            Spawner::Scoped(w) => {
                let mut once = Some(bundle);
                w.spawner_scope(&mut move |w| w.spawn(once.take().unwrap()))
            }
        }
    }

    /// Spawn a [`IntoSpawnable`] with a spawner.
    pub fn spawn(&mut self, spawned: impl IntoSpawnable) -> Entity {
        let mut spawned = spawned.into_spawnable();
        let mut entity_mut = spawned.spawn_mut(self);
        entity_mut.spawn_children(|mut spawner| spawned.spawn_children(&mut spawner));
        entity_mut.insert(spawned.into_bundle());
        entity_mut.id()
    }
}

/// A global dynamic spawner.
///
/// This is meant to support `bevy_defer`.
pub trait ScopedSpawner {
    fn spawner_scope(&mut self, f: &mut dyn FnMut(&mut Spawner) -> Entity) -> EntityMutSpawner;
}

/// A global dynamic spawner.
///
/// This is meant to support `bevy_defer`,
pub trait ScopedEntityMut {
    fn id(&self) -> Entity;
    fn entity_mut_scope(&mut self, f: &mut dyn FnMut(&mut EntityMutSpawner));
}
