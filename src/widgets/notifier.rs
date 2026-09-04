use crate::element::{ElementActive, ElementId, ElementProps};
use crate::parser::AttributeMap;
use crate::parser::color::parse_color;
use crate::parser::values::{parse_attribute, parse_float, parse_font_size, parse_int, parse_val};
use crate::props::Properties;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::app::App;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, UiRect, Val};
use roxmltree::Node as XmlNode;

fn handle_close(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &ChildOf),
        (Changed<Interaction>, With<NotificationCloseButton>),
    >,
) {
    for (interaction, parent) in &interaction_query {
        if *interaction == Interaction::Pressed {
            commands.entity(parent.0).despawn_children().despawn();
        }
    }
}

fn handle_notify(
    mut commands: Commands,
    mut messages: MessageReader<NotifyMessage>,
    notifiers: Query<
        (
            Entity,
            Option<&ElementId>,
            &Properties<NotifierProps>,
            Option<&Children>,
        ),
        With<NotifierContainer>,
    >,
) {
    for event in messages.read() {
        for (notifier_entity, id, props, children) in &notifiers {
            if let Some(target) = &event.target
                && let Some(id) = id
                && id != target
            {
                continue;
            }

            let active_props = &props.default;

            if let Some(children) = children
                && children.len() >= active_props.max
            {
                let overflow_count = (children.len() - active_props.max) + 1;
                for child in children.iter().take(overflow_count) {
                    commands.entity(child).despawn_children().despawn();
                }
            }

            let duration_secs = event.duration.unwrap_or(active_props.duration);

            let timer = if duration_secs > 0.0 {
                Some(Timer::from_seconds(duration_secs, TimerMode::Once))
            } else {
                None
            };

            let bg_color = event.notify_color.unwrap_or(active_props.notify_color);
            let color = event.color.unwrap_or(active_props.color);

            let card_entity = commands
                .spawn((
                    NotificationItem { timer },
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        column_gap: Val::Px(12.0),
                        min_width: Val::Px(200.0),
                        max_width: Val::Px(400.0),
                        width: Val::Auto,
                        ..default()
                    },
                    BackgroundColor(bg_color),
                    BorderColor::all(color),
                    FocusPolicy::Block,
                ))
                .set_parent_in_place(notifier_entity)
                .id();

            // Message Text
            commands
                .spawn((
                    NotificationText,
                    Text::new(&event.message),
                    TextFont {
                        font_size: active_props.font_size,
                        ..default()
                    },
                    TextColor(color),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                    FocusPolicy::Pass,
                ))
                .set_parent_in_place(card_entity);

            // Close Button ('x')
            commands
                .spawn((
                    NotificationCloseButton,
                    Button,
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    FocusPolicy::Block,
                    BorderColor::all(color.with_alpha(0.7)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("x"),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(color.with_alpha(0.7)),
                        FocusPolicy::Pass,
                    ));
                })
                .set_parent_in_place(card_entity);
        }
    }
}

fn tick_notifications(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut NotificationItem, Option<&Interaction>)>,
) {
    for (entity, mut item, interaction) in &mut query {
        if let Some(Interaction::Hovered) = interaction {
            continue;
        }

        if let Some(timer) = &mut item.timer {
            timer.tick(time.delta());
            if timer.is_finished() {
                commands.entity(entity).despawn_children().despawn();
            }
        }
    }
}

fn update_props(
    mut query: Query<
        (&Interaction, &Properties<NotifierProps>, &mut Node),
        (
            With<NotifierContainer>,
            Or<(
                With<ElementActive>,
                Changed<Interaction>,
                Changed<Properties<NotifierProps>>,
            )>,
        ),
    >,
) {
    for (interaction, props, mut node) in &mut query {
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        crate::set_if_changed!(node.row_gap, active_props.gap);
    }
}

/// A notification message.
///
/// Sent via [Commands::write_message] and handled by a [NotifierWidget].
#[derive(Debug, Clone, Default, Message)]
pub struct NotifyMessage {
    /// The optional [ElementId] of the target [NotifierWidget].
    ///
    /// When set, the notification can only be processed by the [NotifierWidget] with the given ID.
    pub target: Option<ElementId>,
    /// The message of the notification.
    pub message: String,
    /// The notification card background color.
    ///
    /// Defaults to [NotifierProps]'s `notify_color` field.
    pub notify_color: Option<Color>,
    /// The notification card foreground color.
    ///
    /// Defaults to [NotifierProps]'s `color` field.
    pub color: Option<Color>,
    /// The duration of the notification in seconds.
    ///
    /// This will determine how long the notification will be displayed.
    ///
    /// Defaults to [NotifierProps]'s `duration` field.
    pub duration: Option<f32>,
}

