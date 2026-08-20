use crate::element::ElementProps;
use crate::parser::color::parse_color;
use crate::parser::values::{parse_attribute, parse_border_rect, parse_float};
use crate::parser::values::{parse_bool, parse_matches, parse_rect};
use crate::props::Properties;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::app::{App, Update};
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::math::Rect;
use bevy::prelude::{
    BorderRect, Changed, Entity, EntityCommands, ImageNode, Interaction, IntoScheduleConfigs,
    NodeImageMode, Or, Query, Res, TextureSlicer, VisualBox,
};
use bevy::sprite::SliceScaleMode;
use bevy::ui::Val;
use roxmltree::Node;

fn update_props(
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
/// - `sliced-border = "<borderRect>"`: The border rect when `mode = "sliced"`.
/// - `sliced-center-scale-stretch = "<float>"`: The center scale when `mode = "sliced"` and `sliced-center-scale = "tile"`.
/// - `sliced-center-scale = "<stretch|tile>"`: The center scale when `mode = "sliced"`.
/// - `sliced-sides-scale-stretch = "<float>"`: The sides scale when `mode = "sliced"` and `sliced-sides-scale = "tile"`.
/// - `sliced-sides-scale = "<stretch|tile>"`: The sides scale when `mode = "sliced"`.
/// - `sliced-max-corner-scale = "<float>"`: The max corner scale when `mode = "sliced"`.
/// - `tiled-x = "<bool>"`: Whether to tile the image horizontally, when `mode = "tiled"`.
/// - `tiled-y = "<bool>"`: Whether to tile the image vertically, when `mode = "tiled"`.
/// - `tiled-stretch = "<float>"`: The stretch scale when `mode = "tiled"`.
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// Use [ImageProps] to control the image widget.
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

    fn setup(&self, app: &mut App) {
        app.add_systems(Update, update_props.in_set(PageSystemSet));
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
        node: &Node,
        default: &mut ElementProps,
        hover: &mut ElementProps,
        click: &mut ElementProps,
    ) {
        crate::set_missing_attrs!(
            node,

            "width" => default.node.width = Val::Px(250.0),
            "hover.width" => hover.node.width = default.node.width,
            "click.width" => click.node.width = default.node.width,

            "height" => default.node.height = Val::Px(250.0),
            "hover.height" => hover.node.height = default.node.height,
            "click.height" => click.node.height = default.node.height,

            "flex-grow" => default.node.flex_grow = 1.0,
            "hover.flex-grow" => hover.node.flex_grow = default.node.flex_grow,
            "click.flex-grow" => click.node.flex_grow = default.node.flex_grow,
        );
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
    /// The [BorderRect] to use for the image when in `mode = "sliced"`.
    ///
    /// Unused when `mode` is not `"sliced"`.
    pub sliced_border: BorderRect,
    /// The center stretch value to use for the image when `mode = "sliced"` and `sliced-center-scale = "tile"`.
    ///
    /// Unused when `mode` is not `"sliced"` and `sliced-center-scale` is not `"tile"`.
    pub sliced_center_scale_stretch: f32,
    /// The center [SliceScaleMode] to use for the image when `mode = "sliced"`.
    ///
    /// Unused when `mode` is not `"sliced"`.
    pub sliced_center_scale: SliceScaleMode,
    /// The sides stretch value to use for the image when `mode = "sliced"` and `sliced-sides-scale = "tile"`.
    ///
    /// Unused when `mode` is not `"sliced"` and `sliced-sides-scale` is not `"tile"`.
    pub sliced_sides_scale_stretch: f32,
    /// The sides [SliceScaleMode] to use for the image when `mode = "sliced"`.
    ///
    /// Unused when `mode` is not `"sliced"`.
    pub sliced_sides_scale: SliceScaleMode,
    /// The max corner scale to use for the image when `mode = "sliced"`.
    ///
    /// Unused when `mode` is not `"sliced"`.
    pub sliced_max_corner_scale: f32,
    /// If the image should repeat on the x-axis when in `mode = "tiled"`.
    ///
    /// Unused when `mode` is not `"tiled"`.
    pub tiled_x: bool,
    /// If the image should repeat on the y-axis when in `mode = "tiled"`.
    ///
    /// Unused when `mode` is not `"tiled"`.
    pub tiled_y: bool,
    /// The stretch value to use for the image when `mode = "tiled"`.
    ///
    /// Unused when `mode` is not `"tiled"`.
    pub tiled_stretch: f32,
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

        let sliced_border = parse_attribute(node, "sliced-border", prefix, parse_border_rect)?
            .unwrap_or(base.sliced_border);

        let sliced_center_scale_stretch =
            parse_attribute(node, "sliced-center-scale-stretch", prefix, parse_float)?
                .unwrap_or(base.sliced_center_scale_stretch);

        let sliced_center_scale = parse_attribute(node, "sliced-center-scale", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("stretch", &|| Ok(SliceScaleMode::Stretch)),
                    ("tile", &|| {
                        Ok(SliceScaleMode::Tile {
                            stretch_value: sliced_center_scale_stretch,
                        })
                    }),
                ],
            )
        })?
        .unwrap_or(base.sliced_center_scale);

        let sliced_sides_scale_stretch =
            parse_attribute(node, "sliced-sides-scale-stretch", prefix, parse_float)?
                .unwrap_or(base.sliced_sides_scale_stretch);

        let sliced_sides_scale = parse_attribute(node, "sliced-sides-scale", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("stretch", &|| Ok(SliceScaleMode::Stretch)),
                    ("tile", &|| {
                        Ok(SliceScaleMode::Tile {
                            stretch_value: sliced_sides_scale_stretch,
                        })
                    }),
                ],
            )
        })?
        .unwrap_or(base.sliced_sides_scale);

        let sliced_max_corner_scale =
            parse_attribute(node, "sliced-max-corner-scale", prefix, parse_float)?
                .unwrap_or(base.sliced_max_corner_scale);

        let tiled_x = parse_attribute(node, "tiled-x", prefix, parse_bool)?.unwrap_or(base.tiled_x);

        let tiled_y = parse_attribute(node, "tiled-y", prefix, parse_bool)?.unwrap_or(base.tiled_y);

        let tiled_stretch = parse_attribute(node, "tiled-stretch", prefix, parse_float)?
            .unwrap_or(base.tiled_stretch);

        let mode = parse_attribute(node, "mode", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("auto", &|| Ok(NodeImageMode::Auto)),
                    ("sliced", &|| {
                        Ok(NodeImageMode::Sliced(TextureSlicer::default()))
                    }),
                    ("tiled", &|| {
                        Ok(NodeImageMode::Tiled {
                            tile_x: tiled_x,
                            tile_y: tiled_y,
                            stretch_value: tiled_stretch,
                        })
                    }),
                    ("stretch", &|| Ok(NodeImageMode::Stretch)),
                ],
            )
        })?
        .unwrap_or_else(|| base.mode.clone());

        let visual_box = parse_attribute(node, "visual-box", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("padding", &|| Ok(VisualBox::PaddingBox)),
                    ("content", &|| Ok(VisualBox::ContentBox)),
                    ("border", &|| Ok(VisualBox::BorderBox)),
                ],
            )
        })?
        .unwrap_or(base.visual_box);

        let mut props = Self {
            src,
            color,
            flip_x,
            flip_y,
            rect,
            mode,
            visual_box,
            sliced_border,
            sliced_center_scale_stretch,
            sliced_center_scale,
            sliced_sides_scale_stretch,
            sliced_sides_scale,
            sliced_max_corner_scale,
            tiled_x,
            tiled_y,
            tiled_stretch,
        };

        props.mode = props.compute_mode();

        Ok(props)
    }

    /// Constructs the final [NodeImageMode] dynamically from all individual slice and tile attributes.
    pub fn compute_mode(&self) -> NodeImageMode {
        match &self.mode {
            NodeImageMode::Auto => NodeImageMode::Auto,
            NodeImageMode::Stretch => NodeImageMode::Stretch,
            NodeImageMode::Sliced(_) => {
                let center_scale_mode = match self.sliced_center_scale {
                    SliceScaleMode::Stretch => SliceScaleMode::Stretch,
                    SliceScaleMode::Tile { .. } => SliceScaleMode::Tile {
                        stretch_value: self.sliced_center_scale_stretch,
                    },
                };

                let sides_scale_mode = match self.sliced_sides_scale {
                    SliceScaleMode::Stretch => SliceScaleMode::Stretch,
                    SliceScaleMode::Tile { .. } => SliceScaleMode::Tile {
                        stretch_value: self.sliced_sides_scale_stretch,
                    },
                };

                NodeImageMode::Sliced(TextureSlicer {
                    border: self.sliced_border,
                    center_scale_mode,
                    sides_scale_mode,
                    max_corner_scale: self.sliced_max_corner_scale,
                })
            }
            NodeImageMode::Tiled { .. } => NodeImageMode::Tiled {
                tile_x: self.tiled_x,
                tile_y: self.tiled_y,
                stretch_value: self.tiled_stretch,
            },
        }
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
            sliced_border: BorderRect::default(),
            sliced_center_scale_stretch: 1.0,
            sliced_center_scale: SliceScaleMode::default(),
            sliced_sides_scale_stretch: 1.0,
            sliced_sides_scale: SliceScaleMode::default(),
            sliced_max_corner_scale: 1.0,
            tiled_x: true,
            tiled_y: true,
            tiled_stretch: 1.0,
        }
    }
}
