use crate::element::{ElementId, ElementProps};
use crate::events::{ElementClick, ElementToggle};
use crate::parser::AttributeMap;
use crate::parser::color::{darken_color, lighten_color, parse_color};
use crate::parser::values::{parse_attribute, parse_bool, parse_val};
use crate::props::Properties;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::ui::{AlignItems, BorderColor, JustifyContent, Node, PositionType, UiRect, Val};
use roxmltree::Node as XmlNode;

fn toggle_switch(
    trigger: On<ElementClick>,
    mut commands: Commands,
    mut query: Query<(&mut SwitchState, Option<&ElementId>)>,
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
    query: Query<(&SwitchState, &Properties<SwitchProps>, &Children), Changed<SwitchState>>,
    mut thumbs: Query<(&mut Node, &mut BackgroundColor), With<SwitchThumb>>,
) {
    for (state, props, children) in &query {
        let active_props = &props.default;

        let active_thumb_color = if state.0 {
            active_props.thumb_color_on
        } else {
            active_props.thumb_color
        };

        for child in children.iter() {
            if let Ok((mut node, mut bg_color)) = thumbs.get_mut(child) {
                let target_left = if state.0 {
                    Val::Percent(100.0)
                } else {
                    Val::Px(0.0)
                };

                let target_margin = if state.0 {
                    UiRect::left(-active_props.thumb_size)
                } else {
                    UiRect::ZERO
                };

                crate::set_if_changed!(
                    node.left, target_left;
                    node.margin, target_margin;
                    bg_color.0, active_thumb_color;
                );
            }
        }
    }
}

fn update_props(
    mut query: Query<
        (
            &Interaction,
            &Properties<SwitchProps>,
            &SwitchState,
            &Children,
        ),
        Or<(Changed<Interaction>, Changed<Properties<SwitchProps>>)>,
    >,
    mut thumb_query: Query<(&mut Node, &mut BackgroundColor), With<SwitchThumb>>,
) {
    for (interaction, props, state, children) in &mut query {
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        let active_thumb_color = if state.0 {
            active_props.thumb_color_on
        } else {
            active_props.thumb_color
        };

        for child in children.iter() {
            if let Ok((mut node, mut bg_color)) = thumb_query.get_mut(child) {
                let target_left = if state.0 {
                    Val::Percent(100.0)
                } else {
                    Val::Px(0.0)
                };

                let target_margin = if state.0 {
                    UiRect::left(-active_props.thumb_size)
                } else {
                    UiRect::ZERO
                };

                crate::set_if_changed!(
                    node.width, active_props.thumb_size;
                    node.height, active_props.thumb_size;
                    node.border_radius, BorderRadius::all(active_props.thumb_size / 2.0);
                    node.left, target_left;
                    node.margin, target_margin;
                    bg_color.0, active_thumb_color;
                );
            }
        }
    }
}

/// The runtime state of a switch widget.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Component)]
pub struct SwitchState(pub bool);

/// Marker component for the inner thumb entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct SwitchThumb;

/// A switch widget.
///
/// It's really just a fancier checkbox.
///
/// ## XML Usage
///
/// Build a switch widget using the `<Switch />` tag.
///
/// ### Attributes
/// - `toggled = "<bool>"`: The state of the switch.
/// - `thumb-color = "<color>"`: The color of the thumb when not toggled.
/// - `thumb-color-on = "<color>"`: The color of the thumb when toggled.
/// - `thumb-size = "<size>"`: The size of the thumb.
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// Use [SwitchProps] to control the switch widget.
/// Furthermore, the widget emits [ElementToggle] events when toggled.
///
/// You may also use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct SwitchWidget {
    props: Properties<SwitchProps>,
}

