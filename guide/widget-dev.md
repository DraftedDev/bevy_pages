# Widget Development

You can create your very own widgets using the `Widget` trait.

If you want inspiration, you can always look at the source code of common widgets like `TextWidget` or `CheckboxWidget`.

## Understanding `Widget`

The `Widget` trait looks like this:

```rust
/// A trait to define a widget.
pub trait Widget: Debug + Send + Sync + 'static {
    /// The name of the widget.
    ///
    /// This should also be its XML tag name.
    fn name() -> &'static str
    where
        Self: Sized;

    /// Set up the widget logic.
    ///
    /// This is where widgets should add systems and observers to the bevy app.
    fn setup(&self, app: &mut App);

    /// Parses the widget from an XML node.
    fn parse(&mut self, node: &Node) -> Result<(), String>;

    /// Spawns the widget. Called inside [Element::spawn](crate::element::Element::spawn).
    fn spawn(&self, entity: Entity, world: &mut World) -> Entity;
    //                                     ^^^^^^^^^^^ 
    // Notice the `&mut World` here. This gives your widget complete access to the bevy world, so you have full control.

    /// Apply this widget's default properties.
    fn apply_defaults(
        &self,
        node: &Node,
        default: &mut ElementProps,
        hover: &mut ElementProps,
        click: &mut ElementProps,
    );

    /// Creates a dynamic clone of the widget.
    fn dyn_clone(&self) -> Box<dyn Widget>;
}
```

Widgets are stored as `Box<dyn Widget>`, so they mostly need to be object-safe (no `Sized`, no generics, only `&self`
methods). `name() -> &'static str` is an exception here, since it's not called on `Box<dyn Widget>`.

The widget also needs to implement `Default` (the requirement is enforced in `PagesPlugin::with_widget`).

All the methods of a widget are called in this particular order:

1. `name`: Right after calling `PagesPlugin::with_widget` to get a unique type identifier.
2. (`default`: Right after calling `PagesPlugin::with_widget` to get a default widget instance).
3. `setup`: When the `PagesPlugin` initializes widgets.
4. `dyn_clone`: To create a dynamic clone of the widget that will actually be spawned.
5. `parse`: When the widget is parsed from an XML node.
6. `apply_defaults`: To apply default properties to element props.
7. `spawn`: To spawn the actual widget.

The cloned instance of the `Widget` is dropped after spawning.

## Example `Widget`

To learn more about how to implement a `Widget`, we can take a look at the `CheckboxWidget`:

```rust
// Observer function for checkbox state changes.
fn toggle_checkbox(
    trigger: On<ElementClick>,
    mut commands: Commands,
    mut query: Query<(&mut CheckboxState, Option<&ElementId>)>,
) {
    // Check if the clicked entity is even a checkbox.
    if let Ok((mut state, id)) = query.get_mut(trigger.entity) {
        // Set the new state.
        state.0 = !state.0;

        // Trigger a toggle event.
        commands.trigger(ElementToggle {
            entity: trigger.entity,
            id: id.cloned(),
            state: state.0,
        });
    }
}

// Synchronize checkbox visuals with state.
fn sync_visuals(
    query: Query<(&CheckboxState, &Children), Changed<CheckboxState>>,
    mut checkmarks: Query<&mut Node, With<CheckboxCheckmark>>,
) {
    for (state, children) in &query {
        for child in children.iter() {
            // If the child is a checkbox checkmark (is checked), update its display to `Flex`.
            // Otherwise, set it to `None`, which makes the "checkmark" invisible.
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

// Update changed widget properties.
// Every widget with custom properties should have this system.
fn update_props(
    mut query: Query<
        (&Interaction, &Properties<CheckboxProps>, &Children),
        Or<(Changed<Interaction>, Changed<Properties<CheckboxProps>>)>,
    >,
    mut checkmark_query: Query<(&mut Text, &mut TextColor), With<CheckboxCheckmark>>,
) {
    for (interaction, props, children) in &mut query {
        // Get the active properties based on the interaction state.
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        for child in children.iter() {
            if let Ok((mut text, mut color)) = checkmark_query.get_mut(*child) {
                // This macro sets the original properties to the new values only if original != new.
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
    // Store internal properties.
    props: Properties<CheckboxProps>,
}

impl Widget for CheckboxWidget {
    // Translates to `<Checkbox/>` in XML.
    // You could also technically use `<Checkbox>...</Checkbox>`, but most people don't want children INSIDE a checkbox.
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Checkbox"
    }

    // Add our basic systems and the toggle event observer.
    fn setup(&self, app: &mut App) {
        app.add_systems(Update, (update_props, sync_visuals.in_set(PageSystemSet)))
            .add_observer(toggle_checkbox);
    }

    fn parse(&mut self, _: &XmlNode, attrs: AttributeMap) -> Result<(), String>
    where
        Self: Sized,
    {
        let default = CheckboxProps::parse(&attrs, None, &CheckboxProps::default())?;
        let hover = CheckboxProps::parse(&attrs, Some("hover"), &default)?;
        let click = CheckboxProps::parse(&attrs, Some("click"), &default)?;

        self.props = Properties {
            default,
            hover,
            click,
        };

        Ok(())
    }

    // Spawn the checkbox and its checkmark.
    fn spawn(&self, entity: Entity, world: &mut World) -> Entity {
        let props = &self.props.default;

        world
            .entity_mut(entity)
            .insert((self.props.clone(), CheckboxState(props.state)));

        // !!! IMPORTANT !!!
        // Direct world access requires you to do most things (like add the `ChildOf` component for children) MANUALLY!!!
        world.spawn((
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
            // Make the checkmark entity a child of the root checkbox entity.
            ChildOf(entity),
        ));

        entity
    }

    // Apply default properties to the checkbox.
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

        // This macro only sets the attributes if they are missing,
        // so when specifying something like `width="10px"`, the default doesn't override it.
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

    // Simply clone the widget to a `Box<dyn Widget>`.
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
    fn parse(attrs: &AttributeMap, prefix: Option<&str>, base: &Self) -> Result<Self, String> {
        // Parse `checked` or use the base value.
        let state = parse_attribute(attrs, "checked", prefix, parse_bool)?.unwrap_or(base.state);

        // Parse `check-color` or use the base value.
        let check_color =
            parse_attribute(attrs, "check-color", prefix, parse_color)?.unwrap_or(base.check_color);

        // Parse `check-symbol` or use the base value.
        let symbol = parse_attribute(attrs, "check-symbol", prefix, |s| Ok(s.to_string()))?
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
            // Default symbol is "x".
            // We don't default to "✓" or some fancy Unicodde character because it's not supported by the default bevy font (as of the time of writing).
            symbol: "x".to_string(),
        }
    }
}
```
