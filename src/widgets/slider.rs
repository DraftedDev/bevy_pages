use crate::element::{ElementId, ElementProps};
use crate::events::ElementSet;
use crate::parser::AttributeMap;
use crate::parser::color::parse_color;
use crate::parser::values::{parse_attribute, parse_float, parse_val};
use crate::props::Properties;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::color::Color;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{BackgroundColor, BorderRadius, Display, FlexDirection, Node, PositionType, Val};
use bevy::ui_widgets::{
    Slider, SliderDragState, SliderRange, SliderThumb, SliderValue, TrackClick, observe,
    slider_self_update,
};
use roxmltree::Node as XmlNode;

fn sync_visuals(
    mut commands: Commands,
    mut slider_query: Query<
        (
            Entity,
            &Properties<SliderProps>,
            &SliderValue,
            &SliderRange,
            &mut SliderState,
            Option<&ElementId>,
        ),
        Or<(Changed<SliderValue>, Changed<SliderRange>)>,
    >,
    children_query: Query<&Children>,
    mut fill_query: Query<&mut Node, (With<SliderFill>, Without<SliderThumb>)>,
    mut thumb_query: Query<&mut Node, (With<SliderThumb>, Without<SliderFill>)>,
) {
    for (entity, props, value, range, mut state, id) in &mut slider_query {
        let mut val = value.0;

        if let Some(step) = props.default.step
            && step > 0.0
        {
            val = (val / step).round() * step;
        }

        let pct = range.thumb_position(val) * 100.0;

        if (state.0 - val).abs() > f32::EPSILON {
            let delta = val - state.0;
            state.0 = val;

            commands.trigger(ElementSet {
                entity,
                id: id.cloned(),
                value: val,
                delta: Some(delta),
            });
        }

        for descendant in children_query.iter_descendants(entity) {
            if let Ok(mut fill_node) = fill_query.get_mut(descendant) {
                fill_node.width = Val::Percent(pct);
            }

            if let Ok(mut thumb_node) = thumb_query.get_mut(descendant) {
                thumb_node.left = Val::Percent(pct);
            }
        }
    }
}

fn update_props(
    mut slider_query: Query<
        (Entity, &Properties<SliderProps>, &Hovered, &SliderDragState),
        Or<(
            Changed<Properties<SliderProps>>,
            Changed<Hovered>,
            Changed<SliderDragState>,
        )>,
    >,
    children_query: Query<&Children>,
    mut track_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<SliderTrack>, Without<SliderFill>, Without<SliderThumb>),
    >,
    mut fill_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<SliderFill>, Without<SliderTrack>, Without<SliderThumb>),
    >,
    mut thumb_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<SliderThumb>, Without<SliderTrack>, Without<SliderFill>),
    >,
) {
    for (entity, props, hovered, drag_state) in &mut slider_query {
        let active_props = if drag_state.dragging {
            &props.click
        } else if hovered.0 {
            &props.hover
        } else {
            &props.default
        };

        for descendant in children_query.iter_descendants(entity) {
            if let Ok((mut track_node, mut track_bg)) = track_query.get_mut(descendant) {
                crate::set_if_changed!(
                    track_node.height, active_props.track_height;
                    track_node.border_radius, BorderRadius::all(active_props.track_height / 2.0);
                    track_bg.0, active_props.track_color;
                );
            }

            if let Ok((mut fill_node, mut fill_bg)) = fill_query.get_mut(descendant) {
                crate::set_if_changed!(
                    fill_node.height, active_props.track_height;
                    fill_node.border_radius, BorderRadius::all(active_props.track_height / 2.0);
                    fill_bg.0, active_props.fill_color;
                );
            }

            if let Ok((mut thumb_node, mut thumb_bg)) = thumb_query.get_mut(descendant) {
                crate::set_if_changed!(
                    thumb_node.width, active_props.thumb_size;
                    thumb_node.height, active_props.thumb_size;
                    thumb_node.border_radius, BorderRadius::all(active_props.thumb_size / 2.0);
                    thumb_bg.0, active_props.thumb_color;
                );
            }
        }
    }
}

/// The internal state of the slider.
#[derive(Copy, Clone, Debug, Component)]
pub struct SliderState(pub f32);

/// Marker component for the track of a Slider.
#[derive(Component)]
pub struct SliderTrack;

/// Marker component for the fill of a Slider.
#[derive(Component)]
pub struct SliderFill;

/// A slider widget.
///
/// ## XML Usage
///
/// Build a slider widget using the `<Slider />` tag.
///
/// ### Attributes
/// - `min = "<float>"`: The minimum of the slider.
/// - `max = "<float>"`: The maximum of the slider.
/// - `step = "<float>"`: The step size for the slider.
/// - `value = "<float>"`: The value of the slider.
/// - `track-color = "<color>"`: The color of the slider track.
/// - `thumb-color = "<color>"`: The color of the slider thumb.
/// - `fill-color = "<color>"`: The color of the slider fill.
/// - `track-height = "<size>"`: The height of the slider track.
/// - `thumb-size = "<size>"`: The size of the slider thumb.
///
/// All the attributes, except `min` and `max`, support state overrides.
///
/// ## Logic
///
/// Use the [SliderProps] to control the slider.
/// Furthermore, the slider emits `ElementSet<f32>` events.
///
/// You may also use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct SliderWidget {
    props: Properties<SliderProps>,
}

