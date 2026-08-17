use crate::element::{ElementId, ElementProps};
use crate::events::{ElementClick, ElementSet};
use crate::parser::color::{lighten_color, parse_color};
use crate::parser::values::parse_attribute;
use crate::parser::values::parse_float;
use crate::props::Properties;
use crate::widgets::Widget;
use bevy::color::Color;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use roxmltree::Node as XmlNode;

pub(crate) fn handle_focus(
    trigger: On<ElementClick>,
    mut all_inputs: Query<(Entity, &mut TextInputState, Option<&ElementId>)>,
) {
    let clicked_entity = trigger.entity;

    for (entity, mut state, _id) in all_inputs.iter_mut() {
        if entity == clicked_entity {
            if !state.is_focused {
                state.is_focused = true;
            }
        } else if state.is_focused {
            state.is_focused = false;
        }
    }
}

pub(crate) fn handle_click_outside(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    interaction_query: Query<&Interaction, With<TextInputState>>,
    mut all_inputs: Query<(Entity, &mut TextInputState, Option<&ElementId>)>,
) {
    let just_pressed = mouse_button.just_pressed(MouseButton::Left)
        || touches.iter_just_pressed().next().is_some();

    if !just_pressed {
        return;
    }

    let clicking_input = interaction_query.iter().any(|i| *i == Interaction::Pressed);

    if !clicking_input {
        for (entity, mut state, id) in all_inputs.iter_mut() {
            if state.is_focused {
                state.is_focused = false;
                commands.trigger(ElementSet {
                    entity,
                    id: id.cloned(),
                    value: state.value.clone(),
                    delta: None,
                });
            }
        }
    }
}

pub(crate) fn handle_typing(
    mut commands: Commands,
    mut key_events: MessageReader<KeyboardInput>,
    mut inputs: Query<(Entity, &mut TextInputState, Option<&ElementId>)>,
) {
    for event in key_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        for (entity, mut state, id) in inputs.iter_mut() {
            if !state.is_focused {
                continue;
            }

            match &event.logical_key {
                Key::Backspace => {
                    state.value.pop();
                }

                Key::Enter => {
                    state.is_focused = false;
                    commands.trigger(ElementSet {
                        entity,
                        id: id.cloned(),
                        value: state.value.clone(),
                        delta: None,
                    });
                }

                Key::Character(input_str) => {
                    // Ignore control key sequences
                    if !input_str.chars().any(|c| c.is_control()) {
                        state.value.push_str(input_str.as_str());
                    }
                }
                Key::Space => {
                    state.value.push(' ');
                }

                _ => {}
            }
        }
    }
}

