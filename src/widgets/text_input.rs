use crate::element::{ElementActive, ElementId, ElementProps};
use crate::events::ElementSet;
use crate::parser::AttributeMap;
use crate::parser::color::{darken_color, lighten_color, parse_color};
use crate::parser::values::{parse_attribute, parse_bool, parse_float, parse_font_size};
use crate::props::Properties;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::input_focus::InputFocus;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::{AlignItems, BorderColor, Display, JustifyContent, UiRect, Val};
use roxmltree::Node as XmlNode;

fn sync_text_changes(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &EditableText,
            &mut TextInputState,
            &mut Properties<TextInputProps>,
            Option<&ElementId>,
        ),
        (With<ElementActive>, Changed<EditableText>),
    >,
) {
    for (entity, editable, mut state, mut props, id) in &mut query {
        let current_val = editable.value().to_string();

        if state.0 != current_val {
            state.0 = current_val.clone();
            props.mutate(|p| p.value = state.0.clone());

            commands.trigger(ElementSet {
                entity,
                id: id.cloned(),
                value: current_val,
                delta: None,
            });
        }
    }
}

fn unfocus_on_outside_click(
    trigger: On<Pointer<Press>>,
    mut focus: ResMut<InputFocus>,
    editable_query: Query<(), With<EditableText>>,
) {
    if !editable_query.contains(trigger.entity) {
        focus.clear();
    }
}

fn update_props(
    mut query: Query<
        (
            &Interaction,
            &Hovered,
            &Properties<TextInputProps>,
            &mut TextFont,
            &mut TextColor,
            &mut EditableText,
        ),
        (
            With<ElementActive>,
            Or<(
                Changed<Interaction>,
                Changed<Hovered>,
                Changed<Properties<TextInputProps>>,
                Added<ElementActive>,
            )>,
        ),
    >,
    assets: Res<AssetServer>,
) {
    for (interaction, hovered, props, mut font, mut text_color, mut edits) in &mut query {
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            _ if hovered.0 => &props.hover,
            _ => &props.default,
        };

        crate::set_if_changed!(
            font.font_size, active_props.font_size;
            text_color.0, active_props.color;
            edits.allow_newlines, active_props.allow_newlines;
            edits.visible_width, active_props.visible_width;
        );

        if let Some(font_name) = &active_props.font {
            let src = FontSource::Handle(assets.load(font_name));

            crate::set_if_changed!(font.font, src);
        }
    }
}

/// The internal state of the text input.
#[derive(Clone, Debug, Default, Component)]
pub struct TextInputState(pub String);

/// A text input widget that enables typed user input.
///
/// ## XML Usage
///
/// Build a new text input widget using the `<TextInput/>` tag.
///
/// ### Attributes
/// - `value = "<string>"`: The text input value.
/// - `font-size = "<fontSize>"`: The text font size.
/// - `font = "<string>"`: The font of the text. When unspecified, the default bevy font will be used.
/// - `visible-width = "<float>"`: The optional maximum width of visible text inside the text box.
/// - `allow-newlines = "<bool>"`: If the text input should allow new lines.
/// - `color = "<color>"`: The text color.
///
/// All the attributes listed, except `value`, support state overrides.
///
/// ## Logic
///
/// Use [TextInputProps] to control the text input widget.
/// Furthermore, the widget emits `ElementSet<String>` events when text is typed.
///
/// You can also use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct TextInputWidget {
    props: Properties<TextInputProps>,
}

