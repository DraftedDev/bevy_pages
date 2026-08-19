use crate::element::{ElementId, ElementProps};
use crate::events::ElementSet;
use crate::parser::color::parse_color;
use crate::parser::values::parse_attribute;
use crate::props::Properties;
use crate::widgets::Widget;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::prelude::*;
use roxmltree::Node as XmlNode;

pub(crate) fn trigger_menu(
    mut trigger_query: Query<
        (&Interaction, &ChildOf),
        (With<DropdownTrigger>, Changed<Interaction>),
    >,
    mut root_query: Query<&mut DropdownState>,
) {
    for (interaction, child_of) in &mut trigger_query {
        if *interaction == Interaction::Pressed
            && let Ok(mut state) = root_query.get_mut(child_of.0)
        {
            state.is_open = !state.is_open;
        }
    }
}

pub(crate) fn option_select(
    mut commands: Commands,
    interaction_query: Query<(Entity, &Interaction), Changed<Interaction>>,
    child_of_query: Query<&ChildOf>,
    menu_query: Query<&ChildOf, With<DropdownMenu>>,
    mut root_query: Query<(Entity, &mut DropdownState, &Children, Option<&ElementId>)>,
    trigger_query: Query<&Children, With<DropdownTrigger>>,
    label_query: Query<Entity, With<DropdownLabelText>>,
    mut text_query: Query<&mut Text>,
    children_query: Query<&Children>,
) {
    for (clicked_entity, interaction) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(root_entity) = find_dropdown_root(clicked_entity, &child_of_query, &menu_query)
            && let Ok((entity, mut state, root_children, element_id)) =
                root_query.get_mut(root_entity)
        {
            // Extract label text from the clicked entity or its child subtree
            let extracted_label =
                extract_text_from_entity(clicked_entity, &text_query, &children_query)
                    .unwrap_or_else(|| "Selected Option".to_string());

            state.selected_value = extracted_label.clone();
            state.selected_label = extracted_label.clone();
            state.is_open = false; // Auto-close menu on selection

            commands.trigger(ElementSet {
                entity,
                id: element_id.cloned(),
                value: extracted_label.clone(),
                delta: None,
            });

            // Sync the trigger header label text
            for child in root_children.iter() {
                if let Ok(trigger_children) = trigger_query.get(child) {
                    for trigger_child in trigger_children.iter() {
                        if label_query.contains(trigger_child)
                            && let Ok(mut text) = text_query.get_mut(trigger_child)
                        {
                            **text = extracted_label.clone();
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn visibility(
    root_query: Query<(&DropdownState, &Children), Changed<DropdownState>>,
    mut menu_query: Query<(&mut Node, &mut ZIndex), With<DropdownMenu>>,
) {
    for (state, children) in &root_query {
        for child in children.iter() {
            if let Ok((mut node, mut z_index)) = menu_query.get_mut(child) {
                if state.is_open {
                    node.display = Display::Flex;
                    *z_index = ZIndex(100);
                } else {
                    node.display = Display::None;
                    *z_index = ZIndex(0);
                }
            }
        }
    }
}

pub(crate) fn close_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut root_query: Query<(Entity, &mut DropdownState)>,
    interaction_query: Query<&Interaction>,
    children_query: Query<&Children>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    for (root_entity, mut state) in &mut root_query {
        if !state.is_open {
            continue;
        }

        // Check if any entity within this specific dropdown's tree is hovered or pressed
        let is_interacting =
            is_hierarchy_interacting(root_entity, &interaction_query, &children_query);

        if !is_interacting {
            state.is_open = false;
        }
    }
}

pub(crate) fn update_props(
    root_query: Query<(&Properties<DropdownProps>, &DropdownState, &Children)>,
    mut trigger_query: Query<
        (&Interaction, &Children, &mut BackgroundColor),
        With<DropdownTrigger>,
    >,
    mut menu_query: Query<&mut BackgroundColor, (With<DropdownMenu>, Without<DropdownTrigger>)>,
    mut label_query: Query<&mut Text, With<DropdownLabelText>>,
    changed_roots: Query<Entity, Or<(Changed<Properties<DropdownProps>>, Changed<DropdownState>)>>,
    changed_triggers: Query<&ChildOf, (With<DropdownTrigger>, Changed<Interaction>)>,
) {
    let mut roots_to_update = std::collections::HashSet::new();

    for entity in &changed_roots {
        roots_to_update.insert(entity);
    }

    for child_of in &changed_triggers {
        roots_to_update.insert(child_of.0);
    }

    for root_entity in roots_to_update {
        let Ok((props, state, root_children)) = root_query.get(root_entity) else {
            continue;
        };

        let mut active_interaction = Interaction::None;
        for child in root_children.iter() {
            if let Ok((interaction, _, _)) = trigger_query.get(child) {
                active_interaction = *interaction;
                break;
            }
        }

        let active_props = match active_interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        for child in root_children.iter() {
            if let Ok((_, trigger_children, mut trigger_bg)) = trigger_query.get_mut(child) {
                crate::set_if_changed!(trigger_bg.0, active_props.bg_color);

                if state.selected_index.is_none() && state.selected_value.is_empty() {
                    for trigger_child in trigger_children.iter() {
                        if let Ok(mut text) = label_query.get_mut(trigger_child) {
                            crate::set_if_changed!(text.0, active_props.placeholder => active_props.placeholder.clone());
                        }
                    }
                }
            }

            if let Ok(mut menu_bg) = menu_query.get_mut(child) {
                crate::set_if_changed!(menu_bg.0, active_props.menu_bg_color);
            }
        }
    }
}

#[inline]
fn find_dropdown_root(
    start_entity: Entity,
    child_of_query: &Query<&ChildOf>,
    menu_query: &Query<&ChildOf, With<DropdownMenu>>,
) -> Option<Entity> {
    let mut current = start_entity;

    loop {
        // Check if current entity is the DropdownMenu container
        if let Ok(menu_child_of) = menu_query.get(current) {
            return Some(menu_child_of.0); // Returns root Dropdown entity
        }

        // Ascend to parent
        if let Ok(parent) = child_of_query.get(current) {
            current = parent.0;
        } else {
            return None;
        }
    }
}

#[inline]
fn extract_text_from_entity(
    entity: Entity,
    text_query: &Query<&mut Text>,
    children_query: &Query<&Children>,
) -> Option<String> {
    if let Ok(text) = text_query.get(entity)
        && !text.0.is_empty()
    {
        return Some(text.0.clone());
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            if let Some(found) = extract_text_from_entity(child, text_query, children_query) {
                return Some(found);
            }
        }
    }

    None
}

#[inline]
fn is_hierarchy_interacting(
    entity: Entity,
    interaction_query: &Query<&Interaction>,
    children_query: &Query<&Children>,
) -> bool {
    if let Ok(interaction) = interaction_query.get(entity)
        && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    {
        return true;
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            if is_hierarchy_interacting(child, interaction_query, children_query) {
                return true;
            }
        }
    }

    false
}

/// The internal state of a dropdown widget.
#[derive(Component, Debug, Clone)]
pub struct DropdownState {
    /// If the dropdown menu is open.
    pub is_open: bool,
    /// The selected item index.
    pub selected_index: Option<usize>,
    /// The selected value.
    pub selected_value: String,
    /// The selected label.
    pub selected_label: String,
}

impl Default for DropdownState {
    #[inline(always)]
    fn default() -> Self {
        Self {
            is_open: false,
            selected_index: None,
            selected_value: String::new(),
            selected_label: "Select an option...".to_string(),
        }
    }
}

/// Marker component for the header button.
#[derive(Component, Debug, Clone, Copy)]
pub struct DropdownTrigger;

/// Marker component for the overlay container holding the dropdown items.
#[derive(Component, Debug, Clone, Copy)]
pub struct DropdownMenu;

/// Marker for the dropdown label text.
#[derive(Component, Debug, Clone, Copy)]
pub struct DropdownLabelText;

/// A dropdown widget with a list of options.
///
/// ## XML Usage
///
/// Build a dropdown widget using the `<Dropdown></Dropdown>` tag.
///
/// Every "root" children of the dropdown is considered a new option.
///
/// ### Attributes
/// - `placeholder = "<string>"`: The placeholder text to display when no option is selected.
/// - `dropdown-bg-color = "<color>"`: The background color of the dropdown.
/// - `menu-bg-color = "<color>"`: The background color of the dropdown menu.
///
/// All the attributes listed support state overrides.
///
/// ## Logic
///
/// Use the [DropdownProps] to control the dropdown.
/// Furthermore, the dropdown widget triggers `ElementSet<String>` when an option is selected.
///
/// You can also use generic element events to implement custom behavior.
#[derive(Clone, Debug, Default)]
pub struct DropdownWidget {
    props: Properties<DropdownProps>,
}

impl Widget for DropdownWidget {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Dropdown"
    }

    fn parse(&mut self, node: &XmlNode) -> Result<(), String> {
        let default = DropdownProps::parse(node, None, &DropdownProps::default())?;
        let hover = DropdownProps::parse(node, Some("hover"), &default)?;
        let click = DropdownProps::parse(node, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    fn spawn(&self, commands: &mut EntityCommands, _assets: &AssetServer) -> Entity {
        let root_entity = commands.id();
        let props = &self.props.default;

        commands.insert((
            self.props.clone(),
            DropdownState {
                selected_label: props.placeholder.clone(),
                ..default()
            },
        ));

        let mut menu_entity = None;

        commands.with_children(|parent| {
            parent
                .spawn((
                    Transform::default(),
                    GlobalTransform::default(),
                    DropdownTrigger,
                    Interaction::None,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(36.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(props.bg_color),
                    BorderColor::all(Color::srgb(0.4, 0.4, 0.4)),
                ))
                .with_children(|trigger| {
                    trigger.spawn((
                        Transform::default(),
                        GlobalTransform::default(),
                        DropdownLabelText,
                        Text::new(&props.placeholder),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));

                    trigger.spawn((
                        Transform::default(),
                        GlobalTransform::default(),
                        Text::new(">"),
                        TextFont {
                            font_size: FontSize::Px(10.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                });

            let menu_cmd = parent.spawn((
                Transform::default(),
                GlobalTransform::default(),
                DropdownMenu,
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    max_height: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    margin: UiRect::top(Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                ZIndex(100),
                BackgroundColor(props.menu_bg_color),
                BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
            ));

            menu_entity = Some(menu_cmd.id());
        });

        menu_entity.unwrap_or(root_entity)
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
        );
    }

    fn dyn_clone(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// The properties of a [DropdownWidget].
#[derive(Clone, Debug)]
pub struct DropdownProps {
    /// The placeholder text to display when no option is selected.
    pub placeholder: String,
    /// The dropdown background color.
    pub bg_color: Color,
    /// The dropdown menu background color.
    pub menu_bg_color: Color,
}

impl DropdownProps {
    fn parse(node: &XmlNode, prefix: Option<&str>, base: &Self) -> Result<Self, String> {
        let placeholder = parse_attribute(node, "placeholder", prefix, |s| Ok(s.to_string()))?
            .unwrap_or_else(|| base.placeholder.clone());

        let bg_color = parse_attribute(node, "dropdown-bg-color", prefix, parse_color)?
            .unwrap_or(base.bg_color);

        let menu_bg_color = parse_attribute(node, "menu-bg-color", prefix, parse_color)?
            .unwrap_or(base.menu_bg_color);

        Ok(Self {
            placeholder,
            bg_color,
            menu_bg_color,
        })
    }
}

impl Default for DropdownProps {
    #[inline(always)]
    fn default() -> Self {
        Self {
            placeholder: String::default(),
            bg_color: Color::srgb(0.2, 0.2, 0.2),
            menu_bg_color: Color::srgb(0.15, 0.15, 0.15),
        }
    }
}