impl NotifyMessage {
    /// Creates a new [NotifyMessage] with the given message.
    pub fn new(msg: impl ToString) -> Self {
        Self {
            target: None,
            message: msg.to_string(),
            notify_color: None,
            color: None,
            duration: None,
        }
    }
}

/// A notification item.
#[derive(Component, Debug)]
pub struct NotificationItem {
    /// An optional timer for the notification.
    pub timer: Option<Timer>,
}

/// A marker component for the notifier container.
#[derive(Component, Debug, Clone)]
pub struct NotifierContainer;

/// A marker component for the notification text.
#[derive(Component, Debug, Copy, Clone)]
pub struct NotificationText;

/// A marker component for the notification close button.
#[derive(Component, Debug, Copy, Clone)]
pub struct NotificationCloseButton;

/// A notification area where [NotifyMessage]s are displayed.
///
/// ## XML Usage
///
/// Build a new notifier widget using the `<Notifier/>` tag.
///
/// The widget logic will automatically handle the display of notifications.
///
/// ### Attributes
/// - `max = "<int>"`: The maximum amount of notifications to display.
/// - `duration = "<float>"`: The default duration of notifications in seconds.
/// - `notify-color = "<color>"`: The default background color of notifications.
/// - `color = "<color>"`: The default foreground color of notifications.
/// - `font-size = "<fontSize>"`: The default font size of notifications.
/// - `gap = "<size>"`: The default gap between notifications.
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// Use [NotifierProps] to control the notification props.
/// Send notifications by writing a [NotifyMessage] via [Commands::write_message].
///
/// You may also use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct NotifierWidget {
    props: Properties<NotifierProps>,
}

impl Widget for NotifierWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Notifier"
    }

    fn setup(&self, app: &mut App) {
        app.add_message::<NotifyMessage>().add_systems(
            Update,
            (
                handle_notify,
                tick_notifications,
                handle_close,
                update_props,
            )
                .in_set(PageSystemSet),
        );
    }

    fn parse(&mut self, _: &XmlNode, attrs: &AttributeMap) -> Result<(), String> {
        let default = NotifierProps::parse(attrs, None, &NotifierProps::default())?;
        let hover = NotifierProps::parse(attrs, Some("hover"), &default)?;
        let click = NotifierProps::parse(attrs, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, entity: Entity, world: &mut World) -> Entity {
        let props = &self.props.default;

        let container_node = Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: props.gap,
            ..default()
        };

        world.entity_mut(entity).insert((
            self.props.clone(),
            NotifierContainer,
            container_node,
            FocusPolicy::Pass,
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
            "display" => default.node.display = Display::Flex,
            "hover.display" => hover.node.display = default.node.display,
            "click.display" => click.node.display = default.node.display,

            "flex-direction" => default.node.flex_direction = FlexDirection::Column,
            "hover.flex-direction" => hover.node.flex_direction = default.node.flex_direction,
            "click.flex-direction" => click.node.flex_direction = default.node.flex_direction,
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties for a [NotifierWidget] widget.
#[derive(Clone, Debug)]
pub struct NotifierProps {
    /// The maximum amount of notifications shown.
    pub max: usize,
    /// The default duration of a notification in seconds.
    pub duration: f32,
    /// The notification card background color.
    pub notify_color: Color,
    /// The foreground (including text) color.
    pub color: Color,
    /// The notification text font size.
    pub font_size: FontSize,
    /// The gap between notifications.
    pub gap: Val,
}

impl NotifierProps {
    fn parse(attrs: &AttributeMap, prefix: Option<&str>, base: &Self) -> Result<Self, String> {
        let max = parse_attribute(attrs, "max", prefix, parse_int)?
            .map(|i| i as usize)
            .unwrap_or(base.max);

        let duration =
            parse_attribute(attrs, "duration", prefix, parse_float)?.unwrap_or(base.duration);

        let notify_color = parse_attribute(attrs, "notify-color", prefix, parse_color)?
            .unwrap_or(base.notify_color);

        let color = parse_attribute(attrs, "color", prefix, parse_color)?.unwrap_or(base.color);

        let font_size =
            parse_attribute(attrs, "font-size", prefix, parse_font_size)?.unwrap_or(base.font_size);

        let gap = parse_attribute(attrs, "gap", prefix, parse_val)?.unwrap_or(base.gap);

        Ok(Self {
            max,
            duration,
            notify_color,
            color,
            font_size,
            gap,
        })
    }
}

impl Default for NotifierProps {
    fn default() -> Self {
        Self {
            max: 5,
            duration: 4.0,
            notify_color: Color::srgb(0.15, 0.15, 0.18),
            color: Color::WHITE,
            font_size: FontSize::Px(14.0),
            gap: Val::Px(8.0),
        }
    }
}
