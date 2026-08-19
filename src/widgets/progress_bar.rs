use crate::element::ElementProps;
use crate::parser::color::parse_color;
use crate::parser::values::{parse_attribute, parse_float};
use crate::props::Properties;
use crate::widgets::Widget;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui::{BackgroundColor, BorderRadius, Node, PositionType, Val};
use roxmltree::Node as XmlNode;

pub(crate) fn sync_progress_bar_visuals(
    pb_query: Query<(&ProgressBarState, &Children), Changed<ProgressBarState>>,
    mut fill_query: Query<&mut Node, With<ProgressBarFill>>,
) {
    for (pb, children) in &pb_query {
        let pct = pb.percentage();

        for &child in children {
            if let Ok(mut fill_node) = fill_query.get_mut(child) {
                fill_node.width = Val::Percent(pct);
            }
        }
    }
}

pub(crate) fn update_props(
    mut root_query: Query<
        (
            Ref<Properties<ProgressBarProps>>,
            &Interaction,
            &Children,
            &mut ProgressBarState,
        ),
        Or<(Changed<Properties<ProgressBarProps>>, Changed<Interaction>)>,
    >,
    mut track_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<ProgressBarTrack>, Without<ProgressBarFill>),
    >,
    mut fill_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<ProgressBarFill>, Without<ProgressBarTrack>),
    >,
) {
    for (props, interaction, children, mut state) in &mut root_query {
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        if props.is_changed() {
            crate::set_if_changed!(
                state.min, active_props.min;
                state.max, active_props.max;
                state.value, active_props.value => active_props.value.clamp(active_props.min, active_props.max);
            );
        }

        for &child in children {
            // Background Track
            if let Ok((mut track_node, mut track_bg)) = track_query.get_mut(child) {
                crate::set_if_changed!(
                    track_bg.0, active_props.track_color;
                    track_node.height, Val::Px(active_props.track_height);
                    track_node.border_radius, BorderRadius::all(Val::Px(active_props.track_height / 2.0));
                );
            }

            // Inner Active Fill Bar
            if let Ok((mut fill_node, mut fill_bg)) = fill_query.get_mut(child) {
                crate::set_if_changed!(
                    fill_bg.0, active_props.fill_color;
                    fill_node.height, Val::Px(active_props.track_height);
                    fill_node.border_radius, BorderRadius::all(Val::Px(active_props.track_height / 2.0));
                );
            }
        }
    }
}

/// The internal progress bar state.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct ProgressBarState {
    /// The minimum value.
    pub min: f32,
    /// The maximum value.
    pub max: f32,
    /// The current value.
    pub value: f32,
}

impl ProgressBarState {
    /// Returns the progress normalized between `0.0` and `1.0`.
    #[inline(always)]
    pub fn normalized(&self) -> f32 {
        if self.max > self.min {
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Returns the current progress as a percentage between `0.0` and `100.0`.
    #[inline(always)]
    pub fn percentage(&self) -> f32 {
        self.normalized() * 100.0
    }
}

/// Marker component for the background track node.
#[derive(Component)]
pub struct ProgressBarTrack;

/// Marker component for the active inner fill bar node.
#[derive(Component)]
pub struct ProgressBarFill;

/// A progress bar widget.
///
/// ## XML Usage
///
/// Build a new progress bar using the `<ProgressBar />` tag.
///
/// ### Attributes
/// - `min = "<float>"`: The minimum of the progress bar.
/// - `max = "<float>"`: The maximum of the progress bar.
/// - `value = "<float>"`: The value of the progress bar.
/// - `track-color = "<color>"`: The background color of the progress bar container track.
/// - `fill-color = "<color>"`: The color of the inner filled progress indicator bar.
/// - `track-height = "<float>"`: The height of the track in pixels.
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// Use the [ProgressBarProps] to control the progress bar.
/// You may also use generic element events to implement custom behavior.
#[derive(Debug, Clone, Default)]
pub struct ProgressBarWidget {
    props: Properties<ProgressBarProps>,
}

impl Widget for ProgressBarWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "ProgressBar"
    }

