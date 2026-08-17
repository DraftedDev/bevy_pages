use bevy::prelude::Component;
use std::fmt::Debug;

/// A generic container for element and widget properties.
///
/// Every possible state has a corresponding property to use.
///
/// Possible states are:
/// - `default`: The default/fallback state.
/// - `hover`: The state when the mouse is hovering over the element.
/// - `click`: The state when the element is clicked.
#[derive(Clone, Debug, Default, Component)]
pub struct Properties<T: Clone + Debug + Default> {
    /// The default properties.
    ///
    /// **NOTE**: If you generally want to mutate properties, use [mutate](Properties::mutate).
    pub default: T,

    /// The properties when the element is hovered on.
    ///
    /// **NOTE**: If you generally want to mutate properties, use [mutate](Properties::mutate).
    pub hover: T,

    /// The properties when the element is clicked.
    ///
    /// **NOTE**: If you generally want to mutate properties, use [mutate](Properties::mutate).
    pub click: T,
}

impl<T: Clone + Debug + Default> Properties<T> {
    /// Mutates the properties for all states by calling the given function for each state.
    ///
    /// You should mostly use this method for mutating properties.
    pub fn mutate(&mut self, mut f: impl FnMut(&mut T)) {
        f(&mut self.default);
        f(&mut self.hover);
        f(&mut self.click);
    }
}
