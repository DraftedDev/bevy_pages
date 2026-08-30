# Quick Start

**This tutorial shows how to get started with `bevy_pages` by creating a simple counter app.**

## Setting everything up

Add `bevy_pages` to your project via `cargo add bevy_pages` or simply add this line to your `Cargo.toml`:

```toml
[dependencies]
bevy_pages = "*" # Or whatever version you want to use
```

To actually make `bevy_pages` usable, you need to add the `PagesPlugin`:

```rust
use bevy_pages::PagesPlugin;

fn main() {
    // `let app = App::new();`

    // Setup your app here

    app.add_plugins(PagesPlugin::default());

    app.run();
}
```

## Creating the XML

It's recommended to start your XML page with:

```xml
<?xml version="1.0" encoding="utf-8"?>
<Page
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="https://raw.githubusercontent.com/DraftedDev/bevy_pages/master/schema.xsd">
    <!-- Your page content here -->
</Page>
```

If you get errors regarding the `schema.xsd` file, you can download it
from [here](https://raw.githubusercontent.com/DraftedDev/bevy_pages/master/schema.xsd) and add it to your project. Make
sure to make your XML point to the location of the `schema.xsd` file if you choose this way.

You can create UI nodes just like you would in HTML. For our counter app we will use following built-in elements:

- `<Text></Text>` for text display
- `<Button></Button>` for user interaction
- `<Node></Node>` for the layout

```xml
<?xml version="1.0" encoding="utf-8"?>
<Page
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="https://raw.githubusercontent.com/DraftedDev/bevy_pages/master/schema.xsd"
>
    <Node>
        <Text>
            Counter
        </Text>

        <Text>
            0
        </Text>

        <Node>
            <Button>
                <Text>-</Text>
            </Button>

            <Button>
                <Text>
                    Reset
                </Text>
            </Button>

            <Button>
                <Text>+</Text>
            </Button>
        </Node>
    </Node>
</Page>
```

## Styling

Every node, including the `Page` node support common bevy styling via attributes. Different elements can also have
different attributes.

Also note the `id` attribute which uniquely identifies an element.

You may also use extra `<Style/>` elements to define style presets which can be applied using the `styles` attribute of
elements.

The resulting XML page becomes:

```xml
<?xml version="1.0" encoding="utf-8"?>
<Page
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="../schema.xsd"
        justify-content="center"
        align-items="center"
>
    <Styles>
        <Style name="centerNode" justify-content="center" align-items="center" padding="10px"/>
    </Styles>

    <Node
            styles="centerNode"
            flex-direction="column"
            row-gap="24px"
            padding="32px"
    >
        <Text
                font-size="36px"
                color="white"
        >
            Counter
        </Text>

        <Text
                font-size="72px"
                color="#4CAF50"
                id="counter"
        >
            0
        </Text>

        <Node
                styles="centerNode"
                flex-direction="row"
                column-gap="16px"
        >
            <Button
                    styles="centerNode"
                    id="decrement"
                    width="64px"
                    height="64px"
            >
                <Text
                        font-size="32px"
                        color="white"
                >
                    -
                </Text>
            </Button>

            <Button
                    styles="centerNode"
                    id="reset"
                    width="120px"
                    height="64px"
            >
                <Text
                        font-size="20px"
                        color="white"
                >
                    Reset
                </Text>
            </Button>

            <Button
                    styles="centerNode"
                    id="increment"
                    width="64px"
                    height="64px"
            >
                <Text
                        font-size="32px"
                        color="white"
                >+
                </Text>
            </Button>
        </Node>
    </Node>
</Page>
```

For more on attributes and styling, see:

- [The Schema Reference](./schema.md)
- [The Styles Guide](./styles.md)

## Spawning the page

Now that our page is complete, we need to integrate it into our app.
Since a `Page` is just a loaded asset, we would need to create an asset-loaded callback to spawn the loaded page.
Fortunately, `PageManager` handles all of this:

```rust
fn main() {
    let app = App::new();
    // ...
    app.add_systems(Startup, setup);
    // ...
    app.run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, mut manager: ResMut<PageManager>) {
    /*
    commands.spawn((Camera2d::default(), Camera::default(), Transform::default()));
    let handle = assets.load("counter.xml");
    */

    manager.spawn("counter", handle);
    manager.set_active("counter", true);
}
```

## Adding Interactivity

Interactivity is managed via element events and observers.

You can manage the page itself via the `Page` resource.

This `on_click` function shows how to handle our button clicks:

```rust
fn main() {
    // ...
    app.add_observer(on_click);
    // ...
}

// This function is called whenever an element is clicked.
fn on_click(
    click: On<ElementClick>,
    manager: Res<PageManager>,
    mut query: Query<&mut Properties<TextProps>>,
) {
    // Get the page with the ID "counter"
    if let Some(page) = manager.get("counter") {
        // Get the entity with the ID "counter"
        let counter_entity = page.get("counter");

        // Get the properties of the counter text
        let mut counter_text = query.get_mut(counter_entity).unwrap();
        let counter = counter_text.default.content.parse::<i32>().unwrap();

        // 'increment' button pressed => counter + 1
        if click.matches_id("increment") {
            counter_text.mutate(|props| props.content = (counter + 1).to_string());
        }

        // 'decrement' button pressed => counter - 1
        if click.matches_id("decrement") {
            counter_text.mutate(|props| props.content = (counter - 1).to_string());
        }

        // 'reset' button pressed => counter = 0
        if click.matches_id("reset") {
            counter_text.mutate(|props| props.content = "0".to_string());
        }
    }
}
```

Notice that we use `mutate(|props| ...)` to mutate the properties of the text entity.
Every widget has its own `Properties<WidgetProps>`. The `Properties` struct contains the properties of the widget for
different states: `default`, `hover` and `click`.
Using `props.mutate(|props| ...)` mutates **all** the property states.

## Despawning the Page

If you switch to a new scene, you may want to despawn the page. You can use the `PageManager` to do so:

```rust
fn despawn_page(mut commands: Commands, mut manager: ResMut<PageManager>) {
    manager.despawn(&mut commands, "counter");
}
```

## Summary

You now have a basic counter app with increment, decrement and reset functionality.

You can now do `cargo run` to run your app and look at the beautiful page you created!