impl Widget for TextInputWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "TextInput"
    }

    fn setup(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_props, sync_text_changes).in_set(PageSystemSet),
        )
        .add_observer(unfocus_on_outside_click);
    }

    fn parse(&mut self, _: &XmlNode, attrs: &AttributeMap) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = TextInputProps::parse(attrs, None, &TextInputProps::default())?;
        let hover = TextInputProps::parse(attrs, Some("hover"), &default)?;
        let click = TextInputProps::parse(attrs, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, entity: Entity, world: &mut World) -> Entity {
        let assets = world.resource::<AssetServer>();
        let props = &self.props.default;

        let font_handle = props
            .font
            .as_ref()
            .map(|path| assets.load(path))
            .unwrap_or_default();

        world.entity_mut(entity).insert((
            self.props.clone(),
            TextInputState(props.value.clone()),
            EditableText {
                visible_width: props.visible_width,
                allow_newlines: props.allow_newlines,
                ..default()
            },
            TextLayout::no_wrap(),
            TextFont {
                font: FontSource::Handle(font_handle),
                font_size: props.font_size,
                ..default()
            },
            TextColor(props.color),
            TextCursorStyle::default(),
            Hovered::default(),
        ));

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
            .unwrap_or_else(|| Color::srgb(0.12, 0.12, 0.14));

        crate::set_missing_attrs!(
            attrs,

            "width" => default.node.width = Val::Px(200.0),
            "hover.width" => hover.node.width = default.node.width,
            "click.width" => click.node.width = default.node.width,

            "height" => default.node.height = Val::Px(32.0),
            "hover.height" => hover.node.height = default.node.height,
            "click.height" => click.node.height = default.node.height,

            "display" => default.node.display = Display::Flex,
            "hover.display" => hover.node.display = default.node.display,
            "click.display" => click.node.display = default.node.display,

            "align-items" => default.node.align_items = AlignItems::Center,
            "hover.align-items" => hover.node.align_items = default.node.align_items,
            "click.align-items" => click.node.align_items = default.node.align_items,

            "justify-content" => default.node.justify_content = JustifyContent::FlexStart,
            "hover.justify-content" => hover.node.justify_content = default.node.justify_content,
            "click.justify-content" => click.node.justify_content = default.node.justify_content,

            "padding" => default.node.padding = UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            "hover.padding" => hover.node.padding = default.node.padding,
            "click.padding" => click.node.padding = default.node.padding,

            "bg-color" => default.bg_color = Some(base_bg),
            "hover.bg-color" => hover.bg_color = Some(lighten_color(base_bg, 0.05)),
            "click.bg-color" => click.bg_color = Some(darken_color(base_bg, 0.05)),

            "border-color" => default.border_color = Some(BorderColor::all(Color::srgb(0.4, 0.4, 0.45))),
            "hover.border-color" => hover.border_color = Some(BorderColor::all(Color::srgb(0.6, 0.6, 0.7))),
            "click.border-color" => click.border_color = Some(BorderColor::all(Color::srgb(0.38, 0.69, 0.94))),

            "border" => default.node.border = UiRect::all(Val::Px(1.5)),
            "hover.border" => hover.node.border = default.node.border,
            "click.border" => click.node.border = default.node.border,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties of a [TextInputWidget].
#[derive(Clone, Debug)]
pub struct TextInputProps {
    /// Initial value of the text field.
    pub value: String,
    /// The font size.
    pub font_size: FontSize,
    /// Optional asset path for a custom font.
    pub font: Option<String>,
    /// Optional limit on visible character width.
    pub visible_width: Option<f32>,
    /// Controls if newline characters are allowed.
    pub allow_newlines: bool,
    /// Color of the inner text.
    pub color: Color,
}

impl TextInputProps {
    fn parse(
        attrs: &AttributeMap,
        prefix: Option<&str>,
        base: &Self,
    ) -> std::result::Result<Self, String> {
        let value = parse_attribute(attrs, "value", prefix, |s| Ok(s.to_string()))?
            .unwrap_or_else(|| base.value.clone());

        let font_size =
            parse_attribute(attrs, "font-size", prefix, parse_font_size)?.unwrap_or(base.font_size);

        let font =
            parse_attribute(attrs, "font", prefix, |s| Ok(s.to_string()))?.or(base.font.clone());

        let visible_width =
            parse_attribute(attrs, "visible-width", prefix, parse_float)?.or(base.visible_width);

        let allow_newlines = parse_attribute(attrs, "allow-newlines", prefix, parse_bool)?
            .unwrap_or(base.allow_newlines);

        let color = parse_attribute(attrs, "color", prefix, parse_color)?.unwrap_or(base.color);

        Ok(Self {
            value,
            font_size,
            font,
            visible_width,
            allow_newlines,
            color,
        })
    }
}

impl Default for TextInputProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            value: String::new(),
            font_size: FontSize::Px(16.0),
            font: None,
            visible_width: None,
            allow_newlines: false,
            color: Color::WHITE,
        }
    }
}