pub(crate) fn sync_visuals(
    query: Query<
        (
            Entity,
            &TextInputState,
            &Properties<TextInputProps>,
            &Children,
        ),
        Or<(Changed<TextInputState>, Changed<Interaction>)>,
    >,
    mut border_query: Query<&mut BorderColor>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<TextInputText>>,
    mut cursor_query: Query<&mut Node, With<TextInputCursor>>,
) {
    for (entity, state, props, children) in &query {
        if let Ok(mut border) = border_query.get_mut(entity)
            && state.is_focused
        {
            *border = BorderColor::all(Color::srgb(0.3, 0.6, 1.0));
        }

        for child in children.iter() {
            if let Ok((mut text, mut color)) = text_query.get_mut(child) {
                if state.value.is_empty() {
                    text.0 = state.placeholder.clone();
                    color.0 = props.default.placeholder_color;
                } else {
                    text.0 = state.value.clone();
                    color.0 = props.default.text_color;
                }
            }

            // Update Cursor display state
            if let Ok(mut cursor_node) = cursor_query.get_mut(child) {
                cursor_node.display = if state.is_focused {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}

pub(crate) fn update_props(
    mut input_query: Query<
        (
            Entity,
            &Interaction,
            &Properties<TextInputProps>,
            &TextInputState,
            &Children,
        ),
        Or<(
            Changed<Interaction>,
            Changed<Properties<TextInputProps>>,
            Changed<TextInputState>,
        )>,
    >,
    mut text_query: Query<
        (&mut Text, &mut TextFont, &mut TextColor),
        (With<TextInputText>, Without<TextInputCursor>),
    >,
    mut cursor_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<TextInputCursor>, Without<TextInputText>),
    >,
) {
    for (_entity, interaction, props, state, children) in &mut input_query {
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        for child in children.iter() {
            if let Ok((mut text, mut font, mut color)) = text_query.get_mut(child) {
                crate::set_if_changed!(
                    font.font_size, FontSize::Px(active_props.font_size);
                );

                let (state_text, state_color) = if state.value.is_empty() {
                    (state.placeholder.clone(), active_props.placeholder_color)
                } else {
                    (state.value.clone(), active_props.text_color)
                };

                crate::set_if_changed!(
                    text.0, state_text;
                    color.0, state_color;
                );
            }

            if let Ok((mut cursor_node, mut cursor_bg)) = cursor_query.get_mut(child) {
                crate::set_if_changed!(
                    cursor_node.height, Val::Px(active_props.font_size);
                    cursor_bg.0, active_props.text_color;
                );
            }
        }
    }
}

/// The internal value and focus state of the text input widget.
#[derive(Component, Debug, Clone)]
pub struct TextInputState {
    /// The current value of the text input.
    pub value: String,
    /// If the widget is currently focused.
    pub is_focused: bool,
    /// The placeholder text to display when the text input is empty.
    pub placeholder: String,
}

/// Marker component for the text input text.
#[derive(Component, Debug, Clone, Copy)]
pub struct TextInputText;

/// Marker component for the text input cursor.
#[derive(Component, Debug, Clone, Copy)]
pub struct TextInputCursor;

/// A text input widget allowing user text input.
///
/// ## XML Usage
///
/// Build a text input widget using the `<TextInput />` tag.
///
/// ### Attributes
/// - `value = "<string>"`: The value of the text input.
/// - `placeholder = "<string>"`: The placeholder text to display when the text input is empty.
/// - `text-color = "<color>"`: The text color.
/// - `placeholder-color = "<color>"`: The placeholder color.
/// - `font-size = "<float>"`: The font size of the text in pixels.
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// Use the [TextInputProps] to control the text input.
/// The widget also emits `ElementSet<String>` events.
///
/// You may also use generic element events to implement custom behavior.
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

    fn parse(&mut self, node: &XmlNode) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = TextInputProps::parse(node, None, &TextInputProps::default())?;
        let hover = TextInputProps::parse(node, Some("hover"), &default)?;
        let click = TextInputProps::parse(node, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, _: &AssetServer) -> Entity {
        let props = &self.props.default;

        let display_text = if props.value.is_empty() {
            props.placeholder.clone()
        } else {
            props.value.clone()
        };

        let active_color = if props.value.is_empty() {
            props.placeholder_color
        } else {
            props.text_color
        };

        commands.insert((
            self.props.clone(),
            TextInputState {
                value: props.value.clone(),
                is_focused: false,
                placeholder: props.placeholder.clone(),
            },
            FocusPolicy::Block,
        ));

        commands.with_children(|parent| {
            parent.spawn((
                TextInputText,
                Text::new(display_text),
                TextFont {
                    font_size: FontSize::Px(props.font_size),
                    ..Default::default()
                },
                TextColor(active_color),
                Node {
                    align_self: AlignSelf::Center,
                    ..Default::default()
                },
                FocusPolicy::Pass,
            ));

            parent.spawn((
                TextInputCursor,
                Node {
                    width: Val::Px(2.0),
                    height: Val::Px(props.font_size),
                    margin: UiRect::left(Val::Px(2.0)),
                    display: Display::None,
                    align_self: AlignSelf::Center,
                    ..Default::default()
                },
                BackgroundColor(props.text_color),
                FocusPolicy::Pass,
            ));
        });

        commands.id()
    }

    fn apply_defaults(
        &self,
        node: &XmlNode,
        default: &mut ElementProps,
        hover: &mut ElementProps,
        click: &mut ElementProps,
    ) {
        let base_bg = default
            .bg_color
            .unwrap_or_else(|| Color::srgb(0.12, 0.12, 0.14));

        crate::set_missing_attrs!(
            node,

            "width" => default.node.width = Val::Px(160.0),
            "hover.width" => hover.node.width = default.node.width,
            "click.width" => click.node.width = default.node.width,

            "height" => default.node.height = Val::Px(32.0),
            "hover.height" => hover.node.height = default.node.height,
            "click.height" => click.node.height = default.node.height,

            "padding" => default.node.padding = UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            "hover.padding" => hover.node.padding = default.node.padding,
            "click.padding" => click.node.padding = default.node.padding,

            "align-items" => default.node.align_items = AlignItems::Center,
            "hover.align-items" => hover.node.align_items = default.node.align_items,
            "click.align-items" => click.node.align_items = default.node.align_items,

            "justify-content" => default.node.justify_content = JustifyContent::FlexStart,
            "hover.justify-content" => hover.node.justify_content = default.node.justify_content,
            "click.justify-content" => click.node.justify_content = default.node.justify_content,

            "bg-color" => default.bg_color = Some(base_bg),
            "hover.bg-color" => hover.bg_color = Some(lighten_color(base_bg, 0.05)),
            "click.bg-color" => click.bg_color = Some(base_bg),

            "border-color" => default.border_color = Some(BorderColor::all(Color::srgb(0.3, 0.3, 0.35))),
            "hover.border-color" => hover.border_color = Some(BorderColor::all(Color::srgb(0.5, 0.5, 0.6))),

            "border" => default.node.border = UiRect::all(Val::Px(1.0)),
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
    /// The value of the text input.
    pub value: String,
    /// The placeholder text to display when the text input is empty.
    pub placeholder: String,
    /// The color of the inner text.
    pub text_color: Color,
    /// The color of the placeholder text.
    pub placeholder_color: Color,
    /// The font size of the text in pixels.
    pub font_size: f32,
}

impl TextInputProps {
    fn parse(
        node: &roxmltree::Node,
        prefix: Option<&str>,
        base: &Self,
    ) -> std::result::Result<Self, String> {
        let value = parse_attribute(node, "value", prefix, |s| Ok(s.to_string()))?
            .unwrap_or_else(|| base.value.clone());

        let placeholder = parse_attribute(node, "placeholder", prefix, |s| Ok(s.to_string()))?
            .unwrap_or_else(|| base.placeholder.clone());

        let text_color =
            parse_attribute(node, "text-color", prefix, parse_color)?.unwrap_or(base.text_color);

        let placeholder_color = parse_attribute(node, "placeholder-color", prefix, parse_color)?
            .unwrap_or(base.placeholder_color);

        let font_size =
            parse_attribute(node, "font-size", prefix, parse_float)?.unwrap_or(base.font_size);

        Ok(Self {
            value,
            placeholder,
            text_color,
            placeholder_color,
            font_size,
        })
    }
}

impl Default for TextInputProps {
    fn default() -> Self {
        Self {
            value: "".to_string(),
            placeholder: "".to_string(),
            text_color: Color::WHITE,
            placeholder_color: Color::srgb(0.5, 0.5, 0.5),
            font_size: 14.0,
        }
    }
}
