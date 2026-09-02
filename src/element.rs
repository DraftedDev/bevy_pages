use crate::events::ElementSpawn;
use crate::props::Properties;
use crate::widgets::Widget;
use bevy::color::Color;
use bevy::prelude::{
    BorderColor, ChildOf, Component, Entity, GlobalTransform, Interaction, Transform, World,
};
use bevy::ui::{BackgroundColor, Node};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// A UI element representing a node in the UI hierarchy.
#[derive(Debug)]
pub struct Element {
    /// The element widget.
    pub widget: Box<dyn Widget>,
    /// The core visual, layout, and widget properties.
    pub props: Properties<ElementProps>,
    /// The optional ID of this element.
    pub id: Option<ElementId>,
    /// The children of this element.
    pub children: Vec<Element>,
}

impl Element {
    /// Spawn the element into the world.
    pub fn spawn(
        &self,
        world: &mut World,
        parent: Option<Entity>,
        reg: &mut FxHashMap<ElementId, Entity>,
    ) -> Entity {
        let props = &self.props.default;

        let root_entity = world
            .spawn((
                self.props.clone(),
                props.node.clone(),
                Interaction::None,
                Transform::default(),
                GlobalTransform::default(),
                BackgroundColor(props.bg_color.unwrap_or(Color::NONE)),
                props.border_color.unwrap_or(BorderColor::DEFAULT),
                ElementState::Inactive,
            ))
            .id();

        if let Some(parent_entity) = parent {
            world.entity_mut(root_entity).insert(ChildOf(parent_entity));
        }

        if let Some(id) = &self.id {
            world.entity_mut(root_entity).insert(id.clone());
        }

        let target_entity = self.widget.spawn(root_entity, world);

        for child in &self.children {
            child.spawn(world, Some(target_entity), reg);
        }

        if let Some(id) = self.id() {
            reg.insert(id, root_entity);
        }

        world.trigger(ElementSpawn {
            entity: root_entity,
            id: self.id(),
        });

        root_entity
    }

    /// Returns the [ElementId] of the element, if it has one
    #[inline(always)]
    pub fn id(&self) -> Option<ElementId> {
        self.id.clone()
    }
}

impl Clone for Element {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.dyn_clone(),
            props: self.props.clone(),
            id: self.id.clone(),
            children: self.children.clone(),
        }
    }
}

/// Pure visual and layout properties for any element.
#[derive(Clone, Debug, Default)]
pub struct ElementProps {
    /// The node layout settings behind this element.
    pub node: Node,
    /// The background color of this element.
    pub bg_color: Option<Color>,
    /// The border color of this element.
    pub border_color: Option<BorderColor>,
}

/// A unique identifier for an element.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Component)]
pub struct ElementId(SmolStr);

impl ElementId {
    /// Creates a new [ElementId].
    #[inline(always)]
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(SmolStr::new(s))
    }

    /// Creates a new [ElementId] from a static string.
    #[inline(always)]
    pub fn new_static(s: &'static str) -> Self {
        Self(SmolStr::new_static(s))
    }
}

impl From<&'static str> for ElementId {
    #[inline(always)]
    fn from(s: &'static str) -> Self {
        Self::new_static(s)
    }
}

impl PartialEq<str> for ElementId {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        self.0.as_str() == other
    }
}

impl PartialEq<&str> for ElementId {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        self.0.as_str() == *other
    }
}

impl PartialEq<ElementId> for &str {
    #[inline(always)]
    fn eq(&self, other: &ElementId) -> bool {
        *self == other.0.as_str()
    }
}

impl PartialEq<String> for ElementId {
    #[inline(always)]
    fn eq(&self, other: &String) -> bool {
        self.0.as_str() == other.as_str()
    }
}

impl PartialEq<ElementId> for String {
    #[inline(always)]
    fn eq(&self, other: &ElementId) -> bool {
        self.as_str() == other.0.as_str()
    }
}

/// The state of an element.
///
/// Elements can either be active or inactive.
///
/// Active elements have the [ElementActive] component.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Component)]
pub enum ElementState {
    /// The element is active.
    Active,
    /// The element is inactive (hidden).
    Inactive,
}

/// A marker component for active elements.
///
/// This component is added to an element if [ElementState] is `Active`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Component)]
pub struct ElementActive;
