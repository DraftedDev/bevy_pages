use crate::events::{ActivatePage, DeactivatePage, DespawnPage, SpawnPage};
use crate::page::Page;
use bevy::asset::{Assets, Handle};
use bevy::prelude::{Commands, On, ResMut, Resource, World};
use rustc_hash::FxHashMap;
use smol_str::{SmolStr, ToSmolStr};

pub(crate) fn spawn_page(world: &mut World) {
    let handles = world
        .resource_mut::<PageManager>()
        .handles
        .drain()
        .collect::<Vec<_>>();

    for (name, page) in handles {
        if let Some(mut page) = world.resource_mut::<Assets<Page>>().remove(&page) {
            page.spawn(world);

            world
                .resource_mut::<PageManager>()
                .pages
                .insert(name.clone(), page);

            world.trigger(SpawnPage { name });
        }
    }
}

pub(crate) fn de_activate_pages(mut commands: Commands, mut manager: ResMut<PageManager>) {
    let requests: Vec<(SmolStr, bool)> = manager
        .active_requests
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();

    for (name, active) in requests {
        if let Some(page) = manager.pages.get_mut(&name) {
            page.set_active(&mut commands, active);
            manager.active_requests.remove(&name);

            if active {
                commands.trigger(ActivatePage { name });
            } else {
                commands.trigger(DeactivatePage { name });
            }
        }
    }
}

pub(crate) fn despawn_page(
    event: On<DespawnPage>,
    mut commands: Commands,
    mut manager: ResMut<PageManager>,
) {
    if let Some(page) = manager.pages.remove(&event.name) {
        commands
            .entity(page.entity().expect("Cannot despawn non-spawned page"))
            .despawn();
    }
}

/// Integrated resource to manage UI pages.
///
/// Handles page (de)spawning and (de)activation.
#[derive(Resource)]
pub struct PageManager {
    handles: FxHashMap<SmolStr, Handle<Page>>,
    pages: FxHashMap<SmolStr, Page>,
    active_requests: FxHashMap<SmolStr, bool>,
}

impl PageManager {
    /// Creates a new empty [PageManager] instance.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            handles: FxHashMap::with_capacity_and_hasher(1, Default::default()),
            pages: FxHashMap::with_capacity_and_hasher(1, Default::default()),
            active_requests: FxHashMap::with_capacity_and_hasher(1, Default::default()),
        }
    }

    /// Returns [true] if any page is currently active.
    pub fn any_active(&self) -> bool {
        self.pages.values().any(|p| p.is_active())
    }

    /// Returns [true] if there are any active requests.
    ///
    /// Active requests are created when a page is (de)activated.
    pub fn has_active_requests(&self) -> bool {
        !self.active_requests.is_empty()
    }

    /// Returns a reference to the page with the given name, if it exists.
    pub fn get(&self, name: impl ToSmolStr) -> Option<&Page> {
        self.pages.get(&name.to_smolstr())
    }

    /// Spawns a new page.
    ///
    /// **NOTE:** Make sure to [PageManager::despawn] any existing page before spawning a new one.
    ///
    /// This will indirectly trigger [SpawnPage].
    #[inline(always)]
    pub fn spawn(&mut self, name: impl ToSmolStr, handle: Handle<Page>) {
        self.handles.insert(name.to_smolstr(), handle);
    }

    /// Activates or deactivates a page.
    ///
    /// This will indirectly trigger [ActivatePage] or [DeactivatePage].
    #[inline(always)]
    pub fn set_active(&mut self, name: impl ToSmolStr, active: bool) {
        self.active_requests.insert(name.to_smolstr(), active);
    }

    /// Despawns the current page.
    ///
    /// Won't panic if no page is currently spawned.
    ///
    /// This will directly trigger [DespawnPage].
    #[inline(always)]
    pub fn despawn(&mut self, commands: &mut Commands, name: impl ToSmolStr) {
        commands.trigger(DespawnPage {
            name: name.to_smolstr(),
        });
    }
}

impl Default for PageManager {
    fn default() -> Self {
        Self::new()
    }
}
