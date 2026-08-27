use crate::element::ElementProps;
use crate::parser::AttributeMap;
use bevy::app::App;
use bevy::prelude::{Entity, World};
use roxmltree::Node;
use std::fmt::Debug;

/// Contains the checkbox widget and functionality.
pub mod checkbox;

/// Contains the switch widget and functionality.
pub mod switch;

/// Contains the node widget and functionality.
pub mod node;

/// Contains the button widget and functionality.
pub mod button;

/// Contains the text widget and functionality.
pub mod text;

/// Contains the image widget and functionality.
pub mod image;

/// Contains the text input widget and functionality.
pub mod text_input;

/// Contains the slider widget and functionality.
pub mod slider;

/// Contains the progress bar widget and functionality.
pub mod progress_bar;

/// Contains the tooltip widget and functionality.
pub mod tooltip;

/// Contains the divider widget and functionality.
pub mod divider;

/// Contains the scroll view widget and functionality.
pub mod scroll_view;

/// Contains the dropdown widget and functionality.
pub mod dropdown;

/// A trait to define a widget.
pub trait Widget: Debug + Send + Sync + 'static {
    /// The name of the widget.
    ///
    /// This should also be its XML tag name.
    fn name() -> &'static str
    where
        Self: Sized;

    /// Set up the widget logic.
    ///
    /// This is where widgets should add systems and observers to the bevy app.
    fn setup(&self, app: &mut App);

    /// Parses the widget from an XML node and an [AttributeMap].
    fn parse(&mut self, node: &Node, attrs: &AttributeMap) -> Result<(), String>;

    /// Spawns the widget. Called inside [Element::spawn](crate::element::Element::spawn).
    fn spawn(&self, entity: Entity, world: &mut World) -> Entity;

    /// Apply this widget's default properties.
    fn apply_defaults(
        &self,
        attrs: &AttributeMap,
        default: &mut ElementProps,
        hover: &mut ElementProps,
        click: &mut ElementProps,
    );

    /// Creates a dynamic clone of the widget.
    fn dyn_clone(&self) -> Box<dyn Widget>;
}

impl Clone for Box<dyn Widget> {
    #[inline(always)]
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}
