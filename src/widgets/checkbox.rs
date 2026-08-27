use crate::element::{ElementId, ElementProps};
use crate::events::{ElementClick, ElementToggle};
use crate::parser::AttributeMap;
use crate::parser::color::{darken_color, lighten_color, parse_color};
use crate::parser::values::{parse_attribute, parse_bool, parse_val};
use crate::props::Properties;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::app::{App, Update};
use bevy::color::Color;
use bevy::prelude::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, Changed, ChildOf, Children,
    Commands, Component, Entity, Interaction, IntoScheduleConfigs, JustifyContent, JustifySelf,
    Node, On, Or, Query, UiRect, Val, With, World,
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
    mut checkmark_query: Query<(&mut Node, &mut BackgroundColor), With<CheckboxCheckmark>>,
) {
    for (interaction, props, children) in &mut query {
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        for child in children.iter() {
            if let Ok((mut node, mut bg)) = checkmark_query.get_mut(*child) {
                crate::set_if_changed!(
                    node.width, active_props.check_width => active_props.check_width;
                    node.height, active_props.check_height => active_props.check_height;
                    bg.0, active_props.check_color => active_props.check_color;
                    node.border_radius, active_props.check_radius => active_props.check_radius;
                );
            }
        }
    }
}

/// The runtime state of a checkbox widget.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Component)]
pub struct CheckboxState(pub bool);

/// Marker component for the inner checkmark rectangle entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct CheckboxCheckmark;

/// A checkbox widget using an inner rounded rectangle indicator.
///
/// ## XML Usage
///
/// Build a checkbox widget using the `<Checkbox />` tag.
///
/// ### Attributes
/// - `checked = "<bool>"`: The state of the checkbox.
/// - `check-color = "<color>"`: The background color of the inner marker.
/// - `check-width = "<size>"`: The width of the inner marker.
/// - `check-height = "<size>"`: The height of the inner marker.
/// - `check-radius = "<size>"`: The border radius of the inner marker.
///
/// All the attributes listed, except `checked`, support state overrides.
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
        app.add_systems(Update, (update_props, sync_visuals).in_set(PageSystemSet))
            .add_observer(toggle_checkbox);
    }

    fn parse(&mut self, _: &XmlNode, attrs: &AttributeMap) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = CheckboxProps::parse(attrs, None, &CheckboxProps::default())?;
        let hover = CheckboxProps::parse(attrs, Some("hover"), &default)?;
        let click = CheckboxProps::parse(attrs, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, entity: Entity, world: &mut World) -> Entity {
        let props = &self.props.default;

        world
            .entity_mut(entity)
            .insert((self.props.clone(), CheckboxState(props.state)));

        world.spawn((
            CheckboxCheckmark,
            BackgroundColor(props.check_color),
            Node {
                width: props.check_width,
                height: props.check_height,
                display: if props.state {
                    Display::Flex
                } else {
                    Display::None
                },
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                border_radius: props.check_radius,
                ..Default::default()
            },
            ChildOf(entity),
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
            .unwrap_or_else(|| Color::srgb(0.18, 0.18, 0.22));

        crate::set_missing_attrs!(
            attrs,

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

            "border-radius" => default.node.border_radius = BorderRadius::all(Val::Px(5.0)),
            "hover.border-radius" => hover.node.border_radius = default.node.border_radius,
            "click.border-radius" => click.node.border_radius = default.node.border_radius,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties of a [CheckboxWidget].
#[derive(Clone, Debug)]
pub struct CheckboxProps {
    /// The initial state of the checkbox.
    pub state: bool,
    /// Color of the inner marker rectangle.
    pub check_color: Color,
    /// Width of the inner marker rectangle.
    pub check_width: Val,
    /// Height of the inner marker rectangle.
    pub check_height: Val,
    /// Border radius of the inner marker rectangle.
    pub check_radius: BorderRadius,
}

impl CheckboxProps {
    fn parse(attrs: &AttributeMap, prefix: Option<&str>, base: &Self) -> Result<Self, String> {
        let state = parse_attribute(attrs, "checked", prefix, parse_bool)?.unwrap_or(base.state);

        let check_color =
            parse_attribute(attrs, "check-color", prefix, parse_color)?.unwrap_or(base.check_color);

        let check_width =
            parse_attribute(attrs, "check-width", prefix, parse_val)?.unwrap_or(base.check_width);

        let check_height =
            parse_attribute(attrs, "check-height", prefix, parse_val)?.unwrap_or(base.check_height);

        let check_radius = parse_attribute(attrs, "check-radius", prefix, parse_val)?
            .map(BorderRadius::all)
            .unwrap_or(base.check_radius);

        Ok(Self {
            state,
            check_color,
            check_width,
            check_height,
            check_radius,
        })
    }
}

impl Default for CheckboxProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            state: false,
            check_color: Color::srgb(1.0, 1.0, 1.0),
            check_width: Val::Px(10.0),
            check_height: Val::Px(10.0),
            check_radius: BorderRadius::all(Val::Px(2.5)),
        }
    }
}
