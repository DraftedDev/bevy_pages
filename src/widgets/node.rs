use crate::element::ElementProps;
use crate::widgets::Widget;
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::prelude::{Entity, EntityCommands};
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

    fn parse(&mut self, _: &Node) -> Result<(), String> {
        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, _: &AssetServer) -> Entity {
        commands.id()
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