    fn parse(&mut self, node: &XmlNode) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = ProgressBarProps::parse(node, None, &ProgressBarProps::default())?;
        let hover = ProgressBarProps::parse(node, Some("hover"), &default)?;
        let click = ProgressBarProps::parse(node, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, _: &AssetServer) -> Entity {
        let props = &self.props.default;

        let norm_val = if props.max > props.min {
            ((props.value - props.min) / (props.max - props.min)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let pct = norm_val * 100.0;

        commands
            .insert((
                self.props.clone(),
                ProgressBarState {
                    min: props.min,
                    max: props.max,
                    value: props.value,
                },
            ))
            .with_children(|parent| {
                // Background Track
                parent.spawn((
                    ProgressBarTrack,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(props.track_height),
                        border_radius: BorderRadius::all(Val::Px(props.track_height / 2.0)),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(props.track_color),
                ));

                // Active Fill Bar
                parent.spawn((
                    ProgressBarFill,
                    Node {
                        width: Val::Percent(pct),
                        height: Val::Px(props.track_height),
                        border_radius: BorderRadius::all(Val::Px(props.track_height / 2.0)),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(props.fill_color),
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
        crate::set_missing_attrs!(
            node,

            "width" => default.node.width = Val::Px(200.0),
            "hover.width" => hover.node.width = default.node.width,
            "click.width" => click.node.width = default.node.width,

            "height" => default.node.height = Val::Px(12.0),
            "hover.height" => hover.node.height = default.node.height,
            "click.height" => click.node.height = default.node.height,

            "align-items" => default.node.align_items = AlignItems::Center,
            "hover.align-items" => hover.node.align_items = default.node.align_items,
            "click.align-items" => click.node.align_items = default.node.align_items,

            "justify-content" => default.node.justify_content = JustifyContent::FlexStart,
            "hover.justify-content" => hover.node.justify_content = default.node.justify_content,
            "click.justify-content" => click.node.justify_content = default.node.justify_content,

            "position" => default.node.position_type = PositionType::Relative,
            "hover.position" => hover.node.position_type = default.node.position_type,
            "click.position" => click.node.position_type = default.node.position_type,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties of a [ProgressBarWidget].
#[derive(Clone, Debug)]
pub struct ProgressBarProps {
    /// The minimum value of the progress bar.
    pub min: f32,
    /// The maximum value of the progress bar.
    pub max: f32,
    /// The current value of the progress bar.
    pub value: f32,
    /// Background color of the progress bar container track.
    pub track_color: Color,
    /// Color of the inner filled progress indicator bar.
    pub fill_color: Color,
    /// Height of the track in pixels.
    ///
    /// TODO: Support bevy's [Val].
    pub track_height: f32,
}

impl ProgressBarProps {
    fn parse(
        node: &XmlNode,
        prefix: Option<&str>,
        base: &Self,
    ) -> std::result::Result<Self, String> {
        let min = parse_attribute(node, "min", prefix, parse_float)?.unwrap_or(base.min);

        let max = parse_attribute(node, "max", prefix, parse_float)?.unwrap_or(base.max);

        if min >= max {
            return Err(format!(
                "ProgressBar 'min' ({}) must be strictly less than 'max' ({})",
                min, max
            ));
        }

        let value = parse_attribute(node, "value", prefix, parse_float)?
            .unwrap_or(base.min)
            .clamp(min, max);

        let track_color =
            parse_attribute(node, "track-color", prefix, parse_color)?.unwrap_or(base.track_color);

        let fill_color =
            parse_attribute(node, "fill-color", prefix, parse_color)?.unwrap_or(base.fill_color);

        let track_height = parse_attribute(node, "track-height", prefix, parse_float)?
            .unwrap_or(base.track_height);

        Ok(Self {
            min,
            max,
            value,
            track_color,
            fill_color,
            track_height,
        })
    }
}

impl Default for ProgressBarProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            value: 0.0,
            track_color: Color::srgb(0.16, 0.17, 0.20),
            fill_color: Color::srgb(0.38, 0.69, 0.94),
            track_height: 12.0,
        }
    }
}
