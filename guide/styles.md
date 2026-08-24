# Styles

CSS introduces the concept of selector-specific styling attributes. While `bevy_pages` doesn't have classes or classic
CSS selectors, you can still create your custom palette of styles.

## The `<Styles>...</Styles>` Element

Every XML document can have a `<Styles>...</Styles>` element, which contains a set of styles. This attribute **must** be
placed before any other elements (right inside `<Page>...</Page>`):

```xml
<?xml version="1.0" encoding="utf-8"?>
<Page
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="../schema.xsd"
        justify-content="center"
        align-items="center"
>
    <Styles>
        <!-- YOUR STYLES-->
    </Styles>

    <Text>
        Hello World!
    </Text>
</Page>

```

## Creating a `<Style/>`

Inside your `<Styles></Styles>` element, you can now add an infinite amount of `<Style/>` children. Each `<Style/>`
needs a `name` attribute to identify it.

A `<Style/>` element can now have any attribute you want. When applied to a widget, it automatically overrides the
widget's attributes.

```xml
<?xml version="1.0" encoding="utf-8"?>
<Page
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="../schema.xsd"
        justify-content="center"
        align-items="center"
>
    <Styles>
        <!-- Every attribute defined, will be applied to the widget.  -->
        <Style name="someStyle" width="200px" height="50px" hover.color="red" click.color="blue" padding="1px"/>

        <!-- This is still valid. The invalid attributes are simply not used. -->
        <Style name="otherStyle" padding="10px" thisAttrDoesntExist="123"/>
    </Styles>

    <!-- Apply multiple styles to a widget using a comma separated list. -->
    <!-- The widget will now have following attributes:
            - width="200px"
            - height="50px"
            - padding="10px"
            - hover.color="red"
            - click.color="blue
            - thisAttrDoesntExist="123" (ignored)
     -->
    <Text styles="someStyle otherStyle">
        Hello World!
    </Text>
</Page>
```

There are a few things to note:

1. Styles **must** have a `name` attribute.
2. Elements can specify `styles` with a comma separated list.
3. Even attributes that don't exist are valid.
4. You can even use `hover.` or `click.` to define attributes that are only applied when the widget is hovered or
   clicked.
5. You can have duplicate style attributes. The attribute of the last style specified will be used in this case.

## Styles under the Hood

Internally, styles are only applying attributes at the **XML-level** using the `AttributesMap` type which will then be
passed to the `parse` method of the `Widget`.

The runtime workflow is as follows:

1. The XML itself is parsed into a tree of nodes.
2. The different styles are parsed and stored.
3. Every style from every element is fetched and applies their attributes to the XML nodes.
4. The widget actually parses the attributes.
5. The runtime continues...
