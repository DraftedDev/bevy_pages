# Events

Every widget triggers events when you interact with it.

Events are usually structured like this:

```rust
struct EventName {
    /// The entity behind the target element.
    pub entity: Entity,
    /// The element ID of the target element.
    pub id: Option<ElementId>,
    // <...>
}
```

Most event types also have a methods called `matches_id` which you can use to identify the element behind an event by
its ID:

```rust
// <Node id="my-fancy-widget" />

fn observer(event: On<SomeEvent>) {
    if event.matches_id("my-fancy-widget") {
        println!("Something happened!");
    }
}
```

For more usage examples, see the [examples](../examples) directory.

## Generic Events

Some events (we call them "generic" events for simplicity) are widget-independent and are **always** called when
interacting with **any** widget.

- `ElementClick`
    - Triggered when an element is clicked.
- `ElementHover`
    - Triggered when an element is hovered on.
- `ElementSpawn`
    - Triggered when an element is spawned.

## Widget-specific Events

Some events are triggered by widget logic and are not supported by all widgets.

- `ElementToggle`
    - Triggered when a widget is toggled from `false` to `true` or vice versa.
    - Also contains the new state of the widget as `bool`.
    - Supported by: `Checkbox`, `Slider`.
- `ElementSet<T>`
    - Triggered when a widget's value is set.
    - Also contains the new value of the widget as `T`.
    - Supported by: `Slider (f32)`, `TextInput (String)`.
