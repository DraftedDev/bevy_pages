use crate::element::{Element, ElementId};
use bevy::asset::Asset;
use bevy::prelude::{Entity, GlobalTransform, Resource, Transform, TypePath, World};
use bevy::ui::Node;
use rustc_hash::FxHashMap;

/// A UI page loaded from an XML file.
///
/// Spawn pages into the world using the [PageSpawner](crate::spawner::PageSpawner).
#[derive(Debug, Resource, Asset, TypePath)]
pub struct Page {
    root: Node,
    entity: Option<Entity>,
    registry: FxHashMap<ElementId, Entity>,
    elements: Vec<Element>,
}

impl Page {
    /// Creates a new page.
    ///
    /// You should use the [AssetServer] to load UI pages instead of this function.
    #[inline(always)]
    pub fn new(root: Node, elements: Vec<Element>) -> Self {
        Self {
            root,
            entity: None,
            registry: FxHashMap::with_capacity_and_hasher(elements.len(), Default::default()),
            elements,
        }
    }

    #[inline(always)]
    pub(crate) fn spawn(&mut self, world: &mut World) -> Entity {
        let root_entity = world
            .spawn((
                self.root.clone(),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        for element in &self.elements {
            element.spawn(world, Some(root_entity), &mut self.registry);
        }

        self.entity = Some(root_entity);
        root_entity
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
}
