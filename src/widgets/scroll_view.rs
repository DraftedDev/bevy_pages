use crate::element::ElementProps;
use crate::parser::color::parse_color;
use crate::parser::values::{parse_attribute, parse_bool, parse_float, parse_matches};
use crate::props::Properties;
use crate::widgets::Widget;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{ControlOrientation, Scrollbar, ScrollbarDragState, ScrollbarThumb};
use roxmltree::Node as XmlNode;
use rustc_hash::FxHashSet;

pub(crate) fn update_scroll_bounds(
    mut scroll_query: Query<(Entity, &Children, &mut ScrollViewState), With<ScrollViewArea>>,
    content_query: Query<&ComputedNode, With<ScrollViewContent>>,
    area_query: Query<&ComputedNode, With<ScrollViewArea>>,
) {
    for (area_entity, children, mut state) in &mut scroll_query {
        if let Ok(area_node) = area_query.get(area_entity) {
            state.viewport_size = area_node.size();
        }

        for child in children.iter() {
            if let Ok(content_node) = content_query.get(child) {
                state.content_size = content_node.size();
                break;
            }
        }
    }
}

pub(crate) fn scroll_view_mouse_wheel(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut scroll_query: Query<(Entity, &Interaction, &mut ScrollViewState), With<ScrollViewArea>>,
    parent_query: Query<(&ChildOf, Option<&Interaction>)>,
) {
    let mut total_delta = Vec2::ZERO;

    for event in mouse_wheel_events.read() {
        let delta = match event.unit {
            // Scales by typical line height in pixels ~24px
            MouseScrollUnit::Line => Vec2::new(event.x, event.y) * 24.0,
            MouseScrollUnit::Pixel => Vec2::new(event.x, event.y),
        };
        total_delta += delta;
    }

    if total_delta == Vec2::ZERO {
        return;
    }

    for (entity, interaction, mut state) in &mut scroll_query {
        // Check hover status on entire widget
        let is_hovered = *interaction == Interaction::Hovered
            || *interaction == Interaction::Pressed
            || parent_query
                .get(entity)
                .ok()
                .and_then(|(parent, _)| parent_query.get(parent.0).ok())
                .and_then(|(_, parent_interaction)| parent_interaction)
                .is_some_and(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);

        if is_hovered {
            let max_scroll_x = (state.content_size.x - state.viewport_size.x).max(0.0);
            let max_scroll_y = (state.content_size.y - state.viewport_size.y).max(0.0);

            let scroll_step =
                Vec2::new(-total_delta.x, -total_delta.y) * (state.scroll_speed / 30.0);

            let new_x = state.target_offset.x + scroll_step.x;
            let new_y = state.target_offset.y + scroll_step.y;

            // Prevent clamping to zero if content size layout hasn't been computed yet
            state.target_offset.x = if max_scroll_x > 0.0 {
                new_x.clamp(0.0, max_scroll_x)
            } else {
                new_x.max(0.0)
            };

            state.target_offset.y = if max_scroll_y > 0.0 {
                new_y.clamp(0.0, max_scroll_y)
            } else {
                new_y.max(0.0)
            };
        }
    }
}

pub(crate) fn scroll_view_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scroll_query: Query<(Entity, &Interaction, &mut ScrollViewState), With<ScrollViewArea>>,
    parent_query: Query<(&ChildOf, Option<&Interaction>)>,
) {
    for (entity, interaction, mut state) in &mut scroll_query {
        let is_hovered = *interaction == Interaction::Hovered
            || *interaction == Interaction::Pressed
            || parent_query
                .get(entity)
                .ok()
                .and_then(|(parent, _)| parent_query.get(parent.0).ok())
                .and_then(|(_, parent_interaction)| parent_interaction)
                .is_some_and(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);

        if !is_hovered {
            continue;
        }

        let max_scroll_y = (state.content_size.y - state.viewport_size.y).max(0.0);

        let step = state.scroll_speed * 2.0;

        let page_step = if state.viewport_size.y > 0.0 {
            state.viewport_size.y * 0.8
        } else {
            100.0
        };

        let apply_clamp = |val: f32| -> f32 {
            if max_scroll_y > 0.0 {
                val.clamp(0.0, max_scroll_y)
            } else {
                val.max(0.0)
            }
        };

        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            state.target_offset.y = apply_clamp(state.target_offset.y - step);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            state.target_offset.y = apply_clamp(state.target_offset.y + step);
        }

        if keyboard_input.just_pressed(KeyCode::PageUp) {
            state.target_offset.y = apply_clamp(state.target_offset.y - page_step);
        }

        if keyboard_input.just_pressed(KeyCode::PageDown) {
            state.target_offset.y = apply_clamp(state.target_offset.y + page_step);
        }

        if keyboard_input.just_pressed(KeyCode::Home) {
            state.target_offset.y = 0.0;
        }

        if keyboard_input.just_pressed(KeyCode::End) && max_scroll_y > 0.0 {
            state.target_offset.y = max_scroll_y;
        }
    }
}

