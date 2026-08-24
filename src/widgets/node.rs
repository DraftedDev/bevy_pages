use crate::element::ElementProps;
use crate::parser::AttributeMap;
use crate::widgets::Widget;
use bevy::app::App;
use bevy::prelude::{Entity, World};
use roxmltree::Node;

/// A node widget. Equivalent to `<div></div>` in HTML.
///
/// ## XML Usage
///
/// Build a node widget using the `<Node></Node>` tag.
///
/// The node widget does not introduce any new attributes.
///
/// ## Logic
///
/// This widget does not have any special code logic.
/// Use element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct NodeWidget;

impl Widget for NodeWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Node"
    }

    fn setup(&self, _: &mut App) {}

    fn parse(&mut self, _: &Node, _: AttributeMap) -> Result<(), String> {
        Ok(())
    }

    fn spawn(&self, entity: Entity, _: &mut World) -> Entity {
        entity
    }

    fn apply_defaults(
        &self,
        _: &Node,
        _: &mut ElementProps,
        _: &mut ElementProps,
        _: &mut ElementProps,
    ) {
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(Self)
    }
}
