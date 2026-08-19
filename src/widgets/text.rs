use crate::element::ElementProps;
use crate::parser::values::parse_attribute;
use crate::props::Properties;
use crate::widgets::Widget;
use bevy::asset::{AssetServer, Handle};
use bevy::color::Color;
use bevy::prelude::{
    Changed, Entity, EntityCommands, FontSize, FontSource, FontStyle, FontWeight, FontWidth, Or,
    Query, Res, Text, TextColor, TextFont,
};
use bevy::ui::Interaction;
use roxmltree::Node;

pub(crate) fn update_props(
    assets: Res<AssetServer>,
    mut query: Query<
        (
            &Interaction,
            &Properties<TextProps>,
            &mut Text,
            &mut TextColor,
            &mut TextFont,
        ),
        Or<(Changed<Interaction>, Changed<Properties<TextProps>>)>,
    >,
) {
    for (interaction, props, mut text, mut color, mut font) in &mut query {
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        crate::set_if_changed!(
            text.0, active_props.content => active_props.content.clone();
            color.0, active_props.color;
            font.font_size, active_props.font_size;
            font.weight, active_props.font_weight;
            font.width, active_props.font_width;
            font.style, active_props.font_style;
        );

        if let Some(ref f) = active_props.font {
            let handle: Handle<bevy::text::Font> = assets.load(f);
            let src = FontSource::Handle(handle);

            crate::set_if_changed!(font.font, src);
        }
    }
}

/// A text widget.
///
/// ## XML Usage
///
/// Build a text widget with the `<Text>...</Text>` tag.
/// You can specify the text content either using the inner tag text or the `content` attribute.
///
/// ### Attributes
/// - `content = "<string>"`: Sets the content of the text.
/// - `font = "<string>"`: Sets the font of the text. When unspecified, the default bevy font will be used.
/// - `font-weight = "<int|thin|extra_light|light|normal|medium|semibold|bold|extra_bold|black|extra_black>"`: Sets the font weight.
/// - `font-width = "<float>"`: Sets the font width.
/// - `font-size = "<fontSize>"`: Sets the font size.
/// - `font-style = "<normal|italic|oblique>"`: Sets the font style.
/// - `color = "<color>"`: Sets the text color.
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// Use the [TextProps] to control the text.
/// This widget does not have any special code logic.
///
/// You may use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct TextWidget {
    props: Properties<TextProps>,
}

impl Widget for TextWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Text"
    }

    fn parse(&mut self, node: &Node) -> Result<(), String> {
        let default = TextProps::parse(node, None, &TextProps::default())?;
        let hover = TextProps::parse(node, Some("hover"), &default)?;
        let click = TextProps::parse(node, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, assets: &AssetServer) -> Entity {
        let props = &self.props.default;

        let text = Text::new(&props.content);
        let text_font = TextFont {
            font: props
                .font
                .as_ref()
                .map(|f| FontSource::Handle(assets.load(f)))
                .unwrap_or_default(),
            font_size: props.font_size,
            weight: props.font_weight,
            width: props.font_width,
            style: props.font_style,
            ..Default::default()
        };
        let text_color = TextColor(props.color);

        commands.insert((self.props.clone(), text, text_font, text_color));

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

/// The properties of a [TextWidget].
#[derive(Clone, Debug)]
pub struct TextProps {
    /// The content of the text.
    pub content: String,
    /// An asset path to the font that should be used.
    pub font: Option<String>,
    /// The weight of the font.
    pub font_weight: FontWeight,
    /// The width of the font.
    pub font_width: FontWidth,
    /// The size of the font.
    pub font_size: FontSize,
    /// The style of the font.
    pub font_style: FontStyle,
    /// The color of the text.
    pub color: Color,
}

impl TextProps {
    fn parse(node: &Node, prefix: Option<&str>, base: &Self) -> Result<Self, String> {
        let content = parse_attribute(node, "content", prefix, |s| Ok(s.to_string()))?
            .or_else(|| {
                if prefix.is_none() {
                    node.text().map(str::trim).map(str::to_string)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| base.content.clone());

        let font = parse_attribute(node, "font", prefix, |s| Ok(s.to_string()))?
            .or_else(|| base.font.clone());

        let font_weight = parse_attribute(node, "font-weight", prefix, |s| {
            crate::parser::values::parse_int(s)
                .map(|i| Ok(FontWeight(i as u16)))
                .unwrap_or_else(|_| {
                    crate::parser::values::parse_matches(
                        s,
                        &[
                            ("thin", || Ok(FontWeight::THIN)),
                            ("extra_light", || Ok(FontWeight::EXTRA_LIGHT)),
                            ("light", || Ok(FontWeight::LIGHT)),
                            ("normal", || Ok(FontWeight::NORMAL)),
                            ("medium", || Ok(FontWeight::MEDIUM)),
                            ("semibold", || Ok(FontWeight::SEMIBOLD)),
                            ("bold", || Ok(FontWeight::BOLD)),
                            ("extra_bold", || Ok(FontWeight::EXTRA_BOLD)),
                            ("black", || Ok(FontWeight::BLACK)),
                            ("extra_black", || Ok(FontWeight::EXTRA_BLACK)),
                        ],
                    )
                })
        })?
        .unwrap_or(base.font_weight);

        let font_width = parse_attribute(node, "font-width", prefix, |s| {
            crate::parser::values::parse_float(s).map(FontWidth)
        })?
        .unwrap_or(base.font_width);

        let font_size = parse_attribute(
            node,
            "font-size",
            prefix,
            crate::parser::values::parse_font_size,
        )?
        .unwrap_or(base.font_size);

        let font_style = parse_attribute(node, "font-style", prefix, |s| {
            crate::parser::values::parse_matches(
                s,
                &[
                    ("normal", || Ok(FontStyle::Normal)),
                    ("italic", || Ok(FontStyle::Italic)),
                    ("oblique", || Ok(FontStyle::Oblique(None))),
                ],
            )
        })?
        .unwrap_or(base.font_style);

        let color = parse_attribute(node, "color", prefix, crate::parser::color::parse_color)?
            .unwrap_or(base.color);

        Ok(Self {
            content,
            font,
            font_weight,
            font_width,
            font_size,
            font_style,
            color,
        })
    }
}

impl Default for TextProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            content: String::new(),
            font: None,
            font_weight: FontWeight::NORMAL,
            font_width: FontWidth::NORMAL,
            font_size: FontSize::Px(16.0),
            font_style: FontStyle::Normal,
            color: Color::WHITE,
        }
    }
}
