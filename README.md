# bevy_pages

---

<a href="https://www.buymeacoffee.com/drafteddev" target="_blank">
  <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" width="180">
</a>

**A lightweight and elegant framework to upgrade your Bevy UI experience.**

---

[![crates.io](https://img.shields.io/crates/v/bevy_pages)](https://crates.io/crates/bevy_pages)
[![docs.rs](https://docs.rs/bevy_pages/badge.svg)](https://docs.rs/bevy_pages)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevy.org/learn/quick-start/plugin-development/#main-branch-tracking)

---

This is a crate extending [bevy](https://bevy.org) to add support for XML-based UI pages. Kinda like an HTML page, but
supercharged and with XML.

## Features

- **🪽 Lightweight:** Bevy is already huge enough. This crate only enables actually **necessary** features and has a *
  *minimal**
  set of dependencies.

- **⚡ Fast:** The only existent overhead is parsing the XML files, which is already very fast. Widgets and internal
  systems are written with **ECS-filtering** and **optimized queries**.

- **📜 Expressive:** Use the power of XML to create interactive UIs with ease. You can even
  have **multiple** pages activated at once and use custom CSS-like `<Styles></Styles>`.

- **✒️ Fully documented:** I added my signature `#[warn(missing_docs)]` and you can also look at
  the [official guide](guide/README.md) to learn more about the framework.

- **👁️ Built-in Auto-Completion:** Use the official [XSD Schema](schema.xsd) to enable **auto-completion** inside your
  favorite IDE.

- **✨ Extended Widget Catalog:** The framework has a set of basic and advanced widgets with optimal performance and
  customizability:
    - `<Node></Node>`
    - `<Button></Button>`
    - `<Text></Text>`
    - `<Checkbox/>`
    - `<Divider/>`
    - `<Slider/>`
    - `<Dropdown></Dropdown>`
    - `<ScrollView></ScrollView>`
    - `<ProgressBar/>`
    - `<TextInput/>`
    - `<Tooltip></Tooltip>`
    - `<Image/>`
    - `<Switch/>`

## Getting Started

After adding `bevy_pages` to your project, you can start coding your XML pages.

An XML page roughly looks like this:

```xml
<?xml version="1.0" encoding="utf-8"?>
<Page
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="https://raw.githubusercontent.com/DraftedDev/bevy_pages/master/schema.xsd">
    <Text>Hello World!</Text>
</Page>
```

**NOTE:** If you have issues with the remote `schema.xsd`, you are free to download it locally and link it in your XML.

You will also need to add the `PagesPlugin` to your app and use the `PageManager` to spawn and manage your page:

```rust
fn spawn_my_page(mut commands: Commands, assets: Res<AssetServer>, mut manager: ResMut<PageManager>) {
    let handle = assets.load("my_page.xml");

    manager.spawn("my_page", handle);
    // Pages are inactive by default
    manager.set_active("my_page", true);
}
```

See more in [this](guide/quick-start.md) Tutorial.

## Learning Resources

- **[The Official Guide](guide/README.md)**
- **[The Quick Start Tutorial](guide/quick-start.md)**
- **[The Official Examples](examples)**

## Bevy Support

The following table shows the supported `bevy` versions:

| bevy | bevy_pages     |
|------|----------------|
| 0.19 | 0.1.0 - latest |
