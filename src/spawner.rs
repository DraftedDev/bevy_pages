use crate::page::Page;
use bevy::asset::{Assets, Handle};
use bevy::prelude::{AssetServer, Commands, Entity, Event, On, Res, ResMut, Resource};

pub(crate) fn spawn_page(
    mut commands: Commands,
    mut spawner: ResMut<PageSpawner>,
    server: Res<AssetServer>,
    mut assets: ResMut<Assets<Page>>,
) {
    if let Some(handle) = &spawner.handle
        && let Some(mut page) = assets.remove(handle)
    {
        let entity = page.spawn(&mut commands, &server);
        commands.insert_resource(page);

        spawner.handle = None;
        spawner.spawned = Some(entity);

        commands.trigger(SpawnPage);
    }
}

pub(crate) fn despawn_page(
    _: On<DespawnPage>,
    mut commands: Commands,
    mut spawner: ResMut<PageSpawner>,
) {
    if let Some(entity) = spawner.spawned {
        commands.entity(entity).despawn();
        commands.remove_resource::<Page>();

        spawner.spawned = None;
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
    pub fn spawn(&mut self, handle: Handle<Page>) {
        self.handle = Some(handle);
    }

    /// Despawns the current page.
    ///
    /// Won't panic if no page is currently spawned.
    ///
    /// This will directly trigger [DespawnPage].
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
