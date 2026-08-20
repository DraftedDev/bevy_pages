use crate::parser::color::parse_color;
use bevy::math::{Rect, Vec2};
use bevy::prelude::{
    BorderColor, BorderRadius, BorderRect, FontSize, GridPlacement, GridTrack, RepeatedGridTrack,
    UiRect, Val,
};
use roxmltree::Node;

pub(crate) fn parse_attribute<T, F>(
    node: &Node,
    attr_name: &str,
    prefix: Option<&str>,
    parser: F,
) -> Result<Option<T>, String>
where
    F: FnOnce(&str) -> Result<T, String>,
{
    let key = match prefix {
        Some(p) => format!("{}.{}", p, attr_name),
        None => attr_name.to_string(),
    };

    node.attribute(key.as_str()).map(parser).transpose()
}

pub(crate) fn parse_border_color(
    node: &Node,
    prefix: Option<&str>,
) -> Result<Option<BorderColor>, String> {
    if let Some(color) = parse_attribute(node, "border-color", prefix, parse_color)? {
        return Ok(Some(BorderColor::all(color)));
    }

    let top = parse_attribute(node, "border-color-top", prefix, parse_color)?;
    let right = parse_attribute(node, "border-color-right", prefix, parse_color)?;
    let bottom = parse_attribute(node, "border-color-bottom", prefix, parse_color)?;
    let left = parse_attribute(node, "border-color-left", prefix, parse_color)?;

    if top.is_some() || right.is_some() || bottom.is_some() || left.is_some() {
        Ok(Some(BorderColor {
            top: top.unwrap_or_default(),
            right: right.unwrap_or_default(),
            bottom: bottom.unwrap_or_default(),
            left: left.unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

pub(crate) fn parse_rect(i: &str) -> Result<Rect, String> {
    let values: Vec<f32> = i
        .split_whitespace()
        .map(parse_float)
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [] => Ok(Rect::default()),

        [all] => Ok(Rect {
            min: Vec2::new(-*all, -*all),
            max: Vec2::new(*all, *all),
        }),

        [width, height] => Ok(Rect {
            min: Vec2::new(-*width / 2.0, -*height / 2.0),
            max: Vec2::new(*width / 2.0, *height / 2.0),
        }),

        [left, top, right, bottom] => Ok(Rect {
            min: Vec2::new(*left, *top),
            max: Vec2::new(*right, *bottom),
        }),

        _ => Err(format!("Invalid rect '{i}'. Expected 1, 2 or 4 values.")),
    }
}

pub(crate) fn parse_ui_rect(i: &str) -> Result<UiRect, String> {
    let values: Vec<Val> = i
        .split_whitespace()
        .map(parse_val)
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [] => Ok(UiRect::ZERO),

        [all] => Ok(UiRect::all(*all)),

        [vertical, horizontal] => Ok(UiRect {
            left: *horizontal,
            right: *horizontal,
            top: *vertical,
            bottom: *vertical,
        }),

        [top, horizontal, bottom] => Ok(UiRect {
            left: *horizontal,
            right: *horizontal,
            top: *top,
            bottom: *bottom,
        }),

        [top, right, bottom, left] => Ok(UiRect {
            left: *left,
            right: *right,
            top: *top,
            bottom: *bottom,
        }),

        _ => Err(format!("Invalid rect '{i}'. Expected 1-4 values.")),
    }
}

pub(crate) fn parse_border_radius(i: &str) -> Result<BorderRadius, String> {
    let values: Vec<Val> = i
        .split_whitespace()
        .map(parse_val)
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [] => Ok(BorderRadius::ZERO),

        [all] => Ok(BorderRadius::all(*all)),

        [top_left, top_right, bottom_right, bottom_left] => Ok(BorderRadius {
            top_left: *top_left,
            top_right: *top_right,
            bottom_right: *bottom_right,
            bottom_left: *bottom_left,
        }),

        _ => Err(format!(
            "Invalid border radius '{i}'. Expected 1 or 4 values."
        )),
    }
}

/// Parses a string into a [BorderRect].
///
/// ## Supported Formats
/// - **1 value**: `"10"`: Uniform inset `(10, 10), (10, 10)`
/// - **2 values**: `"10 20"`: Axes `(X, Y)` inset `(10, 20), (10, 20)`
/// - **4 values**: `"5 10 15 20"`: Explicit `min_x, min_y, max_x, max_y`
pub fn parse_border_rect(s: &str) -> Result<BorderRect, String> {
    let parts: Vec<&str> = s
        .trim()
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();

    let numbers: Vec<f32> = parts
        .into_iter()
        .map(parse_float)
        .collect::<Result<_, _>>()?;

    match numbers.as_slice() {
        [val] => Ok(BorderRect {
            min_inset: Vec2::splat(*val),
            max_inset: Vec2::splat(*val),
        }),
        [x, y] => Ok(BorderRect {
            min_inset: Vec2::new(*x, *y),
            max_inset: Vec2::new(*x, *y),
        }),
        [min_x, min_y, max_x, max_y] => Ok(BorderRect {
            min_inset: Vec2::new(*min_x, *min_y),
            max_inset: Vec2::new(*max_x, *max_y),
        }),
        _ => Err(format!(
            "Invalid BorderRect format: '{s}'. Expected 1, 2, or 4 float numbers."
        )),
    }
}

pub(crate) fn parse_grid_template(i: &str) -> Result<Vec<GridTrack>, String> {
    if i.trim().is_empty() {
        return Ok(Vec::new());
    }

    i.split_whitespace()
        .map(|value| {
            Ok(GridTrack::px(parse_float(
                value.strip_suffix("px").unwrap_or(value),
            )?))
        })
        .collect()
}

pub(crate) fn parse_grid_track(i: &str) -> Result<Vec<RepeatedGridTrack>, String> {
    let track = if let Some(i) = i.strip_suffix("px") {
        GridTrack::px(parse_float(i)?)
    } else if let Some(i) = i.strip_suffix('%') {
        GridTrack::percent(parse_float(i)?)
    } else if let Some(i) = i.strip_suffix("fr") {
        GridTrack::flex(parse_float(i)?)
    } else if i == "auto" {
        GridTrack::auto()
    } else {
        return Err(format!(
            "Invalid grid track '{i}'. Expected px, %, fr or auto."
        ));
    };

    Ok(vec![track])
}

pub(crate) fn parse_grid_placement(i: &str) -> Result<GridPlacement, String> {
    let i = i.trim().to_lowercase();

    if i == "auto" {
        return Ok(GridPlacement::auto());
    }

    let line = i
        .parse::<i16>()
        .map_err(|err| format!("Invalid grid placement '{i}': {err}"))?;

    Ok(GridPlacement::start(line))
}

pub(crate) fn parse_val(i: &str) -> Result<Val, String> {
    let i = i.trim().to_lowercase();

    if let Ok(i) = i.parse::<f32>() {
        return Ok(Val::Px(i));
    }

    if i == "auto" {
        return Ok(Val::Auto);
    }

    if let Some(i) = i.strip_suffix("px") {
        return Ok(Val::Px(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("%") {
        return Ok(Val::Percent(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("vw") {
        return Ok(Val::Vw(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("vh") {
        return Ok(Val::Vh(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("vmin") {
        return Ok(Val::VMin(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("vmax") {
        return Ok(Val::VMax(parse_float(i)?));
    }

    Err(format!(
        "Failed to parse value '{i}'. Expected 'auto', <float> or <float><unit> where <unit> is one of: px, %, vw, vh, vmin, vmax"
    ))
}

pub(crate) fn parse_font_size(i: &str) -> Result<FontSize, String> {
    let i = i.trim().to_lowercase();

    if let Ok(i) = i.parse::<f32>() {
        return Ok(FontSize::Px(i));
    }

    if let Some(i) = i.strip_suffix("px") {
        return Ok(FontSize::Px(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("rem") {
        return Ok(FontSize::Rem(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("vw") {
        return Ok(FontSize::Vw(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("vh") {
        return Ok(FontSize::Vh(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("vmin") {
        return Ok(FontSize::VMin(parse_float(i)?));
    }

    if let Some(i) = i.strip_suffix("vmax") {
        return Ok(FontSize::VMax(parse_float(i)?));
    }

    Err(format!(
        "Failed to parse value '{i}'. Expected <float> or <float><unit> where <unit> is one of: px, rem, vw, vh, vmin, vmax"
    ))
}

pub(crate) fn parse_matches<T>(
    i: &str,
    cases: &[(&str, &dyn Fn() -> Result<T, String>)],
) -> Result<T, String> {
    let i = i.trim().to_lowercase();

    for (case, f) in cases {
        if *case == i {
            return f();
        }
    }

    let possible = cases.iter().map(|(case, _)| *case).collect::<Vec<_>>();

    Err(format!(
        "Failed to parse value '{i}'. Expected one of: {}",
        possible.join(", ")
    ))
}

#[inline(always)]
pub(crate) fn parse_float(i: &str) -> Result<f32, String> {
    i.to_lowercase()
        .trim()
        .parse::<f32>()
        .map_err(|err| format!("Failed to parse float '{i}': {err}"))
}

#[inline(always)]
pub(crate) fn parse_int(i: &str) -> Result<i32, String> {
    i.to_lowercase()
        .trim()
        .parse::<i32>()
        .map_err(|err| format!("Failed to parse int '{i}': {err}"))
}

#[inline(always)]
pub(crate) fn parse_bool(i: &str) -> Result<bool, String> {
    i.to_lowercase()
        .trim()
        .parse::<bool>()
        .map_err(|err| format!("Failed to parse int '{i}': {err}"))
}
