use crate::page::Page;
use bevy::asset::{Assets, Handle};
use bevy::prelude::{Commands, Entity, Event, On, ResMut, Resource, World};

pub(crate) fn spawn_page(world: &mut World) {
    let handle = world.resource_mut::<PageSpawner>().handle.take();

    if let Some(handle) = handle
        && let Some(mut page) = world.resource_mut::<Assets<Page>>().remove(&handle)
    {
        let entity = page.spawn(world);

        world.insert_resource(page);
        world.resource_mut::<PageSpawner>().spawned = Some(entity);

        world.trigger(SpawnPage);
    }
}

pub(crate) fn despawn_page(
    _: On<DespawnPage>,
    mut commands: Commands,
    mut spawner: ResMut<PageSpawner>,
) {
    let spawned = spawner.spawned.take();

    if let Some(entity) = spawned {
        commands.entity(entity).despawn();
        commands.remove_resource::<Page>();
    }
}

/// Integrated resource to spawn UI pages.
///
/// Since the asset loader loads assets asynchronously, the spawner must be used to spawn pages.
///
/// The spawner itself is just a resource with a handle to the page to spawn.
#[derive(Resource)]
pub struct PageSpawner {
    handle: Option<Handle<Page>>,
    spawned: Option<Entity>,
}

impl PageSpawner {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            handle: None,
            spawned: None,
        }
    }

    /// Spawns a new page.
    ///
    /// **NOTE:** Make sure to [PageSpawner::despawn] any existing page before spawning a new one.
    ///
    /// This will indirectly trigger [SpawnPage].
    #[inline(always)]
    pub fn spawn(&mut self, handle: Handle<Page>) {
        self.handle = Some(handle);
    }

    /// Despawns the current page.
    ///
    /// Won't panic if no page is currently spawned.
    ///
    /// This will directly trigger [DespawnPage].
    #[inline(always)]
    pub fn despawn(&mut self, commands: &mut Commands) {
        commands.trigger(DespawnPage);
    }
}

/// An event triggered when a [Page] is spawned into the world.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Event)]
pub struct SpawnPage;

/// An event triggered when a [Page] is despawned from the world.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Event)]
pub struct DespawnPage;
