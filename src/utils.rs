/// Macro to set missing XML attributes.
///
/// Used inside [Widget::apply_defaults](crate::widgets::Widget::apply_defaults).
///
/// Only useful for widget development.
///
/// Usage Examples:
/// ```
/// # use bevy_pages::set_missing_attrs;
///
/// set_missing_attrs!(
///     node,
///     "attr" => some.value = 123,
///     "attr2" => some.value2 = 456,
/// )
/// ```
#[macro_export]
macro_rules! set_missing_attrs {
    ($node:expr, $( $attr:expr => $assignment:expr ),* $(,)?) => {
        $(
            if !$node.has_attribute($attr) {
                $assignment;
            }
        )*
    };
}

/// Mutates `target` only if its value differs from `compare`.
///
/// Used inside `update_props` systems of widgets.
///
/// Only useful for widget development.
///
/// Usage Examples:
/// ```
/// # use bevy_pages::set_if_changed;
///
/// // Direct Assignment
/// set_if_changed!(color.0, props.color);
///
/// // Custom Assignment
/// set_if_changed!(text.0, props.content => props.content.clone());
///
/// // Multiple Assignments
/// set_if_changed!(
///     color.0, props.color;
///     text.0, props.content => props.content.clone();
/// );
/// ```
#[macro_export]
macro_rules! set_if_changed {
    ($target:expr, $compare:expr => $assign:expr) => {
        if $target != $compare {
            $target = $assign;
        }
    };

    ($target:expr, $val:expr) => {
        if $target != $val {
            $target = $val;
        }
    };

    ($( $target:expr, $compare:expr $( => $assign:expr )? );* $(;)?) => {
        $(
            $crate::set_if_changed!($target, $compare $( => $assign )?);
        )*
    };
}