pub(crate) fn apply_scroll_physics(
    time: Res<Time>,
    mut scroll_query: Query<(&mut ScrollViewState, &mut ScrollPosition), With<ScrollViewArea>>,
) {
    let delta_time = time.delta_secs();

    for (mut state, mut scroll_pos) in &mut scroll_query {
        let actual_pos = Vec2::new(scroll_pos.x, scroll_pos.y);

        // Detect direct updates from native scrollbar dragging
        if (actual_pos - state.current_offset).length_squared() > 0.001
            && (actual_pos - state.target_offset).length_squared() > 0.001
        {
            state.current_offset = actual_pos;
            state.target_offset = actual_pos;
        } else {
            // Apply LERP smoothing towards mouse/keyboard scroll target
            let decay = (1.0 - (-state.smoothing * delta_time).exp()).clamp(0.0, 1.0);
            state.current_offset = state.current_offset.lerp(state.target_offset, decay);

            if state.current_offset.distance(state.target_offset) < 0.05 {
                state.current_offset = state.target_offset;
            }

            scroll_pos.x = state.current_offset.x;
            scroll_pos.y = state.current_offset.y;
        }
    }
}

pub(crate) fn update_visuals(
    mut q_thumb: Query<
        (&mut BackgroundColor, &Hovered, Option<&ScrollbarDragState>),
        (
            With<ScrollViewThumb>,
            Or<(Changed<Hovered>, Changed<ScrollbarDragState>)>,
        ),
    >,
) {
    for (mut thumb_bg, Hovered(is_hovering), drag) in q_thumb.iter_mut() {
        let is_dragging = drag.is_some_and(|d| d.dragging);
        let color = if is_dragging {
            Color::srgb(0.9, 0.9, 0.9)
        } else if *is_hovering {
            Color::srgb(0.7, 0.7, 0.7)
        } else {
            Color::srgb(0.4, 0.4, 0.4)
        };

        if thumb_bg.0 != color {
            thumb_bg.0 = color;
        }
    }
}

pub(crate) fn update_props(
    root_query: Query<(&Properties<ScrollViewProps>, &Children)>,
    children_query: Query<&Children>,
    mut area_query: Query<
        (
            &Interaction,
            &mut Node,
            &mut BackgroundColor,
            &mut ScrollViewState,
        ),
        (With<ScrollViewArea>, Without<ScrollViewContent>),
    >,
    mut content_query: Query<&mut Node, (With<ScrollViewContent>, Without<ScrollViewArea>)>,
    mut data_query: Query<&mut ScrollViewData>,
    thumb_query: Query<&Hovered, With<ScrollViewThumb>>,
    changed_roots: Query<Entity, Changed<Properties<ScrollViewProps>>>,
    changed_areas: Query<&ChildOf, (With<ScrollViewArea>, Changed<Interaction>)>,
    changed_thumbs: Query<&ChildOf, (With<ScrollViewThumb>, Changed<Hovered>)>,
) {
    let mut roots_to_update = FxHashSet::default();

    for entity in &changed_roots {
        roots_to_update.insert(entity);
    }

    for child_of in &changed_areas {
        roots_to_update.insert(child_of.0);
    }

    for child_of in &changed_thumbs {
        if children_query.get(child_of.0).is_ok() {
            roots_to_update.insert(child_of.0);
        }
    }

    for root_entity in roots_to_update {
        let Ok((props, _)) = root_query.get(root_entity) else {
            continue;
        };

        let mut is_pressed = false;
        let mut is_hovered = false;

        for descendant in children_query.iter_descendants(root_entity) {
            if let Ok((interaction, _, _, _)) = area_query.get(descendant) {
                if *interaction == Interaction::Pressed {
                    is_pressed = true;
                } else if *interaction == Interaction::Hovered {
                    is_hovered = true;
                }
            }
            if let Ok(hovered) = thumb_query.get(descendant)
                && hovered.0
            {
                is_hovered = true;
            }
        }

        let active_props = if is_pressed {
            &props.click
        } else if is_hovered {
            &props.hover
        } else {
            &props.default
        };

        if let Ok(mut data) = data_query.get_mut(root_entity) {
            crate::set_if_changed!(
                data.direction, active_props.direction;
                data.scroll_speed, active_props.scroll_speed;
                data.smooth, active_props.smooth;
            );
        }

        for descendant in children_query.iter_descendants(root_entity) {
            if let Ok((_, mut area_node, mut area_bg, mut state)) = area_query.get_mut(descendant) {
                crate::set_if_changed!(
                    area_bg.0, active_props.bg_color;
                    area_node.overflow, active_props.direction.to_overflow();
                    state.scroll_speed, active_props.scroll_speed;
                );

                if state.smoothing != 18.0 && active_props.smooth {
                    state.smoothing = 18.0;
                }
            }

            if let Ok(mut content_node) = content_query.get_mut(descendant) {
                content_node.flex_direction = match active_props.direction {
                    ScrollDirection::Horizontal => FlexDirection::Row,
                    _ => FlexDirection::Column,
                };
            }
        }
    }
}