impl Widget for SliderWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Slider"
    }

    fn setup(&self, app: &mut App) {
        app.add_systems(Update, (update_props, sync_visuals).in_set(PageSystemSet));
    }

    fn parse(&mut self, _: &XmlNode, attrs: &AttributeMap) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = SliderProps::parse(attrs, None, &SliderProps::default())?;
        let hover = SliderProps::parse(attrs, Some("hover"), &default)?;
        let click = SliderProps::parse(attrs, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, entity: Entity, world: &mut World) -> Entity {
        let props = &self.props.default;

        world.entity_mut(entity).insert((
            self.props.clone(),
            SliderState(props.value),
            Slider {
                track_click: TrackClick::Snap,
                ..default()
            },
            SliderValue(props.value),
            SliderRange::new(props.min, props.max),
            Hovered::default(),
            SliderDragState::default(),
            observe(slider_self_update),
        ));

        world.spawn((
            SliderTrack,
            Node {
                width: Val::Percent(100.0),
                height: props.track_height,
                border_radius: BorderRadius::all(props.track_height / 2.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(props.track_color),
            ChildOf(entity),
        ));

        world.spawn((
            SliderFill,
            Node {
                width: Val::Percent(0.0),
                height: props.track_height,
                border_radius: BorderRadius::all(props.track_height / 2.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(props.fill_color),
            ChildOf(entity),
        ));

        let thumb_container = world
            .spawn((
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: props.thumb_size,
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ChildOf(entity),
            ))
            .id();

        world.spawn((
            SliderThumb,
            Node {
                display: Display::Flex,
                width: props.thumb_size,
                height: props.thumb_size,
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                border_radius: BorderRadius::all(props.thumb_size / 2.0),
                ..default()
            },
            BackgroundColor(props.thumb_color),
            ChildOf(thumb_container),
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
        crate::set_missing_attrs!(
            attrs,

            "width" => default.node.width = Val::Px(200.0),
            "hover.width" => hover.node.width = default.node.width,
            "click.width" => click.node.width = default.node.width,

            "height" => default.node.height = Val::Px(24.0),
            "hover.height" => hover.node.height = default.node.height,
            "click.height" => click.node.height = default.node.height,

            "display" => default.node.display = Display::Flex,
            "hover.display" => hover.node.display = default.node.display,
            "click.display" => click.node.display = default.node.display,

            "flex-direction" => default.node.flex_direction = FlexDirection::Column,
            "hover.flex-direction" => hover.node.flex_direction = default.node.flex_direction,
            "click.flex-direction" => click.node.flex_direction = default.node.flex_direction,

            "justify-content" => default.node.justify_content = JustifyContent::Center,
            "hover.justify-content" => hover.node.justify_content = default.node.justify_content,
            "click.justify-content" => click.node.justify_content = default.node.justify_content,

            "align-items" => default.node.align_items = AlignItems::Stretch,
            "hover.align-items" => hover.node.align_items = default.node.align_items,
            "click.align-items" => click.node.align_items = default.node.align_items,

            "position" => default.node.position_type = PositionType::Relative,
            "hover.position" => hover.node.position_type = default.node.position_type,
            "click.position" => click.node.position_type = default.node.position_type,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties of a [SliderWidget].
#[derive(Clone, Debug)]
pub struct SliderProps {
    /// The minimum slider value.
    pub min: f32,
    /// The maximum slider value.
    pub max: f32,
    /// The optional step size for the slider.
    pub step: Option<f32>,
    /// The value of the slider.
    pub value: f32,
    /// The slider track color.
    pub track_color: Color,
    /// The slider thumb color.
    pub thumb_color: Color,
    /// The slider fill color.
    pub fill_color: Color,
    /// The slider track height.
    pub track_height: Val,
    /// The slider thumb size.
    pub thumb_size: Val,
}

impl SliderProps {
    fn parse(
        attrs: &AttributeMap,
        prefix: Option<&str>,
        base: &Self,
    ) -> std::result::Result<Self, String> {
        let min = parse_attribute(attrs, "min", prefix, parse_float)?.unwrap_or(base.min);

        let max = parse_attribute(attrs, "max", prefix, parse_float)?.unwrap_or(base.max);

        if min >= max {
            return Err(format!(
                "Slider 'min' ({}) must be strictly less than 'max' ({})",
                min, max
            ));
        }

        let step = parse_attribute(attrs, "step", prefix, parse_float)?.or(base.step);

        let mut value = parse_attribute(attrs, "value", prefix, parse_float)?.unwrap_or(base.value);

        if let Some(step_val) = step
            && step_val > 0.0
        {
            value = (value / step_val).round() * step_val;
        }

        value = value.clamp(min, max);

        let track_color =
            parse_attribute(attrs, "track-color", prefix, parse_color)?.unwrap_or(base.track_color);

        let thumb_color =
            parse_attribute(attrs, "thumb-color", prefix, parse_color)?.unwrap_or(base.thumb_color);

        let fill_color =
            parse_attribute(attrs, "fill-color", prefix, parse_color)?.unwrap_or(base.fill_color);

        let track_height =
            parse_attribute(attrs, "track-height", prefix, parse_val)?.unwrap_or(base.track_height);

        let thumb_size =
            parse_attribute(attrs, "thumb-size", prefix, parse_val)?.unwrap_or(base.thumb_size);

        Ok(Self {
            min,
            max,
            step,
            value,
            track_color,
            thumb_color,
            fill_color,
            track_height,
            thumb_size,
        })
    }
}

impl Default for SliderProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            step: None,
            value: 0.0,
            track_color: Color::srgb(0.4, 0.4, 0.4),
            thumb_color: Color::WHITE,
            fill_color: Color::srgb(0.9, 0.9, 0.9),
            track_height: Val::Px(6.0),
            thumb_size: Val::Px(16.0),
        }
    }
}
