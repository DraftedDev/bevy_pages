# Schema Reference

**This file documents the entire syntax and structure of the XML format. For the actual schema file, see
the [schema.xsd](../schema.xsd).**

---

# Table of Contents

- [Overview](#overview)
- [Common Types](#common-types)
- [Common Attributes](#common-attributes)
- [Attribute State-Overrides](#attribute-state-overrides)
- [Widgets](#widgets)
    - [Node Widget](#node-widget)
    - [Button Widget](#button-widget)
    - [Text Widget](#text-widget)
    - [Checkbox Widget](#checkbox-widget)
    - [Divider Widget](#divider-widget)
    - [Dropdown Widget](#dropdown-widget)
    - [Image Widget](#image-widget)
    - [Progress Bar Widget](#progress-bar-widget)
    - [Scroll View Widget](#scroll-view-widget)
    - [Slider Widget](#slider-widget)
    - [Text Input Widget](#text-input-widget)
    - [Tooltip Widget](#tooltip-widget)
- [Styles](#styles)

---

# Overview

A page always consists of a `<Page></Page>` element with any number of child elements.

Example:

```xml

<Page>
    <Text>Hello</Text>
    <Text>World</Text>
</Page>
```

---

# Common Types

Different attributes use different types. While strings, ints and floats are self-explanatory, there are some more
complex type definitions:

| Type           | Format                                                                                                                                                                          | Example                                      |
|----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------|
| Color          | CSS-like colors. Available: `rgb(...)`, `rgba(...)`, `hsl(...)`, `hsla(...)`, `#<hex>`, named colors.                                                                           | `rgb(255, 20, 20)`, `#FFFFFF`, `lightblue`   |
| Size           | CSS-like size. Syntax: `auto`, `<float><unit>` or just `<float>` to default to pixel unit. Available units: `px`, `%`, `vw`, `vh`, `vmin`, `vmax`.                              | `auto`, `10`, `35.5vw`                       |
| Font Size      | CSS-like font size values. Syntax: `<float><unit>` or just `<float>` to default to pixel unit. Available units: `px`, `rem`, `vw`, `vh`, `vmin`, `vmax`.                        | `10.5`, `3rem`                               |
| Rect           | Available: empty for empty rect, `<float>` for uniform, `<float> <size>` for vertical + horizontal, `<float> <float> <float> <float>` for X, Y, Width and Height.               | `10`, `10 15`, `10 15 10.5 15.5`             |
| UI Rect        | Available: empty for empty rect, `<size>` for uniform, `<size> <size>` for vertical + horizontal, `<size> <size> <size> <size>` for explicit top, right, bottom, left values.   | `10px`, `10% 15%`, `10 15 10vw 15vh`         |
| Radius         | Available: empty for no radius, `<size>` for uniform, `<size> <size> <size> <size>` for Top-Left, Top-Right, Bottom-Right, Bottom-Left.                                         | `8px`, `10.5 10.5 1% 1%`                     |
| Grid Track     | Syntax: `auto`, `<float><unit>`. Available units: `px`, `%`, `fr`.                                                                                                              | `auto`, `1fr`, `10%`                         |
| Grid Template  | Space seperated list of track sizes. Track sizes are either `auto`, `<float>` (pixels unit) or `<float><unit>` with units: `px`, `%`, `fr`, `flex`, `vw`, `vh`, `vmin`, `vmax`. | `100px`, `10px 20px 30px`,  `50 100 150 200` |
| Grid Placement | Either an integer or `auto`.                                                                                                                                                    | `1`, `2`, `auto`                             |
| Border Rect    | Available: `<float>` for uniform, `<float> <float>` for X & Y axis, `<float> <float> <float> <float>` for min X, min Y, max X, max Y.                                           | `100.0`, `20.0 10.0`, `5.0 10.0 15.0 20.0`   |

---

# Common Attributes

Every element inside a page, including the page element itself, has common attributes. These attributes can ALWAYS be
used and are completely optional. Note that different widgets may override default attributes if they aren't set
manually.

| Attribute                      | Description                                                       | Type                                                  |
|--------------------------------|-------------------------------------------------------------------|-------------------------------------------------------|
| id                             | The unique identifier of an element                               | Any String                                            |
| styles                         | A space separated list of styles                                  | List of Strings                                       |
| bg-color                       | The background color of the element                               | Color                                                 |
| border-color                   | The general border color                                          | Color                                                 |
| border-color-top               | The top border color                                              | Color                                                 |
| border-color-right             | The right border color                                            | Color                                                 |
| border-color-bottom            | The bottom border color                                           | Color                                                 |
| border-color-left              | The left border color                                             | Color                                                 |
| display                        | The display mode of the element                                   | `flex`, `grid`, `block`,                              |
| `none`                         |                                                                   |                                                       |
| box-sizing                     | The box sizing of the element                                     | `border`,                                             |
| `content`                      |                                                                   |                                                       |
| position                       | The positioning of the element                                    | `absolute`,                                           |
| `relative`                     |                                                                   |                                                       |
| left                           | The concrete position to the left                                 | Size                                                  |
| right                          | The concrete position to the right                                | Size                                                  |
| top                            | The concrete position to the top                                  | Size                                                  |
| bottom                         | The concrete position to the bottom                               | Size                                                  |
| width                          | The preferred element width                                       | Size                                                  |
| height                         | The preferred element height                                      | Size                                                  |
| min-width                      | The minimum element width                                         | Size                                                  |
| min-height                     | The minimum element height                                        | Size                                                  |
| max-width                      | The maximum element width                                         | Size                                                  |
| max-height                     | The maximum element height                                        | Size                                                  |
| aspect-ratio                   | The element aspect ratio                                          | Float                                                 |
| overflow-x                     | The overflow behavior on the X-Axis                               | `visible`, `clip`, `hidden`,                          |
| `scroll`                       |                                                                   |                                                       |
| overflow-y                     | The overflow behavior on the Y-Axis                               | `visible`, `clip`, `hidden`,                          |
| `scroll`                       |                                                                   |                                                       |
| scrollbar-width                | How much space should be reserved for a scroll bar (on overflow)  | Float                                                 |
| overflow-clip-visual-box       | The clipped visual box                                            | `padding`, `content`,                                 |
| `border`                       |                                                                   |                                                       |
| overflow-clip-margin           | Margin on each edge of the visual box in pixels                   | Float                                                 |
| align-items                    | Where to align the element items                                  | `default`, `start`, `end`, `center`, `baseline`,      |
| `stretch`                      |                                                                   |                                                       |
| justify-items                  | How to justify the element item layouts                           | `default`, `start`, `end`, `center`,                  |
| `stretch`                      |                                                                   |                                                       |
| align-self                     | Where to align the element                                        | `auto`, `start`, `end`, `center`,                     |
| `stretch`                      |                                                                   |                                                       |
| justify-self                   | How to justify the element layout                                 | `auto`, `start`, `end`, `center`,                     |
| `stretch`                      |                                                                   |                                                       |
| align-content                  | Where to align the element content                                | `default`, `start`, `end`, `center`,                  |
| `stretch`                      |                                                                   |                                                       |
| justify-content                | How to justify the element content layouts                        | `default`, `start`, `end`, `center`, `space-between`, |
| `space-around`, `space-evenly` |                                                                   |                                                       |
| direction                      | The element inline-axis direction                                 | `ltr`,                                                |
| `rtl`                          |                                                                   |                                                       |
| margin                         | The element margin                                                | UI Rect                                               |
| padding                        | The element padding                                               | UI Rect                                               |
| border                         | The element border UI-Rect                                        | UI Rect                                               |
| border-radius                  | The element border-radius                                         | Radius                                                |
| row-gap                        | The size between rows in a grid layout or horizontal flex layout  | Size                                                  |
| column-gap                     | The size between columns in a grid layout or vertical flex layout | Size                                                  |
| flex-direction                 | If a flexboy layout should be a row or column                     | `row`, `column`, `row-reverse`,                       |
| `column-reverse`               |                                                                   |                                                       |
| flex-wrap                      | The wrapping mechanics of overflowing flexboxes                   | `no-wrap`, `wrap`,                                    |
| `wrap-reverse`                 |                                                                   |                                                       |
| flex-grow                      | Defines flexbox growth in layouts with enough space               | Float                                                 |
| flex-shrink                    | Defines flexbox shrinking in layouts without enough space         | Float                                                 |
| flex-basis                     | The initial length of a flexbox before shrinking/growing          | Size                                                  |
| grid-auto-flow                 | Controls how automatically placed grid items are placed           | `row`, `column`, `row-dense`,                         |
| `column-dense`                 |                                                                   |                                                       |
| grid-template-rows             | Defines the number and size of rows inside a grid layout          | Grid Track                                            |
| grid-template-columns          | Defines the number and size of columns in a grid layout           | Grid Track                                            |
| grid-auto-rows                 | Defines the size of implicitly created grid rows                  | Grid Template                                         |
| grid-auto-columns              | Defines the size of implicitly created grid columns               | Grid Template                                         |
| grid-row                       | The row in a grid layout in which an item starts                  | Grid Placement                                        |
| grid-column                    | The column in a grid layout in which an item starts               | Grid Placement                                        |

---

# Attribute State-Overrides

Attributes can be overridden during specific states. The format for these attributes is:

`<state>.<attr> = "<value>"`.

This syntax is valid for **ALL** attributes, though overriding some widget-specific attributes may not change anything,
as using those overrides is still up to widget logic. Common attribute overrides are all supported though.

Following states exist:

| State | Description                          | Example                                         |
|-------|--------------------------------------|-------------------------------------------------|
| hover | Active when the widget is hovered on | `click.bg-color = "..."`, `click.width = "..."` |
| click | Active when the widget is clicked    | `click.bg-color = "..."`, `click.wdith = "..."` |

---

# Widgets

There are a lot of widgets using simple and complex `bevy` UI logic, but they can all easily be built using simple XML
syntax.

---

## Node Widget

A node widget. Equivalent to `<div></div>` in HTML.

### XML Usage

Build a node widget using the `<Node></Node>` tag.

The node widget does not introduce any new attributes.

### Logic

This widget does not have any special code logic.
Use element events to implement custom behavior.

---

## Button Widget

A simple button widget.

### XML Usage

Build a button widget using the `<Button></Button>` tag.

This widget does not spawn new entities and does not have any new attributes.

The only difference between this and the normal node widget,
is that the button will apply different defaults to itself.

It's really just a `<Node></Node>` with default `click.<...>` and `hover.<...>` attributes.

### Logic

This widget does not have any special code logic.
Use the different element events to implement custom behavior.

---

## Text Widget

A text widget.

### XML Usage

Build a text widget with the `<Text>...</Text>` tag.
You can specify the text content either using the inner tag text or the `content` attribute.

#### Attributes

- `content = "<string>"`: Sets the content of the text.
- `font = "<string>"`: Sets the font of the text. When unspecified, the default bevy font will be used.
- `font-weight = "<int|thin|extra_light|light|normal|medium|semibold|bold|extra_bold|black|extra_black>"`: Sets the
  font weight.
- `font-width = "<float>"`: Sets the font width.
- `font-size = "<fontSize>"`: Sets the font size.
- `font-style = "<normal|italic|oblique>"`: Sets the font style.
- `color = "<color>"`: Sets the text color.

All the attributes listed support state overrides.

### Logic

Use the `TextProps` to control the text.
This widget does not have any special code logic.

You may use generic element events to implement custom behavior.

---

### Checkbox Widget

A checkbox widget using an inner rounded rectangle indicator.

### XML Usage

Build a checkbox widget using the `<Checkbox />` tag.

#### Attributes

- `checked = "<bool>"`: The state of the checkbox.
- `check-color = "<color>"`: The background color of the inner marker.
- `check-width = "<size>"`: The width of the inner marker.
- `check-height = "<size>"`: The height of the inner marker.
- `check-radius = "<size>"`: The border radius of the inner marker.

All the attributes listed, except `checked`, support state overrides.

### Logic

Use `CheckboxProps` to control the checkbox widget.
Furthermore, the widget emits `ElementToggle` events when toggled.

You may also use generic element events to implement custom behavior.

---

## Switch widget.

A switch widget.

It's really just a fancier checkbox.

### XML Usage

Build a switch widget using the `<Switch />` tag.

#### Attributes

- `toggled = "<bool>"`: The state of the switch.
- `thumb-color = "<color>"`: The color of the thumb when not toggled.
- `thumb-color-on = "<color>"`: The color of the thumb when toggled.
- `thumb-size = "<size>"`: The size of the thumb.

All the attributes listed support state overrides.

### Logic

Use `SwitchProps` to control the switch widget.
Furthermore, the widget emits `ElementToggle` events when toggled.

You may also use generic element events to implement custom behavior.

---

## Divider Widget

A widget to show a divider line.

### XML Usage

Build a divider widget using the `<Divider />` tag.

#### Attributes

- `orientation = "<horizontal|vertical>"`: The orientation of the line.

The attributes listed do not support any state overrides.

### Logic

This widget does not have any special code logic, but you can read `DividerProps`.
Use generic element events to implement custom behavior.

---

## Dropdown Widget

A dropdown widget with a list of options.

### XML Usage

Build a dropdown widget using the `<Dropdown></Dropdown>` tag.

Every "root" children of the dropdown is considered a new option.

#### Attributes

- `placeholder = "<string>"`: The placeholder text to display when no option is selected.
- `dropdown-bg-color = "<color>"`: The background color of the dropdown.
- `menu-bg-color = "<color>"`: The background color of the dropdown menu.

All the attributes listed support state overrides.

### Logic

Use the `DropdownProps` to control the dropdown.
Furthermore, the dropdown widget triggers `ElementSet<String>` when an option is selected.

---

## Image Widget

An image widget.

### XML Usage

Build a new image widget using the `<Image />` tag.

#### Attributes

- `src = "<string>"`: The asset path to the actual image to display. Required.
- `color = "<color>"`: The color/tint of the image.
- `flip-x = "<bool>"`: Whether to flip the image horizontally.
- `flip-y = "<bool>"`: Whether to flip the image vertically.
- `rect = "<rect>"`: The rect to clip the image to.
- `mode = "<auto|sliced|tiled|stretch>"`: The layout mode to use for the image.
- `visual-box = "<padding|content|border>"`: The visual box of the image.
- `sliced-border = "<borderRect>"`: The border rect when `mode = "sliced"`.
- `sliced-center-scale-stretch = "<float>"`: The center scale when `mode = "sliced"` and
  `sliced-center-scale = "tile"`.
- `sliced-center-scale = "<stretch|tile>"`: The center scale when `mode = "sliced"`.
- `sliced-sides-scale-stretch = "<float>"`: The sides scale when `mode = "sliced"` and
  `sliced-sides-scale = "tile"`.
- `sliced-sides-scale = "<stretch|tile>"`: The sides scale when `mode = "sliced"`.
- `sliced-max-corner-scale = "<float>"`: The max corner scale when `mode = "sliced"`.
- `tiled-x = "<bool>"`: Whether to tile the image horizontally, when `mode = "tiled"`.
- `tiled-y = "<bool>"`: Whether to tile the image vertically, when `mode = "tiled"`.
- `tiled-stretch = "<float>"`: The stretch scale when `mode = "tiled"`.

All the attributes listed support state overrides.

### Logic

Use `ImageProps` to control the image widget.
Use generic element events to implement custom behavior.

---

## Progress Bar Widget

A progress bar widget.

### XML Usage

Build a new progress bar using the `<ProgressBar />` tag.

#### Attributes

- `min = "<float>"`: The minimum of the progress bar.
- `max = "<float>"`: The maximum of the progress bar.
- `value = "<float>"`: The value of the progress bar.
- `track-color = "<color>"`: The background color of the progress bar container track.
- `fill-color = "<color>"`: The color of the inner filled progress indicator bar.
- `track-height = "<size>"`: The height of the track.

All the attributes listed support state overrides.

### Logic

Use the `ProgressBarProps` to control the progress bar.
You may also use generic element events to implement custom behavior.

---

## Scroll View Widget

A scroll view widget that allows users to scroll through overflowing content.

### XML Usage

Build a scroll view using the `<ScrollView></ScrollView>` tag.

Insert as many children as you want.

#### Attributes

- `scroll-direction = "<vertical|horizontal|both>"`: The scroll direction. See `ScrollDirection`.
- `scroll-speed = "<float>"`: The scroll speed.
- `color = "<color>"`: The background color of the scroll view container.
- `smooth = "<bool>"`: Whether to use smooth scrolling.

All the attributes support state overrides.

### Logic

Use the `ScrollViewProps` to control the scroll view.
You may also use generic element events to implement custom behavior.

---

## Slider Widget

A slider widget.

### XML Usage

Build a slider widget using the `<Slider />` tag.

#### Attributes

- `min = "<float>"`: The minimum of the slider.
- `max = "<float>"`: The maximum of the slider.
- `step = "<float>"`: The step size for the slider.
- `value = "<float>"`: The value of the slider.
- `track-color = "<color>"`: The color of the slider track.
- `thumb-color = "<color>"`: The color of the slider thumb.
- `fill-color = "<color>"`: The color of the slider fill.
- `track-height = "<size>"`: The height of the slider track.
- `thumb-size = "<size>"`: The size of the slider thumb.

All the attributes, except `min` and `max`, support state overrides.

### Logic

Use the `SliderProps` to control the slider.
Furthermore, the slider emits `ElementSet<f32>` events.

You may also use generic element events to implement custom behavior.

---

## Text Input Widget

A text input widget that enables typed user input.

### XML Usage

Build a new text input widget using the `<TextInput/>` tag.

#### Attributes

- `value = "<string>"`: The text input value.
- `font-size = "<fontSize>"`: The text font size.
- `font = "<string>"`: The font of the text. When unspecified, the default bevy font will be used.
- `visible-width = "<float>"`: The optional maximum width of visible text inside the text box.
- `allow-newlines = "<bool>"`: If the text input should allow new lines.
- `color = "<color>"`: The text color.

All the attributes listed, except `value`, support state overrides.

### Logic

Use `TextInputProps` to control the text input widget.
Furthermore, the widget emits `ElementSet<String>` events when text is typed.

You can also use generic element events to implement custom behavior.

---

## Tooltip Widget

A tooltip widget that displays additional information when hovered.

### XML Usage

Build a new tooltip widget using the `<Tooltip></Tooltip>` tag.

The children of the tooltip will trigger the tooltip popup when hovered on.

#### Attributes

- `text = "<string>"`: The tooltip text.
- `anchor = "<top|bottom|left|right>"`: The anchor/position of the tooltip relative to its children.
  See `TooltipAnchor`.
- `tooltip-bg-color = "<color>"`: The background color of the tooltip popup.
- `text-color = "<color>"`: The text color inside the tooltip popup.
- `font-size = "<fontSize>"`: The font size of the text. See `parse_font_size`.

All the attributes listed support state overrides.

### Logic

Use `TooltipProps` to control the text.
You can use generic element events to implement custom behavior.

# Styles

The `<Styles>...</Styles>` tag must always come first inside the root `<Page></Page>` element.

Every style is defined inside the `<Styles>...</Styles>` tag as `<Style attr1="abc" attr2="def"/>`.

A `<Style/>` must have a `name` attribute and can have any number of attributes and may use prefixes (like
`hover.color="..."` or `click.width="..."`).

Every attribute specified inside a style is forwarded to the widget parsing method, which means that you can even
specify attributes that are totally unused and ignored.
