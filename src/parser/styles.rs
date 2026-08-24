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
pub fn parse_styles(root: &Node) -> Styles {
    // TODO: better way to initialize hash map capacity
    let mut styles = FxHashMap::with_capacity_and_hasher(1, Default::default());

    for child in root.children().filter(|n| n.is_element()) {
        debug_assert!(
            child.has_tag_name("Style"),
            "Every node inside <Styles></Styles> must be a <Style> element"
        );

        let name = child
            .attribute("name")
            .expect("<Style> element must have a name attribute")
            .to_string();

        styles.insert(name, parse_style(&child));
    }

    styles
}

/// Parses a `<Style/>` node and collects all attributes.
pub fn parse_style(node: &Node) -> StyleMap {
    // TODO: better way to initialize hash map capacity
    let mut attrs = StyleMap::with_capacity_and_hasher(1, Default::default());

    for attr in node.attributes() {
        attrs.insert(attr.name().to_string(), attr.value().to_string());
    }

    attrs
}

/// Fetch an [AttributeMap] by parsing the `styles` attribute and collecting all style attributes.
pub fn fetch_style_attrs(node: &Node, styles: &Styles) -> Result<AttributeMap, String> {
    if let Some(names) = node.attribute("styles").map(|s| s.split(" ")) {
        // TODO: better way to initialize hash map capacity
        let mut attrs = AttributeMap::with_capacity_and_hasher(1, Default::default());

        for name in names {
            let style = styles
                .get(name)
                .ok_or_else(|| format!("Style '{}' not found", name))?;

            attrs.extend(style.clone());
        }

        Ok(attrs)
    } else {
        Ok(AttributeMap::default())
    }
}
