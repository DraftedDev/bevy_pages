use crate::element::ElementProps;
use crate::parser::color::{darken_color, lighten_color};
use crate::parser::values::parse_ui_rect;
use crate::widgets::Widget;
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::prelude::{Entity, EntityCommands};
use roxmltree::Node;

/// A simple button widget.
///
/// ## XML Usage
///
/// Build a button widget using the `<Button></Button>` tag.
///
/// This widget does not spawn new entities and does not have any new attributes.
///
/// The only difference between this and the normal node widget,
/// is that the button will apply different defaults to itself.
///
/// It's really just a `<Node></Node>` with default `click.<...>` and `hover.<...>` attributes.
///
/// ## Logic
///
/// This widget does not have any special code logic.
/// Use the different element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct ButtonWidget;

impl Widget for ButtonWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Button"
    }

    fn setup(&self, _: &mut App) {}

    fn parse(&mut self, _: &Node) -> Result<(), String>
    where
        Self: Sized,
    {
        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, _: &AssetServer) -> Entity {
        commands.id()
    }

    fn apply_defaults(
        &self,
        node: &Node,
        default: &mut ElementProps,
        hover: &mut ElementProps,
        click: &mut ElementProps,
    ) {
        let base_bg = default
            .bg_color
            .unwrap_or_else(|| Color::srgb(0.2, 0.2, 0.2));

        if !node.has_attribute("padding") {
            default.node.padding = parse_ui_rect("15px 20px").unwrap();
        }

        if !node.has_attribute("hover.padding") {
            hover.node.padding = default.node.padding;
        }

        if node.has_attribute("click.padding") {
            click.node.padding = default.node.padding;
        }

        if default.bg_color.is_none() {
            default.bg_color = Some(base_bg);
        }

        if !node.has_attribute("hover.bg-color") {
            hover.bg_color = Some(lighten_color(base_bg, 0.15));
        }

        if !node.has_attribute("click.bg-color") {
            click.bg_color = Some(darken_color(base_bg, 0.15));
        }
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(Self)
    }
}