/// The direction of the scroll view.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ScrollDirection {
    /// The view scrolls vertically.
    #[default]
    Vertical,
    /// The view scrolls horizontally.
    Horizontal,
    /// The view can be scrolled in both directions.
    Both,
}

impl ScrollDirection {
    /// Parses a scroll direction from a string.
    #[inline(always)]
    pub fn parse(s: &str) -> Result<Self, String> {
        parse_matches(
            s,
            &[
                ("vertical", || Ok(Self::Vertical)),
                ("horizontal", || Ok(Self::Horizontal)),
                ("both", || Ok(Self::Both)),
            ],
        )
    }

    /// Converts the scroll direction to bevy's [Overflow].
    #[inline(always)]
    pub fn to_overflow(self) -> Overflow {
        match self {
            Self::Vertical => Overflow {
                x: OverflowAxis::Clip,
                y: OverflowAxis::Scroll,
            },
            Self::Horizontal => Overflow {
                x: OverflowAxis::Scroll,
                y: OverflowAxis::Clip,
            },
            Self::Both => Overflow::scroll(),
        }
    }
}

/// Scroll view data attached to the scroll view root.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct ScrollViewData {
    /// The scroll direction.
    pub direction: ScrollDirection,
    /// The scroll speed.
    pub scroll_speed: f32,
    /// Whether to use smooth scrolling.
    pub smooth: bool,
}

/// Scroll view state attached to [ScrollViewArea].
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ScrollViewState {
    /// Current rendered scroll offset in pixels.
    pub current_offset: Vec2,
    /// Desired target offset (used for smooth LERP transitions).
    pub target_offset: Vec2,
    /// Viewport size in pixels.
    pub viewport_size: Vec2,
    /// Actual content size in pixels.
    pub content_size: Vec2,
    /// Scroll speed/sensitivity.
    pub scroll_speed: f32,
    /// Lerp factor for smooth scrolling (higher = snappier, lower = smoother).
    pub smoothing: f32,
}

impl Default for ScrollViewState {
    #[inline(always)]
    fn default() -> Self {
        Self {
            current_offset: Vec2::ZERO,
            target_offset: Vec2::ZERO,
            viewport_size: Vec2::ZERO,
            content_size: Vec2::ZERO,
            scroll_speed: 35.0,
            smoothing: 18.0,
        }
    }
}

/// Marker component for the container holding the scrollable inner elements.
#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ScrollViewContent;

/// Marker component for the scroll area container node.
#[derive(Component, Debug, Clone, Copy)]
pub struct ScrollViewArea;

/// Marker component identifying scrollbar thumb components.
#[derive(Component, Debug, Clone, Copy)]
pub struct ScrollViewThumb;

/// A scroll view widget that allows users to scroll through overflowing content.
///
/// ## XML Usage
///
/// Build a scroll view using the `<ScrollView></ScrollView>` tag.
///
/// Insert as many children as you want.
///
/// ### Attributes
/// - `scroll-direction = "<vertical|horizontal|both>"`: The scroll direction. See [ScrollDirection].
/// - `scroll-speed = "<float>"`: The scroll speed.
/// - `color = "<color>"`: The background color of the scroll view container.
/// - `smooth = "<bool>"`: Whether to use smooth scrolling.
///
/// All the attributes support state overrides.
///
/// ## Logic
///
/// Use the [ScrollViewProps] to control the scroll view.
/// You may also use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct ScrollViewWidget {
    props: Properties<ScrollViewProps>,
}

