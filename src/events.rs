use crate::element::ElementId;
use bevy::prelude::{Entity, Event};
use std::fmt::{Debug, Formatter};
use std::hash::Hash;

/// An event triggered when an element is clicked.
///
/// This event is triggered by a universal interaction system
/// and is therefore not needed to be manually triggered by widget logic.
#[derive(Clone, PartialEq, Eq, Debug, Hash, Event)]
pub struct ElementClick {
    /// The entity behind the target element.
    pub entity: Entity,
    /// The element ID of the target element.
    pub id: Option<ElementId>,
}

impl ElementClick {
    /// Returns [true] if the target element ID matches the given ID.
    #[inline(always)]
    pub fn matches_id(&self, id: impl Into<ElementId>) -> bool {
        self.id.as_ref().map(|i| *i == id.into()).unwrap_or(false)
    }
}

/// An event triggered when the mouse hovers over an element.
///
/// This event is triggered by a universal interaction system
/// and is therefore not needed to be manually triggered by widget logic.
#[derive(Clone, PartialEq, Eq, Debug, Hash, Event)]
pub struct ElementHover {
    /// The entity behind the target element.
    pub entity: Entity,
    /// The element ID of the target element.
    pub id: Option<ElementId>,
}

impl ElementHover {
    /// Returns [true] if the target element ID matches the given ID.
    #[inline(always)]
    pub fn matches_id(&self, id: impl Into<ElementId>) -> bool {
        self.id.as_ref().map(|i| *i == id.into()).unwrap_or(false)
    }
}

/// An event triggered when an element is spawned.
///
/// This event is triggered by the page spawner
/// and is therefore not needed to be manually triggered by widget logic.
#[derive(Clone, PartialEq, Eq, Debug, Hash, Event)]
pub struct ElementSpawn {
    /// The entity behind the target element.
    pub entity: Entity,
    /// The element ID of the target element.
    pub id: Option<ElementId>,
}

impl ElementSpawn {
    /// Returns [true] if the target element ID matches the given ID.
    #[inline(always)]
    pub fn matches_id(&self, id: impl Into<ElementId>) -> bool {
        self.id.as_ref().map(|i| *i == id.into()).unwrap_or(false)
    }
}

/// An event triggered when an element is toggled.
///
/// This is used by the checkbox widget for example.
///
/// It's also used in favor of [ElementSet] when possible.
#[derive(Clone, PartialEq, Eq, Debug, Hash, Event)]
pub struct ElementToggle {
    /// The entity behind the target element.
    pub entity: Entity,
    /// The element ID of the target element.
    pub id: Option<ElementId>,
    /// The new state of the element.
    pub state: bool,
}

impl ElementToggle {
    /// Returns [true] if the target element ID matches the given ID.
    #[inline(always)]
    pub fn matches_id(&self, id: impl Into<ElementId>) -> bool {
        self.id.as_ref().map(|i| *i == id.into()).unwrap_or(false)
    }
}

/// An event triggered when an element's value is set.
///
/// This is used by the text input and slider widgets for example.
#[derive(Event)]
pub struct ElementSet<T> {
    /// The entity behind the target element.
    pub entity: Entity,
    /// The element ID of the target element.
    pub id: Option<ElementId>,
    /// The new value of the element.
    pub value: T,
    /// The difference between the old and the new value (computed as `new - old`).
    ///
    /// This is only set for differentiable types like numbers.
    pub delta: Option<T>,
}

impl<T> ElementSet<T> {
    /// Returns [true] if the target element ID matches the given ID.
    #[inline(always)]
    pub fn matches_id(&self, id: impl Into<ElementId>) -> bool {
        self.id.as_ref().map(|i| *i == id.into()).unwrap_or(false)
    }
}

impl<T: Clone> Clone for ElementSet<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            id: self.id.clone(),
            value: self.value.clone(),
            delta: self.delta.clone(),
        }
    }
}

impl<T: Debug> Debug for ElementSet<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElementSet")
            .field("entity", &self.entity)
            .field("id", &self.id)
            .field("value", &self.value)
            .field("delta", &self.delta)
            .finish()
    }
}

impl<T: Hash> Hash for ElementSet<T> {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.entity.hash(state);
        self.id.hash(state);
        self.value.hash(state);
        self.delta.hash(state);
    }
}

impl<T: PartialEq> PartialEq for ElementSet<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
            && self.id == other.id
            && self.value == other.value
            && self.delta == other.delta
    }
}

impl<T: Eq> Eq for ElementSet<T> {}