impl Widget for SwitchWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Switch"
    }

    fn setup(&self, app: &mut App) {
        app.add_systems(Update, (sync_visuals, update_props).in_set(PageSystemSet))
            .add_observer(toggle_switch);
    }

    fn parse(&mut self, _: &XmlNode, attrs: &AttributeMap) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = SwitchProps::parse(attrs, None, &SwitchProps::default())?;
        let hover = SwitchProps::parse(attrs, Some("hover"), &default)?;
        let click = SwitchProps::parse(attrs, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, entity: Entity, world: &mut World) -> Entity {
        let props = &self.props.default;

        let active_thumb_color = if props.state {
            props.thumb_color_on
        } else {
            props.thumb_color
        };

        let left = if props.state {
            Val::Percent(100.0)
        } else {
            Val::Px(0.0)
        };

        let margin = if props.state {
            UiRect::left(-props.thumb_size)
        } else {
            UiRect::ZERO
        };

        world
            .entity_mut(entity)
            .insert((self.props.clone(), SwitchState(props.state)));

        world.spawn((
            ChildOf(entity),
            SwitchThumb,
            Node {
                width: props.thumb_size,
                height: props.thumb_size,
                border_radius: BorderRadius::all(props.thumb_size / 2.0),
                position_type: PositionType::Relative,
                left,
                margin,
                ..Default::default()
            },
            BackgroundColor(active_thumb_color),
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
            .unwrap_or_else(|| Color::srgb(0.18, 0.18, 0.18));

        crate::set_missing_attrs!(
            attrs,

            "width" => default.node.width = Val::Px(40.0),
            "hover.width" => hover.node.width = default.node.width,
            "click.width" => click.node.width = default.node.width,

            "height" => default.node.height = Val::Px(20.0),
            "hover.height" => hover.node.height = default.node.height,
            "click.height" => click.node.height = default.node.height,

            "border-radius" => default.node.border_radius = BorderRadius::all(Val::Px(10.0)),
            "hover.border-radius" => hover.node.border_radius = default.node.border_radius,
            "click.border-radius" => click.node.border_radius = default.node.border_radius,

            "align-items" => default.node.align_items = AlignItems::Center,
            "hover.align-items" => hover.node.align_items = default.node.align_items,
            "click.align-items" => click.node.align_items = default.node.align_items,

            "justify-content" => default.node.justify_content = JustifyContent::FlexStart,
            "hover.justify-content" => hover.node.justify_content = default.node.justify_content,
            "click.justify-content" => click.node.justify_content = default.node.justify_content,

            "padding" => default.node.padding = UiRect::all(Val::Px(2.0)),
            "hover.padding" => hover.node.padding = default.node.padding,
            "click.padding" => click.node.padding = default.node.padding,

            "bg-color" => default.bg_color = Some(base_bg),
            "hover.bg-color" => hover.bg_color = Some(lighten_color(base_bg, 0.12)),
            "click.bg-color" => click.bg_color = Some(darken_color(base_bg, 0.08)),

            "border-color" => default.border_color = Some(BorderColor::all(Color::srgb(0.4, 0.4, 0.4))),
            "hover.border-color" => hover.border_color = Some(BorderColor::all(Color::srgb(0.65, 0.65, 0.65))),
            "click.border-color" => click.border_color = default.border_color,

            "border" => default.node.border = UiRect::all(Val::Px(1.0)),
            "hover.border" => hover.node.border = default.node.border,
            "click.border" => click.node.border = default.node.border,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties of a [SwitchWidget].
#[derive(Clone, Debug)]
pub struct SwitchProps {
    /// The state of the switch.
    pub state: bool,
    /// Color of the thumb when off.
    pub thumb_color: Color,
    /// Color of the thumb when on.
    pub thumb_color_on: Color,
    /// Size of the thumb indicator.
    pub thumb_size: Val,
}

impl SwitchProps {
    fn parse(attrs: &AttributeMap, prefix: Option<&str>, base: &Self) -> Result<Self, String> {
        let state = parse_attribute(attrs, "toggled", prefix, parse_bool)?.unwrap_or(base.state);

        let thumb_color =
            parse_attribute(attrs, "thumb-color", prefix, parse_color)?.unwrap_or(base.thumb_color);

        let thumb_color_on = parse_attribute(attrs, "thumb-color-on", prefix, parse_color)?
            .unwrap_or(base.thumb_color_on);

        let thumb_size =
            parse_attribute(attrs, "thumb-size", prefix, parse_val)?.unwrap_or(base.thumb_size);

        Ok(Self {
            state,
            thumb_color,
            thumb_color_on,
            thumb_size,
        })
    }
}

impl Default for SwitchProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            state: false,
            thumb_color: Color::srgb(0.4, 0.4, 0.4),
            thumb_color_on: Color::srgb(0.9, 0.9, 0.9),
            thumb_size: Val::Px(16.0),
        }
    }
}
