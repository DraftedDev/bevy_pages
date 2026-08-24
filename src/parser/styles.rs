use crate::parser::AttributeMap;
use roxmltree::Node;
use rustc_hash::FxHashMap;

/// A map of styles.
pub type Styles = FxHashMap<String, StyleMap>;

/// A map of style attributes.
pub type StyleMap = FxHashMap<String, String>;

/// Parses a `<Styles></Styles>` node and returns a map of style names with style attributes.
///
/// Every node inside a `<Styles></Styles>` node must be a `<Style/>` element.
///
/// Every attribute inside a `<Style/>` element is added to the style.
#[inline(always)]
pub fn parse_styles(root: &Node) -> Styles {
    root.children()
        .filter(|n| n.is_element())
        .map(|child| {
            debug_assert!(
                child.has_tag_name("Style"),
                "Every node inside <Styles></Styles> must be a <Style> element"
            );

            let name = child
                .attribute("name")
                .expect("<Style> element must have a name attribute")
                .to_string();

            (name, parse_style(&child))
        })
        .collect::<Styles>()
}

/// Parses a `<Style/>` node and collects all attributes.
#[inline(always)]
pub fn parse_style(node: &Node) -> StyleMap {
    node.attributes()
        .map(|attr| (attr.name().to_string(), attr.value().to_string()))
        .collect::<StyleMap>()
}

/// Fetch an [AttributeMap] by parsing the `styles` attribute and collecting all style attributes.
#[inline(always)]
pub fn fetch_style_attrs(node: &Node, styles: &Styles) -> Result<AttributeMap, String> {
    let Some(styles_attr) = node.attribute("styles") else {
        return Ok(AttributeMap::default());
    };

    styles_attr
        .split_whitespace()
        .map(|name| {
            styles
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Style '{name}' not found"))
        })
        .flat_map(|res| match res {
            Ok(map) => map.into_iter().map(Ok).collect::<Vec<_>>(),
            Err(e) => vec![Err(e)],
        })
        .collect()
}
