use crate::element::{Element, ElementId, ElementProps};
use crate::loader::PageLoader;
use crate::page::Page;
use crate::parser::color::parse_color;
use crate::parser::styles::{Styles, fetch_style_attrs, parse_styles};
use crate::parser::values::{
    parse_attribute, parse_border_color, parse_border_radius, parse_float, parse_grid_placement,
    parse_grid_template, parse_grid_track, parse_matches, parse_ui_rect, parse_val,
};
use crate::props::Properties;
use bevy::prelude::OverflowClipMargin;
use bevy::ui;
use bevy::ui::{
    AlignContent, AlignItems, AlignSelf, BoxSizing, Display, FlexDirection, FlexWrap, GridAutoFlow,
    InlineDirection, JustifyContent, JustifyItems, JustifySelf, Overflow, OverflowAxis,
    PositionType, Val, VisualBox,
};
use roxmltree::{Document, Node};
use rustc_hash::FxHashMap;

/// Contains color parsing functions.
pub mod color;

/// Contains value parsing functions.
pub mod values;

/// Contains styles parsing functions.
pub mod styles;

/// A map of XML node attributes.
pub type AttributeMap = FxHashMap<String, String>;

/// Parses a [Page] from the given XML document using the provided [PageLoader].
#[inline(always)]
pub fn parse_page(loader: &PageLoader, doc: Document) -> Result<Page, String> {
    let root_xml = doc
        .root()
        .first_child()
        .expect("Failed to get root element");

    assert!(
        root_xml.has_tag_name("Page"),
        "Root element must be a <Page> element",
    );

    let attrs = root_xml
        .attributes()
        .map(|attr| (attr.name().to_string(), attr.value().to_string()))
        .collect::<AttributeMap>();

    let styles = if let Some(first) = root_xml.children().find(|n| n.has_tag_name("Styles")) {
        parse_styles(&first)
    } else {
        Styles::default()
    };

    let root = parse_node(&attrs, true, None, None)?;

    let elements = parse_children_elements(&root_xml, loader, &styles)?;

    Ok(Page::new(root, elements))
}

/// Parses a UI [Element] from the given XML node and [PageLoader].
///
/// This function will match the tag name of the XML node to a [Widget] in the [PageLoader]
/// and set up the widget with the properties from the XML node.
pub fn parse_element(node: &Node, loader: &PageLoader, styles: &Styles) -> Result<Element, String> {
    if !node.is_element() {
        return Err("Expected an XML element".into());
    }

    let tag = node.tag_name().name().to_string();

    let mut widget = loader
        .get_widget(&tag)
        .ok_or_else(|| format!("Unknown Element: '{tag}'"))?
        .dyn_clone();

    let style_attrs = fetch_style_attrs(node, styles)?;
    let mut attrs = node
        .attributes()
        .map(|attr| (attr.name().to_string(), attr.value().to_string()))
        .collect::<AttributeMap>();

    attrs.extend(style_attrs);

    let mut default = parse_props(&attrs, None, None)?;
    let mut hover = parse_props(&attrs, Some("hover"), Some(&default))?;
    let mut click = parse_props(&attrs, Some("click"), Some(&default))?;

    let id = parse_attribute(&attrs, "id", None, |s| Ok(ElementId::new(s)))?;

    widget.parse(node, attrs)?;

    widget.apply_defaults(node, &mut default, &mut hover, &mut click);

    Ok(Element {
        widget,
        id,
        children: parse_children_elements(node, loader, styles)?,
        props: Properties {
            default,
            hover,
            click,
        },
    })
}

/// Parses [ElementProps] from an [AttributeMap] with possible base props to fall back to.
///
/// You can specify an optional prefix for all attribute names (following convention `<prefix>.<name>`).
pub fn parse_props(
    attrs: &AttributeMap,
    prefix: Option<&str>,
    base: Option<&ElementProps>,
) -> Result<ElementProps, String> {
    let bg_color = parse_attribute(attrs, "bg-color", prefix, parse_color)?
        .or_else(|| base.and_then(|b| b.bg_color));

    let border_color =
        parse_border_color(attrs, prefix)?.or_else(|| base.and_then(|b| b.border_color));

    let node_layout = parse_node(attrs, false, prefix, base.map(|b| &b.node))?;

    Ok(ElementProps {
        node: node_layout,
        bg_color,
        border_color,
    })
}

