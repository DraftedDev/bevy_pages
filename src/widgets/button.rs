use crate::element::ElementProps;
use crate::parser::AttributeMap;
use crate::parser::color::{darken_color, lighten_color};
use crate::widgets::Widget;
use bevy::app::App;
use bevy::color::Color;
use bevy::prelude::{Entity, World};
use bevy::ui::{UiRect, Val};
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

    fn parse(&mut self, _: &Node, _: &AttributeMap) -> Result<(), String>
    where
        Self: Sized,
    {
        Ok(())
    }

    fn spawn(&self, entity: Entity, _: &mut World) -> Entity {
        entity
    }

    fn apply_defaults(
        &self,
        attrs: &AttributeMap,
        default: &mut ElementProps,
        hover: &mut ElementProps,
        click: &mut ElementProps,
    ) {
        let base_bg = default
            .bg_color
            .unwrap_or_else(|| Color::srgb(0.2, 0.2, 0.2));

        crate::set_missing_attrs!(
            attrs,

            "bg-color" => default.bg_color = Some(base_bg),
            "hover.bg-color" => hover.bg_color = Some(lighten_color(base_bg, 0.15)),
            "click.bg-color" => click.bg_color = Some(darken_color(base_bg, 0.15)),

            "padding" => default.node.padding = UiRect::axes(Val::Px(20.0), Val::Px(15.0)),
            "hover.padding" => hover.node.padding = default.node.padding,
            "click.padding" => click.node.padding = default.node.padding,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(Self)
    }
}
