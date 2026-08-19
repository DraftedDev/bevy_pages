use crate::element::ElementProps;
use crate::parser::color::parse_color;
use crate::parser::values::parse_attribute;
use crate::parser::values::{parse_bool, parse_matches, parse_rect};
use crate::props::Properties;
use crate::widgets::Widget;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::math::Rect;
use bevy::prelude::{
    Changed, Entity, EntityCommands, ImageNode, Interaction, NodeImageMode, Or, Query, Res,
    TextureSlicer, VisualBox,
};
use roxmltree::Node;

pub(crate) fn update_props(
    assets: Res<AssetServer>,
    mut query: Query<
        (&Interaction, &Properties<ImageProps>, &mut ImageNode),
        Or<(Changed<Interaction>, Changed<Properties<ImageProps>>)>,
    >,
) {
    for (interaction, props, mut image_node) in &mut query {
        let props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        let image = assets.load(&props.src);

        crate::set_if_changed!(
            image_node.color, props.color;
            image_node.flip_x, props.flip_x;
            image_node.flip_y, props.flip_y;
            image_node.rect, props.rect;
            image_node.image_mode, props.mode => props.mode.clone();
            image_node.visual_box, props.visual_box;
            image_node.image, image;

        );
    }
}

/// An image widget.
///
/// ## XML Usage
///
/// Build a new image widget using the `<Image />` tag.
///
/// ### Attributes
/// - `src = "<string>"`: The asset path to the actual image to display. Required.
/// - `color = "<color>"`: The color/tint of the image.
/// - `flip-x = "<bool>"`: Whether to flip the image horizontally.
/// - `flip-y = "<bool>"`: Whether to flip the image vertically.
/// - `rect = "<rect>"`: The rect to clip the image to.
/// - `mode = "<auto|sliced|tiled|stretch>"`: The layout mode to use for the image.
/// - `visual-box = "<padding|content|border>"`: The visual box of the image.
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// This widget does not have any special code logic.
/// Use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct ImageWidget {
    props: Properties<ImageProps>,
}

impl Widget for ImageWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Image"
    }

    fn parse(&mut self, node: &Node) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = ImageProps::parse(node, None, &ImageProps::default())?;
        let hover = ImageProps::parse(node, Some("hover"), &default)?;
        let click = ImageProps::parse(node, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, assets: &AssetServer) -> Entity {
        let props = &self.props.default;
        let image = ImageNode {
            color: props.color,
            image: assets.load(&props.src),
            texture_atlas: None,
            flip_x: props.flip_x,
            flip_y: props.flip_y,
            rect: props.rect,
            image_mode: props.mode.clone(),
            visual_box: props.visual_box,
        };

        commands.insert(image);

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
        Box::new(self.clone())
    }
}

/// The properties of an [ImageWidget].
#[derive(Clone, Debug)]
pub struct ImageProps {
    /// The source path of the image.
    pub src: String,
    /// The color/tint of the image.
    pub color: Color,
    /// The flip-x attribute.
    pub flip_x: bool,
    /// The flip-y attribute.
    pub flip_y: bool,
    /// The optional rect to use for the image.
    pub rect: Option<Rect>,
    /// The mode to use for the image.
    pub mode: NodeImageMode,
    /// The visual box to use for the image.
    pub visual_box: VisualBox,
}

impl ImageProps {
    fn parse(
        node: &Node,
        prefix: Option<&str>,
        base: &Self,
    ) -> bevy::prelude::Result<Self, String> {
        let src = parse_attribute(node, "src", prefix, |s| Ok(s.to_string()))?
            .unwrap_or_else(|| base.src.clone());

        let color = parse_attribute(node, "color", prefix, parse_color)?.unwrap_or(base.color);

        let flip_x = parse_attribute(node, "flip-x", prefix, parse_bool)?.unwrap_or(base.flip_x);

        let flip_y = parse_attribute(node, "flip-y", prefix, parse_bool)?.unwrap_or(base.flip_y);

        let rect = parse_attribute(node, "rect", prefix, parse_rect)?.or(base.rect);

        let mode = parse_attribute(node, "mode", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("auto", || Ok(NodeImageMode::Auto)),
                    ("sliced", || {
                        Ok(NodeImageMode::Sliced(TextureSlicer::default()))
                    }),
                    ("tiled", || {
                        Ok(NodeImageMode::Tiled {
                            tile_x: true,
                            tile_y: true,
                            stretch_value: 1.0,
                        })
                    }),
                    ("stretch", || Ok(NodeImageMode::Stretch)),
                ],
            )
        })?
        .unwrap_or_else(|| base.mode.clone());

        let visual_box = parse_attribute(node, "visual-box", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("padding", || Ok(VisualBox::PaddingBox)),
                    ("content", || Ok(VisualBox::ContentBox)),
                    ("border", || Ok(VisualBox::BorderBox)),
                ],
            )
        })?
        .unwrap_or(base.visual_box);

        Ok(Self {
            src,
            color,
            flip_x,
            flip_y,
            rect,
            mode,
            visual_box,
        })
    }
}

impl Default for ImageProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            src: "".to_string(),
            color: Color::WHITE,
            flip_x: false,
            flip_y: false,
            rect: None,
            mode: NodeImageMode::default(),
            visual_box: VisualBox::default(),
        }
    }
}
