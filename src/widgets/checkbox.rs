use crate::element::{ElementId, ElementProps};
use crate::events::{ElementClick, ElementToggle};
use crate::parser::color::{darken_color, lighten_color, parse_color};
use crate::parser::values::{parse_attribute, parse_bool};
use crate::props::Properties;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::app::{App, Update};
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::prelude::{
    AlignItems, AlignSelf, BorderColor, Changed, Children, Commands, Component, Entity,
    EntityCommands, FontSize, Interaction, IntoScheduleConfigs, JustifyContent, JustifySelf, Node,
    On, Or, Query, Text, TextColor, TextFont, UiRect, Val, With,
};
use bevy::ui::Display;
use roxmltree::Node as XmlNode;

fn toggle_checkbox(
    trigger: On<ElementClick>,
    mut commands: Commands,
    mut query: Query<(&mut CheckboxState, Option<&ElementId>)>,
) {
    if let Ok((mut state, id)) = query.get_mut(trigger.entity) {
        state.0 = !state.0;

        commands.trigger(ElementToggle {
            entity: trigger.entity,
            id: id.cloned(),
            state: state.0,
        });
    }
}

fn sync_visuals(
    query: Query<(&CheckboxState, &Children), Changed<CheckboxState>>,
    mut checkmarks: Query<&mut Node, With<CheckboxCheckmark>>,
) {
    for (state, children) in &query {
        for child in children.iter() {
            if let Ok(mut node) = checkmarks.get_mut(*child) {
                node.display = if state.0 {
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
        (&Interaction, &Properties<CheckboxProps>, &Children),
        Or<(Changed<Interaction>, Changed<Properties<CheckboxProps>>)>,
    >,
    mut checkmark_query: Query<(&mut Text, &mut TextColor), With<CheckboxCheckmark>>,
) {
    for (interaction, props, children) in &mut query {
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        for child in children.iter() {
            if let Ok((mut text, mut color)) = checkmark_query.get_mut(*child) {
                crate::set_if_changed!(
                    text.0, active_props.symbol => active_props.symbol.clone();
                    color.0, active_props.check_color;
                );
            }
        }
    }
}

/// The runtime state of a checkbox widget.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Component)]
pub struct CheckboxState(pub bool);

/// Marker component for the inner checkmark visual entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct CheckboxCheckmark;

/// A checkbox widget.
///
/// ## XML Usage
///
/// Build a checkbox widget using the `<Checkbox />` tag.
///
/// ### Attributes
/// - `checked = "<bool>"`: The state of the checkbox.
/// - `check-color = "<color>"`: The color of the checkmark.
/// - `check-symbol = "<string>"`: The checkmark symbol.
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// Use [CheckboxProps] to control the checkbox widget.
/// Furthermore, the widget emits [ElementToggle] events when toggled.
///
/// You may also use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct CheckboxWidget {
    props: Properties<CheckboxProps>,
}

impl Widget for CheckboxWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Checkbox"
    }

    fn setup(&self, app: &mut App) {
        app.add_systems(Update, (update_props, sync_visuals.in_set(PageSystemSet)))
            .add_observer(toggle_checkbox);
    }

    fn parse(&mut self, node: &XmlNode) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = CheckboxProps::parse(node, None, &CheckboxProps::default())?;
        let hover = CheckboxProps::parse(node, Some("hover"), &default)?;
        let click = CheckboxProps::parse(node, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, _: &AssetServer) -> Entity {
        let props = &self.props.default;

        commands.insert((self.props.clone(), CheckboxState(props.state)));

        commands.with_children(|parent| {
            parent.spawn((
                CheckboxCheckmark,
                Text::new(&props.symbol),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..Default::default()
                },
                TextColor(props.check_color),
                Node {
                    display: if props.state {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
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
            .unwrap_or_else(|| Color::srgb(0.18, 0.18, 0.22));

        crate::set_missing_attrs!(
            node,

            "width" => default.node.width = Val::Px(20.0),
            "hover.width" => hover.node.width = default.node.width,
            "click.width" => click.node.width = default.node.width,

            "height" => default.node.height = Val::Px(20.0),
            "hover.height" => hover.node.height = default.node.height,
            "click.height" => click.node.height = default.node.height,

            "align-items" => default.node.align_items = AlignItems::Center,
            "hover.align-items" => hover.node.align_items = default.node.align_items,
            "click.align-items" => click.node.align_items = default.node.align_items,

            "justify-content" => default.node.justify_content = JustifyContent::Center,
            "hover.justify-content" => hover.node.justify_content = default.node.justify_content,
            "click.justify-content" => click.node.justify_content = default.node.justify_content,

            "bg-color" => default.bg_color = Some(base_bg),
            "hover.bg-color" => hover.bg_color = Some(lighten_color(base_bg, 0.12)),
            "click.bg-color" => click.bg_color = Some(darken_color(base_bg, 0.08)),

            "border-color" => default.border_color = Some(BorderColor::all(Color::srgb(0.4, 0.4, 0.45))),
            "hover.border-color" => hover.border_color = Some(BorderColor::all(Color::srgb(0.6, 0.6, 0.7))),
            "click.border-color" => click.border_color = default.border_color,

            "border" => default.node.border = UiRect::all(Val::Px(1.5)),
            "hover.border" => hover.node.border = default.node.border,
            "click.border" => click.node.border = default.node.border,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties of a [CheckboxWidget].
#[derive(Clone, Debug)]
pub struct CheckboxProps {
    /// The state of the checkbox.
    pub state: bool,
    /// Color of the checkmark.
    pub check_color: Color,
    /// Symbol used for the check indicator (defaults to "x").
    pub symbol: String,
}

impl CheckboxProps {
    fn parse(node: &XmlNode, prefix: Option<&str>, base: &Self) -> Result<Self, String> {
        let state = parse_attribute(node, "checked", prefix, parse_bool)?.unwrap_or(base.state);

        let check_color =
            parse_attribute(node, "check-color", prefix, parse_color)?.unwrap_or(base.check_color);

        let symbol = parse_attribute(node, "check-symbol", prefix, |s| Ok(s.to_string()))?
            .unwrap_or_else(|| base.symbol.clone());

        Ok(Self {
            state,
            check_color,
            symbol,
        })
    }
}

impl Default for CheckboxProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            state: false,
            check_color: Color::WHITE,
            symbol: "x".to_string(),
        }
    }
}
