use crate::element::ElementProps;
use crate::parser::values::{parse_attribute, parse_matches};
use crate::props::Properties;
use crate::widgets::Widget;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::prelude::*;
use roxmltree::Node as XmlNode;

/// The layout orientation of the divider line.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum DividerOrientation {
    /// A horizontal line across the container.
    #[default]
    Horizontal,
    /// A vertical line down the container.
    Vertical,
}

impl DividerOrientation {
    /// Parses the divider orientation from a string.
    pub fn parse(s: &str) -> Result<Self, String> {
        parse_matches(
            s,
            &[
                ("horizontal", || Ok(Self::Horizontal)),
                ("vertical", || Ok(Self::Vertical)),
            ],
        )
    }
}

/// A widget to show a divider line.
///
/// ## XML Usage
///
/// Build a divider widget using the `<Divider />` tag.
///
/// ### Attributes
/// - `orientation = "<horizontal|vertical>"`: The orientation of the line.
///
/// The attributes listed do not support any state overrides.
///
/// ## Logic
///
/// This widget does not have any special code logic.
/// Use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct DividerWidget {
    props: Properties<DividerProps>,
}

impl Widget for DividerWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Divider"
    }

    fn parse(&mut self, node: &XmlNode) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = DividerProps::parse(node, None, &DividerProps::default())?;
        let hover = DividerProps::parse(node, Some("hover"), &default)?;
        let click = DividerProps::parse(node, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, _: &AssetServer) -> Entity {
        commands.insert(self.props.clone());

        commands.id()
    }

    fn apply_defaults(
        &self,
        node: &XmlNode,
        default: &mut ElementProps,
        hover: &mut ElementProps,
        click: &mut ElementProps,
    ) {
        let is_vertical = node
            .attribute("orientation")
            .map(|s| s.trim() == "vertical")
            .unwrap_or(false);

        let default_color = Color::srgb(0.25, 0.25, 0.28);

        let width = if is_vertical {
            Val::Px(1.0)
        } else {
            Val::Percent(100.0)
        };

        let height = if is_vertical {
            Val::Percent(100.0)
        } else {
            Val::Px(1.0)
        };

        let margin = if is_vertical {
            UiRect::axes(Val::Px(8.0), Val::Px(0.0))
        } else {
            UiRect::axes(Val::Px(0.0), Val::Px(8.0))
        };

        crate::set_missing_attrs!(
            node,

            "width" => default.node.width = width,
            "hover.width" => hover.node.width = default.node.width,
            "click.width" => click.node.width = default.node.width,

            "height" => default.node.height = height,
            "hover.height" => hover.node.height = default.node.height,
            "click.height" => click.node.height = default.node.height,

            "margin" => default.node.margin = margin,
            "hover.margin" => hover.node.margin = default.node.margin,
            "click.margin" => click.node.margin = default.node.margin,

            "bg-color" => default.bg_color = Some(default_color),
            "hover.bg-color" => hover.bg_color = Some(default_color),
            "click.bg-color" => click.bg_color = Some(default_color),
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties of a [DividerWidget].
#[derive(Clone, Debug, Default)]
pub struct DividerProps {
    /// Orientation of the line.
    pub orientation: DividerOrientation,
}

impl DividerProps {
    fn parse(node: &XmlNode, prefix: Option<&str>, base: &Self) -> Result<Self, String> {
        let orientation = parse_attribute(node, "orientation", prefix, DividerOrientation::parse)?
            .unwrap_or(base.orientation);

        Ok(Self { orientation })
    }
}
