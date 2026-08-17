use crate::element::{Element, ElementId, ElementRegistry};
use bevy::asset::{Asset, AssetServer};
use bevy::prelude::{Commands, Entity, GlobalTransform, Resource, Transform, TypePath};
use bevy::ui::Node;

/// A UI page loaded from an XML file.
///
/// Spawn pages into the world using the [PageSpawner](crate::spawner::PageSpawner).
#[derive(Debug, Resource, Asset, TypePath)]
pub struct Page {
    root: Node,
    entity: Option<Entity>,
    registry: ElementRegistry,
    elements: Vec<Element>,
}

impl Page {
    /// Creates a new page.
    ///
    /// You should use the [AssetServer] to load UI pages instead of this function.
    pub fn new(root: Node, elements: Vec<Element>) -> Self {
        Self {
            root,
            entity: None,
            registry: ElementRegistry::new(elements.len()),
            elements,
        }
    }

    pub(crate) fn spawn(&mut self, commands: &mut Commands, assets: &AssetServer) -> Entity {
        let entity = commands
            .spawn((
                self.root.clone(),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .with_children(|parent| {
                for element in &self.elements {
                    element.spawn(parent, assets, &mut self.registry);
                }
            })
            .id();

        self.entity = Some(entity);

        entity
    }

    /// Retrieves the entity associated with the given element ID.
    ///
    /// For a panic-safe variant, see [try_get](Self::try_get).
    pub fn get(&self, id: impl Into<ElementId>) -> Entity {
        self.try_get(id).expect("Element not found")
    }

    /// Retrieves the entity associated with the given element ID.
    ///
    /// Returns `None` if the element ID is not found.
    pub fn try_get(&self, id: impl Into<ElementId>) -> Option<Entity> {
        self.registry.get_element(id.into())
    }

    /// Returns the root entity of the page.
    ///
    /// Returns `None` if the page has not been spawned yet.
    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }
}