/// Parses a list of child [Element]s from an [AttributeMap] using the provided [PageLoader].
#[inline(always)]
pub fn parse_children_elements(
    node: &Node,
    loader: &PageLoader,
    styles: &Styles,
) -> Result<Vec<Element>, String> {
    let mut children = Vec::with_capacity(if node.has_children() { 1 } else { 0 });

    for child in node.children().filter(|n| n.is_element()) {
        if child.has_tag_name("Styles") {
            continue;
        }

        children.push(parse_element(&child, loader, styles)?);
    }

    Ok(children)
}

/// Parses a [ui::Node] from an [AttributeMap] and optionally falls back to `base_node`.
/// You must also specify if the parsed node is the root (the most outer node) of the XML.
///
/// You can specify an optional prefix for all attribute names (following convention `<prefix>.<name>`).
pub fn parse_node(
    attrs: &AttributeMap,
    root: bool,
    prefix: Option<&str>,
    base_node: Option<&ui::Node>,
) -> Result<ui::Node, String> {
    let fallback = ui::Node {
        width: if root { Val::Percent(100.0) } else { Val::Auto },
        height: if root { Val::Percent(100.0) } else { Val::Auto },
        ..Default::default()
    };

    let base = base_node.unwrap_or(&fallback);

    Ok(ui::Node {
        display: parse_attribute(attrs, "display", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("flex", &|| Ok(Display::Flex)),
                    ("grid", &|| Ok(Display::Grid)),
                    ("block", &|| Ok(Display::Block)),
                    ("none", &|| Ok(Display::None)),
                ],
            )
        })?
        .unwrap_or(base.display),

        box_sizing: parse_attribute(attrs, "box-sizing", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("border", &|| Ok(BoxSizing::BorderBox)),
                    ("content", &|| Ok(BoxSizing::ContentBox)),
                ],
            )
        })?
        .unwrap_or(base.box_sizing),

        position_type: parse_attribute(attrs, "position", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("absolute", &|| Ok(PositionType::Absolute)),
                    ("relative", &|| Ok(PositionType::Relative)),
                ],
            )
        })?
        .unwrap_or(base.position_type),

        overflow: Overflow {
            x: parse_attribute(attrs, "overflow-x", prefix, |s| {
                parse_matches(
                    s,
                    &[
                        ("visible", &|| Ok(OverflowAxis::Visible)),
                        ("clip", &|| Ok(OverflowAxis::Clip)),
                        ("hidden", &|| Ok(OverflowAxis::Hidden)),
                        ("scroll", &|| Ok(OverflowAxis::Scroll)),
                    ],
                )
            })?
            .unwrap_or(base.overflow.x),

            y: parse_attribute(attrs, "overflow-y", prefix, |s| {
                parse_matches(
                    s,
                    &[
                        ("visible", &|| Ok(OverflowAxis::Visible)),
                        ("clip", &|| Ok(OverflowAxis::Clip)),
                        ("hidden", &|| Ok(OverflowAxis::Hidden)),
                        ("scroll", &|| Ok(OverflowAxis::Scroll)),
                    ],
                )
            })?
            .unwrap_or(base.overflow.y),
        },

        scrollbar_width: parse_attribute(attrs, "scrollbar-width", prefix, parse_float)?
            .unwrap_or(base.scrollbar_width),

        overflow_clip_margin: OverflowClipMargin {
            visual_box: parse_attribute(attrs, "overflow-clip-visual-box", prefix, |s| {
                parse_matches(
                    s,
                    &[
                        ("padding", &|| Ok(VisualBox::PaddingBox)),
                        ("content", &|| Ok(VisualBox::ContentBox)),
                        ("border", &|| Ok(VisualBox::BorderBox)),
                    ],
                )
            })?
            .unwrap_or(base.overflow_clip_margin.visual_box),

            margin: parse_attribute(attrs, "overflow-clip-margin", prefix, parse_float)?
                .unwrap_or(base.overflow_clip_margin.margin),
        },

        left: parse_attribute(attrs, "left", prefix, parse_val)?.unwrap_or(base.left),
        right: parse_attribute(attrs, "right", prefix, parse_val)?.unwrap_or(base.right),
        top: parse_attribute(attrs, "top", prefix, parse_val)?.unwrap_or(base.top),
        bottom: parse_attribute(attrs, "bottom", prefix, parse_val)?.unwrap_or(base.bottom),

        width: parse_attribute(attrs, "width", prefix, parse_val)?.unwrap_or(base.width),
        height: parse_attribute(attrs, "height", prefix, parse_val)?.unwrap_or(base.height),
        min_width: parse_attribute(attrs, "min-width", prefix, parse_val)?
            .unwrap_or(base.min_width),
        min_height: parse_attribute(attrs, "min-height", prefix, parse_val)?
            .unwrap_or(base.min_height),
        max_width: parse_attribute(attrs, "max-width", prefix, parse_val)?
            .unwrap_or(base.max_width),
        max_height: parse_attribute(attrs, "max-height", prefix, parse_val)?
            .unwrap_or(base.max_height),

        aspect_ratio: parse_attribute(attrs, "aspect-ratio", prefix, parse_float)?
            .or(base.aspect_ratio),

        align_items: parse_attribute(attrs, "align-items", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("default", &|| Ok(AlignItems::Default)),
                    ("start", &|| Ok(AlignItems::Start)),
                    ("end", &|| Ok(AlignItems::End)),
                    ("center", &|| Ok(AlignItems::Center)),
                    ("baseline", &|| Ok(AlignItems::Baseline)),
                    ("stretch", &|| Ok(AlignItems::Stretch)),
                ],
            )
        })?
        .unwrap_or(base.align_items),

        justify_items: parse_attribute(attrs, "justify-items", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("default", &|| Ok(JustifyItems::Default)),
                    ("start", &|| Ok(JustifyItems::Start)),
                    ("end", &|| Ok(JustifyItems::End)),
                    ("center", &|| Ok(JustifyItems::Center)),
                    ("stretch", &|| Ok(JustifyItems::Stretch)),
                ],
            )
        })?
        .unwrap_or(base.justify_items),

        align_self: parse_attribute(attrs, "align-self", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("auto", &|| Ok(AlignSelf::Auto)),
                    ("start", &|| Ok(AlignSelf::Start)),
                    ("end", &|| Ok(AlignSelf::End)),
                    ("center", &|| Ok(AlignSelf::Center)),
                    ("stretch", &|| Ok(AlignSelf::Stretch)),
                ],
            )
        })?
        .unwrap_or(base.align_self),

        justify_self: parse_attribute(attrs, "justify-self", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("auto", &|| Ok(JustifySelf::Auto)),
                    ("start", &|| Ok(JustifySelf::Start)),
                    ("end", &|| Ok(JustifySelf::End)),
                    ("center", &|| Ok(JustifySelf::Center)),
                    ("stretch", &|| Ok(JustifySelf::Stretch)),
                ],
            )
        })?
        .unwrap_or(base.justify_self),

        align_content: parse_attribute(attrs, "align-content", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("default", &|| Ok(AlignContent::Default)),
                    ("start", &|| Ok(AlignContent::Start)),
                    ("end", &|| Ok(AlignContent::End)),
                    ("center", &|| Ok(AlignContent::Center)),
                    ("stretch", &|| Ok(AlignContent::Stretch)),
                ],
            )
        })?
        .unwrap_or(base.align_content),

        justify_content: parse_attribute(attrs, "justify-content", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("default", &|| Ok(JustifyContent::Default)),
                    ("start", &|| Ok(JustifyContent::Start)),
                    ("end", &|| Ok(JustifyContent::End)),
                    ("center", &|| Ok(JustifyContent::Center)),
                    ("space-between", &|| Ok(JustifyContent::SpaceBetween)),
                    ("space-around", &|| Ok(JustifyContent::SpaceAround)),
                    ("space-evenly", &|| Ok(JustifyContent::SpaceEvenly)),
                ],
            )
        })?
        .unwrap_or(base.justify_content),

        direction: parse_attribute(attrs, "direction", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("ltr", &|| Ok(InlineDirection::Ltr)),
                    ("rtl", &|| Ok(InlineDirection::Rtl)),
                ],
            )
        })?
        .unwrap_or(base.direction),

        margin: parse_attribute(attrs, "margin", prefix, parse_ui_rect)?.unwrap_or(base.margin),
        padding: parse_attribute(attrs, "padding", prefix, parse_ui_rect)?.unwrap_or(base.padding),
        border: parse_attribute(attrs, "border", prefix, parse_ui_rect)?.unwrap_or(base.border),
        border_radius: parse_attribute(attrs, "border-radius", prefix, parse_border_radius)?
            .unwrap_or(base.border_radius),

        flex_direction: parse_attribute(attrs, "flex-direction", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("row", &|| Ok(FlexDirection::Row)),
                    ("column", &|| Ok(FlexDirection::Column)),
                    ("row-reverse", &|| Ok(FlexDirection::RowReverse)),
                    ("column-reverse", &|| Ok(FlexDirection::ColumnReverse)),
                ],
            )
        })?
        .unwrap_or(base.flex_direction),

        flex_wrap: parse_attribute(attrs, "flex-wrap", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("no-wrap", &|| Ok(FlexWrap::NoWrap)),
                    ("wrap", &|| Ok(FlexWrap::Wrap)),
                    ("wrap-reverse", &|| Ok(FlexWrap::WrapReverse)),
                ],
            )
        })?
        .unwrap_or(base.flex_wrap),

        flex_grow: parse_attribute(attrs, "flex-grow", prefix, parse_float)?
            .unwrap_or(base.flex_grow),
        flex_shrink: parse_attribute(attrs, "flex-shrink", prefix, parse_float)?
            .unwrap_or(base.flex_shrink),
        flex_basis: parse_attribute(attrs, "flex-basis", prefix, parse_val)?
            .unwrap_or(base.flex_basis),

        row_gap: parse_attribute(attrs, "row-gap", prefix, parse_val)?.unwrap_or(base.row_gap),
        column_gap: parse_attribute(attrs, "column-gap", prefix, parse_val)?
            .unwrap_or(base.column_gap),

        grid_auto_flow: parse_attribute(attrs, "grid-auto-flow", prefix, |s| {
            parse_matches(
                s,
                &[
                    ("row", &|| Ok(GridAutoFlow::Row)),
                    ("column", &|| Ok(GridAutoFlow::Column)),
                    ("row-dense", &|| Ok(GridAutoFlow::RowDense)),
                    ("column-dense", &|| Ok(GridAutoFlow::ColumnDense)),
                ],
            )
        })?
        .unwrap_or(base.grid_auto_flow),

        grid_template_rows: parse_attribute(attrs, "grid-template-rows", prefix, parse_grid_track)?
            .unwrap_or_else(|| base.grid_template_rows.clone()),
        grid_template_columns: parse_attribute(
            attrs,
            "grid-template-columns",
            prefix,
            parse_grid_track,
        )?
        .unwrap_or_else(|| base.grid_template_columns.clone()),
        grid_auto_rows: parse_attribute(attrs, "grid-auto-rows", prefix, parse_grid_template)?
            .unwrap_or_else(|| base.grid_auto_rows.clone()),
        grid_auto_columns: parse_attribute(
            attrs,
            "grid-auto-columns",
            prefix,
            parse_grid_template,
        )?
        .unwrap_or_else(|| base.grid_auto_columns.clone()),

        grid_row: parse_attribute(attrs, "grid-row", prefix, parse_grid_placement)?
            .unwrap_or(base.grid_row),
        grid_column: parse_attribute(attrs, "grid-column", prefix, parse_grid_placement)?
            .unwrap_or(base.grid_column),
    })
}
