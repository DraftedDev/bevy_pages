use crate::element::ElementProps;
use crate::parser::AttributeMap;
use crate::parser::color::parse_color;
use crate::parser::values::{parse_attribute, parse_font_size, parse_matches};
use crate::props::Properties;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, PositionType, UiRect, Val, ZIndex};
use roxmltree::Node as XmlNode;

fn sync_visuals(
    query: Query<(&Interaction, &Children), (With<Properties<TooltipProps>>, Changed<Interaction>)>,
    mut popup_query: Query<&mut Node, With<TooltipPopup>>,
) {
    for (interaction, children) in &query {
        let is_hovered =
            *interaction == Interaction::Hovered || *interaction == Interaction::Pressed;

        for &child in children {
            if let Ok(mut node) = popup_query.get_mut(child) {
                node.display = if is_hovered {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}

fn update_props(
    mut query: Query<
        (Entity, &Interaction, &Properties<TooltipProps>),
        Or<(Changed<Interaction>, Changed<Properties<TooltipProps>>)>,
    >,
    children_query: Query<&Children>,
    mut popup_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<TooltipPopup>, Without<TooltipText>),
    >,
    mut text_query: Query<
        (&mut Text, &mut TextColor, &mut TextFont),
        (With<TooltipText>, Without<TooltipPopup>),
    >,
) {
    const OFFSET: Val = Val::Px(6.0);

    for (entity, interaction, props) in &mut query {
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        for descendant in children_query.iter_descendants(entity) {
            if let Ok((mut popup_node, mut popup_bg)) = popup_query.get_mut(descendant) {
                crate::set_if_changed!(popup_bg.0, active_props.bg_color);

                popup_node.top = Val::Auto;
                popup_node.bottom = Val::Auto;
                popup_node.left = Val::Auto;
                popup_node.right = Val::Auto;
                popup_node.margin = UiRect::ZERO;

                match active_props.anchor {
                    TooltipAnchor::Top => {
                        popup_node.bottom = Val::Percent(100.0);
                        popup_node.margin = UiRect::bottom(OFFSET);
                    }
                    TooltipAnchor::Bottom => {
                        popup_node.top = Val::Percent(100.0);
                        popup_node.margin = UiRect::top(OFFSET);
                    }
                    TooltipAnchor::Left => {
                        popup_node.right = Val::Percent(100.0);
                        popup_node.margin = UiRect::right(OFFSET);
                    }
                    TooltipAnchor::Right => {
                        popup_node.left = Val::Percent(100.0);
                        popup_node.margin = UiRect::left(OFFSET);
                    }
                }
            }

            if let Ok((mut text, mut color, mut font)) = text_query.get_mut(descendant) {
                crate::set_if_changed!(
                    text.0, active_props.text => active_props.text.clone();
                    color.0, active_props.text_color => active_props.text_color;
                    font.font_size, active_props.font_size => active_props.font_size;
                );
            }
        }
    }
}

/// The anchor point for the tooltip popup relative to its children.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TooltipAnchor {
    /// The tooltip will be displayed above the children.
    #[default]
    Top,
    /// The tooltip will be displayed under the children.
    Bottom,
    /// The tooltip will be displayed to the left to the children.
    Left,
    /// The tooltip will be displayed to the right to the children.
    Right,
}

impl TooltipAnchor {
    /// Parses a string into a [TooltipAnchor].
    #[inline(always)]
    pub fn parse(s: &str) -> Result<Self, String> {
        parse_matches(
            s,
            &[
                ("top", &|| Ok(Self::Top)),
                ("bottom", &|| Ok(Self::Bottom)),
                ("left", &|| Ok(Self::Left)),
                ("right", &|| Ok(Self::Right)),
            ],
        )
    }
}

/// Marker component for the tooltip floating popup box.
#[derive(Component, Debug, Clone, Copy)]
pub struct TooltipPopup;

/// Marker component for the tooltip text node.
#[derive(Component, Debug, Clone, Copy)]
pub struct TooltipText;

/// A tooltip widget that displays additional information when hovered.
///
/// ## XML Usage
///
/// Build a new tooltip widget using the `<Tooltip></Tooltip>` tag.
///
/// The children of the tooltip will trigger the tooltip popup when hovered on.
///
/// ### Attributes
/// - `text = "<string>"`: The tooltip text.
/// - `anchor = "<top|bottom|left|right>"`: The anchor/position of the tooltip relative to its children. See [TooltipAnchor].
/// - `tooltip-bg-color = "<color>"`: The background color of the tooltip popup.
/// - `text-color = "<color>"`: The text color inside the tooltip popup.
/// - `font-size = "<fontSize>"`: The font size of the text. See [parse_font_size].
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// Use [TooltipProps] to control the text.
/// You can use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct TooltipWidget {
    props: Properties<TooltipProps>,
}

impl Widget for TooltipWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Tooltip"
    }

    fn setup(&self, app: &mut App) {
        app.add_systems(Update, (update_props, sync_visuals).in_set(PageSystemSet));
    }

    fn parse(&mut self, _: &XmlNode, attrs: AttributeMap) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = TooltipProps::parse(&attrs, None, &TooltipProps::default())?;
        let hover = TooltipProps::parse(&attrs, Some("hover"), &default)?;
        let click = TooltipProps::parse(&attrs, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, entity: Entity, world: &mut World) -> Entity {
        let props = &self.props.default;

        let mut popup_node = Node {
            position_type: PositionType::Absolute,
            display: Display::None, // Hidden by default; shown on hover
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        };

        const OFFSET: Val = Val::Px(6.0);

        match props.anchor {
            TooltipAnchor::Top => {
                popup_node.bottom = Val::Percent(100.0);
                popup_node.margin = UiRect::bottom(OFFSET);
            }
            TooltipAnchor::Bottom => {
                popup_node.top = Val::Percent(100.0);
                popup_node.margin = UiRect::top(OFFSET);
            }
            TooltipAnchor::Left => {
                popup_node.right = Val::Percent(100.0);
                popup_node.margin = UiRect::left(OFFSET);
            }
            TooltipAnchor::Right => {
                popup_node.left = Val::Percent(100.0);
                popup_node.margin = UiRect::left(OFFSET);
            }
        }

        world
            .entity_mut(entity)
            .insert((self.props.clone(), FocusPolicy::Block));

        let popup_entity = world
            .spawn((
                TooltipPopup,
                popup_node,
                BackgroundColor(props.bg_color),
                ZIndex(100),
                FocusPolicy::Pass,
                ChildOf(entity),
            ))
            .id();

        world.spawn((
            TooltipText,
            Text::new(&props.text),
            TextFont {
                font_size: props.font_size,
                ..default()
            },
            TextColor(props.text_color),
            FocusPolicy::Pass,
            ChildOf(popup_entity),
        ));

        entity
    }

    fn apply_defaults(
        &self,
        node: &XmlNode,
        default: &mut ElementProps,
        hover: &mut ElementProps,
        click: &mut ElementProps,
    ) {
        crate::set_missing_attrs!(
            node,

            "position" => default.node.position_type = PositionType::Relative,
            "hover.position" => hover.node.position_type = default.node.position_type,
            "click.position" => click.node.position_type = default.node.position_type,

            "display" => default.node.display = Display::Flex,
            "hover.display" => hover.node.display = default.node.display,
            "click.display" => click.node.display = default.node.display,

            "align-items" => default.node.align_items = AlignItems::Center,
            "hover.align-items" => hover.node.align_items = default.node.align_items,
            "click.align-items" => click.node.align_items = default.node.align_items,

            "justify-content" => default.node.justify_content = JustifyContent::Center,
            "hover.justify-content" => hover.node.justify_content = default.node.justify_content,
            "click.justify-content" => click.node.justify_content = default.node.justify_content,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties for a [TooltipWidget] widget.
#[derive(Clone, Debug)]
pub struct TooltipProps {
    /// The text content displayed inside the tooltip popup.
    pub text: String,
    /// The anchor point for the tooltip popup.
    pub anchor: TooltipAnchor,
    /// Background color of the tooltip bubble.
    pub bg_color: Color,
    /// Text color inside the tooltip bubble.
    pub text_color: Color,
    /// Font size of the text.
    pub font_size: FontSize,
}

impl TooltipProps {
    fn parse(
        attrs: &AttributeMap,
        prefix: Option<&str>,
        base: &Self,
    ) -> std::result::Result<Self, String> {
        let text = parse_attribute(attrs, "text", prefix, |s| Ok(s.to_string()))?
            .unwrap_or_else(|| base.text.clone());

        let anchor =
            parse_attribute(attrs, "anchor", prefix, TooltipAnchor::parse)?.unwrap_or(base.anchor);

        let bg_color = parse_attribute(attrs, "tooltip-bg-color", prefix, parse_color)?
            .unwrap_or(base.bg_color);

        let text_color =
            parse_attribute(attrs, "text-color", prefix, parse_color)?.unwrap_or(base.text_color);

        let font_size =
            parse_attribute(attrs, "font-size", prefix, parse_font_size)?.unwrap_or(base.font_size);

        Ok(Self {
            text,
            anchor,
            bg_color,
            text_color,
            font_size,
        })
    }
}

impl Default for TooltipProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            text: String::new(),
            anchor: TooltipAnchor::default(),
            bg_color: Color::srgb(0.1, 0.1, 0.12),
            text_color: Color::WHITE,
            font_size: FontSize::Px(12.0),
        }
    }
}