impl Widget for ScrollViewWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "ScrollView"
    }

    fn parse(&mut self, node: &XmlNode) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = ScrollViewProps::parse(node, None, &ScrollViewProps::default())?;
        let hover = ScrollViewProps::parse(node, Some("hover"), &default)?;
        let click = ScrollViewProps::parse(node, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, _assets: &AssetServer) -> Entity {
        let props = &self.props.default;

        commands.insert((
            self.props.clone(),
            ScrollViewData {
                direction: props.direction,
                scroll_speed: props.scroll_speed,
                smooth: props.smooth,
            },
            Node {
                display: Display::Grid,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                grid_template_columns: vec![
                    RepeatedGridTrack::flex(1, 1.0),
                    RepeatedGridTrack::auto(1),
                ],
                grid_template_rows: vec![
                    RepeatedGridTrack::flex(1, 1.0),
                    RepeatedGridTrack::auto(1),
                ],
                row_gap: Val::Px(2.0),
                column_gap: Val::Px(2.0),
                ..default()
            },
        ));

        let mut content_entity = commands.id();

        commands.with_children(|parent| {
            let scroll_area_id = parent
                .spawn((
                    Transform::default(),
                    GlobalTransform::default(),
                    ScrollViewArea,
                    Interaction::None,
                    ScrollViewState {
                        scroll_speed: props.scroll_speed,
                        ..default()
                    },
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        overflow: props.direction.to_overflow(),
                        grid_row: GridPlacement::start(1),
                        grid_column: GridPlacement::start(1),
                        ..default()
                    },
                    BackgroundColor(props.bg_color),
                    ScrollPosition::default(),
                ))
                .with_children(|area_parent| {
                    content_entity = area_parent
                        .spawn((
                            Transform::default(),
                            GlobalTransform::default(),
                            ScrollViewContent,
                            Node {
                                display: Display::Flex,
                                flex_direction: match props.direction {
                                    ScrollDirection::Horizontal => FlexDirection::Row,
                                    _ => FlexDirection::Column,
                                },
                                min_width: Val::Percent(100.0),
                                min_height: Val::Percent(100.0),
                                ..default()
                            },
                        ))
                        .id();
                })
                .id();

            if matches!(
                props.direction,
                ScrollDirection::Vertical | ScrollDirection::Both
            ) {
                parent.spawn((
                    Transform::default(),
                    GlobalTransform::default(),
                    Node {
                        min_width: Val::Px(8.0),
                        grid_row: GridPlacement::start(1),
                        grid_column: GridPlacement::start(2),
                        ..default()
                    },
                    Scrollbar {
                        orientation: ControlOrientation::Vertical,
                        target: scroll_area_id,
                        min_thumb_length: 3.5,
                    },
                    Children::spawn(Spawn((
                        ScrollViewThumb,
                        Hovered::default(),
                        BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                        BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                        ScrollbarThumb {
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            border: UiRect::all(Val::Px(1.0)),
                        },
                    ))),
                ));
            }

            if matches!(
                props.direction,
                ScrollDirection::Horizontal | ScrollDirection::Both
            ) {
                parent.spawn((
                    Transform::default(),
                    GlobalTransform::default(),
                    Node {
                        min_height: Val::Px(8.0),
                        grid_row: GridPlacement::start(2),
                        grid_column: GridPlacement::start(1),
                        ..default()
                    },
                    Scrollbar {
                        orientation: ControlOrientation::Horizontal,
                        target: scroll_area_id,
                        min_thumb_length: 3.5,
                    },
                    Children::spawn(Spawn((
                        ScrollViewThumb,
                        Hovered::default(),
                        BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                        BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                        ScrollbarThumb {
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            border: UiRect::all(Val::Px(1.0)),
                        },
                    ))),
                ));
            }
        });

        content_entity
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

            "width" => default.node.width = Val::Percent(100.0),
            "hover.width" => hover.node.width = default.node.width,
            "click.width" => click.node.width = default.node.width,

            "height" => default.node.height = Val::Percent(100.0),
            "hover.height" => hover.node.height = default.node.height,
            "click.height" => click.node.height = default.node.height,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties of a [ScrollViewWidget].
#[derive(Clone, Debug)]
pub struct ScrollViewProps {
    /// The direction of the scroll view.
    pub direction: ScrollDirection,
    /// The scroll speed.
    pub scroll_speed: f32,
    /// The background color of the scroll view container.
    pub bg_color: Color,
    /// Whether to use smooth scrolling.
    pub smooth: bool,
}

impl ScrollViewProps {
    fn parse(
        node: &XmlNode,
        prefix: Option<&str>,
        base: &Self,
    ) -> std::result::Result<Self, String> {
        let direction = parse_attribute(node, "scroll-direction", prefix, ScrollDirection::parse)?
            .unwrap_or(base.direction);

        let scroll_speed = parse_attribute(node, "scroll-speed", prefix, parse_float)?
            .unwrap_or(base.scroll_speed);

        let bg_color =
            parse_attribute(node, "color", prefix, parse_color)?.unwrap_or(base.bg_color);

        let smooth = parse_attribute(node, "smooth", prefix, parse_bool)?.unwrap_or(base.smooth);

        Ok(Self {
            direction,
            scroll_speed,
            bg_color,
            smooth,
        })
    }
}

impl Default for ScrollViewProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            direction: ScrollDirection::default(),
            scroll_speed: 30.0,
            bg_color: Color::NONE,
            smooth: true,
        }
    }
}
