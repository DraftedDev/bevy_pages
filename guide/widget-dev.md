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
    fn spawn(&self, commands: &mut EntityCommands, assets: &AssetServer) -> Entity;

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
// Observer to toggle the checkbox state.
fn toggle_checkbox(
    trigger: On<ElementClick>,
    mut commands: Commands,
    mut query: Query<(&mut CheckboxState, Option<&ElementId>)>,
) {
    if let Ok((mut state, id)) = query.get_mut(trigger.entity) {
        state.0 = !state.0;

        // Triggers custom toggle event.
        commands.trigger(ElementToggle {
            entity: trigger.entity,
            id: id.cloned(),
            state: state.0,
        });
    }
}

// Synchronize visuals based on the inner checkbox state.
fn sync_visuals(
    query: Query<(&CheckboxState, &Children), Changed<CheckboxState>>,
    mut checkmarks: Query<&mut Node, With<CheckboxCheckmark>>,
) {
    for (state, children) in &query {
        for child in children.iter() {
            if let Ok(mut node) = checkmarks.get_mut(*child) {
                node.display = if state.0 {
                    // Checkmark visible
                    Display::Flex
                } else {
                    // Checkmark not visible
                    Display::None
                };
            }
        }
    }
}

// Update widget properties: Needed to sync props with actual UI logic.
fn update_props(
    mut query: Query<
        (&Interaction, &Properties<CheckboxProps>, &Children),
        Or<(Changed<Interaction>, Changed<Properties<CheckboxProps>>)>,
    >,
    mut checkmark_query: Query<(&mut Text, &mut TextColor), With<CheckboxCheckmark>>,
) {
    for (interaction, props, children) in &mut query {
        // The active properties to apply.
        let active_props = match interaction {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => &props.hover,
            Interaction::None => &props.default,
        };

        for child in children.iter() {
            if let Ok((mut text, mut color)) = checkmark_query.get_mut(*child) {
                // We use a util macro (`bevy_pages::set_if_changed!`) to only update the properties if they have actually changed.
                crate::set_if_changed!(
                    // If `text.0` is not `active_props.symbol` => update `text.0` to `active_props.symbol`.
                    text.0, active_props.symbol => active_props.symbol.clone();
                    // If `color.0` is not `active_props.check_color` => update `color.0` to `active_props.check_color`.
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
        // XML: "<Checkbox/>".
        "Checkbox"
    }

    fn setup(&self, app: &mut App) {
        // Add systems and observer for interactivity.
        app.add_systems(Update, (update_props, sync_visuals.in_set(PageSystemSet)))
            .add_observer(toggle_checkbox);
    }

    fn parse(&mut self, node: &XmlNode) -> Result<(), String>
    where
        Self: Sized,
    {
        // Parse the different property overrides.
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
        // Always use default props for initial spawning.
        let props = &self.props.default;

        // Insert properties and internal state.
        commands.insert((self.props.clone(), CheckboxState(props.state)));

        // Spawn checkmark entity.
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

        // Some widgets like `ScrollViewWidget` return a custom root entity.
        // Most other widgets are pretty straightforward though and just return the root entity.
        commands.id()
    }

    // Apply default `ElementProps`
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

        // We use a util macro again for simplicity.
        // The macro only sets the attributes if `!node.has_attribute($attr)`.
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

    // Required for `dyn` cloning.
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
    // Parses the props from an XML node.
    fn parse(node: &XmlNode, prefix: Option<&str>, base: &Self) -> Result<Self, String> {
        // `parse_attribute` is a helper function that parses an attribute from an XML node and applies a parse function (in this case `parse_bool`).
        // You can also specify a prefix (e.g. "hover.checked" has the prefix "hover").
        // You should also fall back to the base value (`base.state` in this case) if the attribute is not set.
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
```
