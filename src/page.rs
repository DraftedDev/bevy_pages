use crate::element::{Element, ElementId, ElementState};
use bevy::asset::Asset;
use bevy::prelude::{Commands, Entity, GlobalTransform, Transform, TypePath, Visibility, World};
use bevy::ui::Node;
use rustc_hash::FxHashMap;

/// A UI page loaded from an XML file.
///
/// Spawn pages into the world using the [PageManager](crate::manager::PageManager).
/// Don't forget to activate the page using [PageManager::set_active](crate::manager::PageManager::set_active).
#[derive(Debug, Asset, TypePath)]
pub struct Page {
    root: Node,
    entity: Option<Entity>,
    registry: FxHashMap<ElementId, Entity>,
    elements: Vec<Element>,
    active: bool,
}

impl Page {
    /// Creates a new page.
    ///
    /// You should use the [AssetServer] to load UI pages.
    #[inline(always)]
    pub fn new(root: Node, elements: Vec<Element>) -> Self {
        Self {
            root,
            entity: None,
            registry: FxHashMap::with_capacity_and_hasher(elements.len(), Default::default()),
            elements,
            active: false,
        }
    }

    /// Spawn the page into the world.
    pub fn spawn(&mut self, world: &mut World) {
        let root_entity = world
            .spawn((
                self.root.clone(),
                Transform::default(),
                GlobalTransform::default(),
                Visibility::Hidden,
            ))
            .id();

        for element in &self.elements {
            element.spawn(world, Some(root_entity), &mut self.registry);
        }

        self.entity = Some(root_entity);
    }

    /// Retrieves the entity associated with the given element ID.
    ///
    /// For a panic-safe variant, see [try_get](Self::try_get).
    #[inline(always)]
    pub fn get(&self, id: impl Into<ElementId>) -> Entity {
        self.try_get(id).expect("Element not found")
    }

    /// Retrieves the entity associated with the given element ID.
    ///
    /// Returns `None` if the element ID is not found.
    #[inline(always)]
    pub fn try_get(&self, id: impl Into<ElementId>) -> Option<Entity> {
        self.registry.get(&id.into()).copied()
    }

    /// Returns the root entity of the page.
    ///
    /// Returns `None` if the page has not been spawned yet.
    #[inline(always)]
    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }

    /// Returns if the page is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Sets the active state of the page.
    ///
    /// Inactive pages are hidden and cannot be interacted with.
    pub fn set_active(&mut self, commands: &mut Commands, active: bool) {
        let mut entity = commands.entity(self.entity.expect("Page not spawned yet"));

        if active {
            entity.insert((Visibility::Visible, ElementState::Active));
        } else {
            entity.insert((Visibility::Hidden, ElementState::Inactive));
        }

        self.active = active;
    }
}
