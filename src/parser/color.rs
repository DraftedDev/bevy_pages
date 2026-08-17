use bevy::color::{Alpha, Color, Srgba};

pub(crate) fn lighten_color(color: Color, factor: f32) -> Color {
    let Srgba {
        red,
        green,
        blue,
        alpha,
    } = color.to_srgba();
    Color::srgb(
        (red + factor).min(1.0),
        (green + factor).min(1.0),
        (blue + factor).min(1.0),
    )
    .with_alpha(alpha)
}

pub(crate) fn darken_color(color: Color, factor: f32) -> Color {
    let Srgba {
        red,
        green,
        blue,
        alpha,
    } = color.to_srgba();
    Color::srgb(
        (red - factor).max(0.0),
        (green - factor).max(0.0),
        (blue - factor).max(0.0),
    )
    .with_alpha(alpha)
}

pub(crate) fn parse_color(i: &str) -> Result<Color, String> {
    let input = i.trim().to_ascii_lowercase();

    if let Some(color) = parse_named_color(&input) {
        return Ok(color);
    }

    if input.starts_with('#') {
        return parse_hex_color(&input);
    }

    if input.starts_with("rgb(") {
        return parse_rgb(&input, false);
    }

    if input.starts_with("rgba(") {
        return parse_rgb(&input, true);
    }

    if input.starts_with("hsl(") {
        return parse_hsl(&input, false);
    }

    if input.starts_with("hsla(") {
        return parse_hsl(&input, true);
    }

    Err(format!("Unknown color '{input}'"))
}

fn parse_hex_color(s: &str) -> Result<Color, String> {
    let hex = &s[1..];

    fn nibble(c: char) -> Result<u8, String> {
        c.to_digit(16)
            .map(|v| v as u8)
            .ok_or_else(|| format!("Invalid hex digit '{c}'"))
    }

    fn byte(s: &str) -> Result<u8, String> {
        u8::from_str_radix(s, 16).map_err(|_| format!("Invalid hex byte '{s}'"))
    }

    match hex.len() {
        3 => {
            let mut chars = hex.chars();
            let r = nibble(chars.next().unwrap())?;
            let g = nibble(chars.next().unwrap())?;
            let b = nibble(chars.next().unwrap())?;

            Ok(Color::srgba_u8(r * 17, g * 17, b * 17, 255))
        }

        4 => {
            let mut chars = hex.chars();
            let r = nibble(chars.next().unwrap())?;
            let g = nibble(chars.next().unwrap())?;
            let b = nibble(chars.next().unwrap())?;
            let a = nibble(chars.next().unwrap())?;

            Ok(Color::srgba_u8(r * 17, g * 17, b * 17, a * 17))
        }

        6 => Ok(Color::srgba_u8(
            byte(&hex[0..2])?,
            byte(&hex[2..4])?,
            byte(&hex[4..6])?,
            255,
        )),

        8 => Ok(Color::srgba_u8(
            byte(&hex[0..2])?,
            byte(&hex[2..4])?,
            byte(&hex[4..6])?,
            byte(&hex[6..8])?,
        )),

        _ => Err("Invalid hex color.".into()),
    }
}

fn parse_rgb(s: &str, alpha: bool) -> Result<Color, String> {
    let start = s.find('(').unwrap() + 1;
    let end = s.rfind(')').ok_or("Missing ')'")?;

    let values: Vec<_> = s[start..end].split(',').map(str::trim).collect();

    let expected = if alpha { 4 } else { 3 };

    if values.len() != expected {
        return Err(format!("Expected {expected} values."));
    }

    fn parse_channel(s: &str) -> Result<f32, String> {
        if let Some(v) = s.strip_suffix('%') {
            Ok(v.parse::<f32>().map_err(|_| "Bad percentage")? / 100.0)
        } else {
            Ok(s.parse::<f32>().map_err(|_| "Bad value")? / 255.0)
        }
    }

    fn parse_alpha(s: &str) -> Result<f32, String> {
        if let Some(v) = s.strip_suffix('%') {
            Ok(v.parse::<f32>().map_err(|_| "Bad alpha")? / 100.0)
        } else {
            s.parse::<f32>().map_err(|_| "Bad alpha".into())
        }
    }

    let r = parse_channel(values[0])?;
    let g = parse_channel(values[1])?;
    let b = parse_channel(values[2])?;
    let a = if alpha { parse_alpha(values[3])? } else { 1.0 };

    Ok(Color::srgba(r, g, b, a))
}

fn parse_hsl(s: &str, alpha: bool) -> Result<Color, String> {
    let start = s.find('(').unwrap() + 1;
    let end = s.rfind(')').ok_or("Missing ')'")?;

    let values: Vec<_> = s[start..end].split(',').map(str::trim).collect();

    let expected = if alpha { 4 } else { 3 };

    if values.len() != expected {
        return Err(format!("Expected {expected} values."));
    }

    let h = values[0].parse::<f32>().map_err(|_| "Invalid hue")?;

    let s = values[1]
        .trim_end_matches('%')
        .parse::<f32>()
        .map_err(|_| "Invalid saturation")?
        / 100.0;

    let l = values[2]
        .trim_end_matches('%')
        .parse::<f32>()
        .map_err(|_| "Invalid lightness")?
        / 100.0;

    let a = if alpha {
        values[3].parse::<f32>().map_err(|_| "Invalid alpha")?
    } else {
        1.0
    };

    Ok(Color::hsla(h, s, l, a))
}

fn parse_named_color(name: &str) -> Option<Color> {
    let (r, g, b, a) = match name.trim().to_ascii_lowercase().as_str() {
        "transparent" => (0, 0, 0, 0),

        "black" => (0x00, 0x00, 0x00, 0xFF),
        "silver" => (0xC0, 0xC0, 0xC0, 0xFF),
        "gray" | "grey" => (0x80, 0x80, 0x80, 0xFF),
        "white" => (0xFF, 0xFF, 0xFF, 0xFF),
        "maroon" => (0x80, 0x00, 0x00, 0xFF),
        "red" => (0xFF, 0x00, 0x00, 0xFF),
        "purple" => (0x80, 0x00, 0x80, 0xFF),
        "fuchsia" | "magenta" => (0xFF, 0x00, 0xFF, 0xFF),
        "green" => (0x00, 0x80, 0x00, 0xFF),
        "lime" => (0x00, 0xFF, 0x00, 0xFF),
        "olive" => (0x80, 0x80, 0x00, 0xFF),
        "yellow" => (0xFF, 0xFF, 0x00, 0xFF),
        "navy" => (0x00, 0x00, 0x80, 0xFF),
        "blue" => (0x00, 0x00, 0xFF, 0xFF),
        "teal" => (0x00, 0x80, 0x80, 0xFF),
        "aqua" | "cyan" => (0x00, 0xFF, 0xFF, 0xFF),

        "aliceblue" => (0xF0, 0xF8, 0xFF, 0xFF),
        "antiquewhite" => (0xFA, 0xEB, 0xD7, 0xFF),
        "aquamarine" => (0x7F, 0xFF, 0xD4, 0xFF),
        "azure" => (0xF0, 0xFF, 0xFF, 0xFF),
        "beige" => (0xF5, 0xF5, 0xDC, 0xFF),
        "bisque" => (0xFF, 0xE4, 0xC4, 0xFF),
        "blanchedalmond" => (0xFF, 0xEB, 0xCD, 0xFF),
        "blueviolet" => (0x8A, 0x2B, 0xE2, 0xFF),
        "brown" => (0xA5, 0x2A, 0x2A, 0xFF),
        "burlywood" => (0xDE, 0xB8, 0x87, 0xFF),
        "cadetblue" => (0x5F, 0x9E, 0xA0, 0xFF),
        "chartreuse" => (0x7F, 0xFF, 0x00, 0xFF),
        "chocolate" => (0xD2, 0x69, 0x1E, 0xFF),
        "coral" => (0xFF, 0x7F, 0x50, 0xFF),
        "cornflowerblue" => (0x64, 0x95, 0xED, 0xFF),
        "cornsilk" => (0xFF, 0xF8, 0xDC, 0xFF),
        "crimson" => (0xDC, 0x14, 0x3C, 0xFF),
        "darkblue" => (0x00, 0x00, 0x8B, 0xFF),
        "darkcyan" => (0x00, 0x8B, 0x8B, 0xFF),
        "darkgoldenrod" => (0xB8, 0x86, 0x0B, 0xFF),
        "darkgray" | "darkgrey" => (0xA9, 0xA9, 0xA9, 0xFF),
        "darkgreen" => (0x00, 0x64, 0x00, 0xFF),
        "darkkhaki" => (0xBD, 0xB7, 0x6B, 0xFF),
        "darkmagenta" => (0x8B, 0x00, 0x8B, 0xFF),
        "darkolivegreen" => (0x55, 0x6B, 0x2F, 0xFF),
        "darkorange" => (0xFF, 0x8C, 0x00, 0xFF),
        "darkorchid" => (0x99, 0x32, 0xCC, 0xFF),
        "darkred" => (0x8B, 0x00, 0x00, 0xFF),
        "darksalmon" => (0xE9, 0x96, 0x7A, 0xFF),
        "darkseagreen" => (0x8F, 0xBC, 0x8F, 0xFF),
        "darkslateblue" => (0x48, 0x3D, 0x8B, 0xFF),
        "darkslategray" | "darkslategrey" => (0x2F, 0x4F, 0x4F, 0xFF),
        "darkturquoise" => (0x00, 0xCE, 0xD1, 0xFF),
        "darkviolet" => (0x94, 0x00, 0xD3, 0xFF),
        "deeppink" => (0xFF, 0x14, 0x93, 0xFF),
        "deepskyblue" => (0x00, 0xBF, 0xFF, 0xFF),
        "dimgray" | "dimgrey" => (0x69, 0x69, 0x69, 0xFF),
        "dodgerblue" => (0x1E, 0x90, 0xFF, 0xFF),
        "firebrick" => (0xB2, 0x22, 0x22, 0xFF),
        "floralwhite" => (0xFF, 0xFA, 0xF0, 0xFF),
        "forestgreen" => (0x22, 0x8B, 0x22, 0xFF),
        "gainsboro" => (0xDC, 0xDC, 0xDC, 0xFF),
        "ghostwhite" => (0xF8, 0xF8, 0xFF, 0xFF),
        "gold" => (0xFF, 0xD7, 0x00, 0xFF),
        "goldenrod" => (0xDA, 0xA5, 0x20, 0xFF),
        "greenyellow" => (0xAD, 0xFF, 0x2F, 0xFF),
        "honeydew" => (0xF0, 0xFF, 0xF0, 0xFF),
        "hotpink" => (0xFF, 0x69, 0xB4, 0xFF),
        "indianred" => (0xCD, 0x5C, 0x5C, 0xFF),
        "indigo" => (0x4B, 0x00, 0x82, 0xFF),
        "ivory" => (0xFF, 0xFF, 0xF0, 0xFF),
        "khaki" => (0xF0, 0xE6, 0x8C, 0xFF),
        "lavender" => (0xE6, 0xE6, 0xFA, 0xFF),
        "lavenderblush" => (0xFF, 0xF0, 0xF5, 0xFF),
        "lawngreen" => (0x7C, 0xFC, 0x00, 0xFF),
        "lemonchiffon" => (0xFF, 0xFA, 0xCD, 0xFF),
        "lightblue" => (0xAD, 0xD8, 0xE6, 0xFF),
        "lightcoral" => (0xF0, 0x80, 0x80, 0xFF),
        "lightcyan" => (0xE0, 0xFF, 0xFF, 0xFF),
        "lightgoldenrodyellow" => (0xFA, 0xFA, 0xD2, 0xFF),
        "lightgray" | "lightgrey" => (0xD3, 0xD3, 0xD3, 0xFF),
        "lightgreen" => (0x90, 0xEE, 0x90, 0xFF),
        "lightpink" => (0xFF, 0xB6, 0xC1, 0xFF),
        "lightsalmon" => (0xFF, 0xA0, 0x7A, 0xFF),
        "lightseagreen" => (0x20, 0xB2, 0xAA, 0xFF),
        "lightskyblue" => (0x87, 0xCE, 0xFA, 0xFF),
        "lightslategray" | "lightslategrey" => (0x77, 0x88, 0x99, 0xFF),
        "lightsteelblue" => (0xB0, 0xC4, 0xDE, 0xFF),
        "lightyellow" => (0xFF, 0xFF, 0xE0, 0xFF),
        "limegreen" => (0x32, 0xCD, 0x32, 0xFF),
        "linen" => (0xFA, 0xF0, 0xE6, 0xFF),
        "mediumaquamarine" => (0x66, 0xCD, 0xAA, 0xFF),
        "mediumblue" => (0x00, 0x00, 0xCD, 0xFF),
        "mediumorchid" => (0xBA, 0x55, 0xD3, 0xFF),
        "mediumpurple" => (0x93, 0x70, 0xDB, 0xFF),
        "mediumseagreen" => (0x3C, 0xB3, 0x71, 0xFF),
        "mediumslateblue" => (0x7B, 0x68, 0xEE, 0xFF),
        "mediumspringgreen" => (0x00, 0xFA, 0x9A, 0xFF),
        "mediumturquoise" => (0x48, 0xD1, 0xCC, 0xFF),
        "mediumvioletred" => (0xC7, 0x15, 0x85, 0xFF),
        "midnightblue" => (0x19, 0x19, 0x70, 0xFF),
        "mintcream" => (0xF5, 0xFF, 0xFA, 0xFF),
        "mistyrose" => (0xFF, 0xE4, 0xE1, 0xFF),
        "moccasin" => (0xFF, 0xE4, 0xB5, 0xFF),
        "navajowhite" => (0xFF, 0xDE, 0xAD, 0xFF),
        "oldlace" => (0xFD, 0xF5, 0xE6, 0xFF),
        "olivedrab" => (0x6B, 0x8E, 0x23, 0xFF),
        "orange" => (0xFF, 0xA5, 0x00, 0xFF),
        "orangered" => (0xFF, 0x45, 0x00, 0xFF),
        "orchid" => (0xDA, 0x70, 0xD6, 0xFF),
        "palegoldenrod" => (0xEE, 0xE8, 0xAA, 0xFF),
        "palegreen" => (0x98, 0xFB, 0x98, 0xFF),
        "paleturquoise" => (0xAF, 0xEE, 0xEE, 0xFF),
        "palevioletred" => (0xDB, 0x70, 0x93, 0xFF),
        "papayawhip" => (0xFF, 0xEF, 0xD5, 0xFF),
        "peachpuff" => (0xFF, 0xDA, 0xB9, 0xFF),
        "peru" => (0xCD, 0x85, 0x3F, 0xFF),
        "pink" => (0xFF, 0xC0, 0xCB, 0xFF),
        "plum" => (0xDD, 0xA0, 0xDD, 0xFF),
        "powderblue" => (0xB0, 0xE0, 0xE6, 0xFF),
        "rebeccapurple" => (0x66, 0x33, 0x99, 0xFF),
        "rosybrown" => (0xBC, 0x8F, 0x8F, 0xFF),
        "royalblue" => (0x41, 0x69, 0xE1, 0xFF),
        "saddlebrown" => (0x8B, 0x45, 0x13, 0xFF),
        "salmon" => (0xFA, 0x80, 0x72, 0xFF),
        "sandybrown" => (0xF4, 0xA4, 0x60, 0xFF),
        "seagreen" => (0x2E, 0x8B, 0x57, 0xFF),
        "seashell" => (0xFF, 0xF5, 0xEE, 0xFF),
        "sienna" => (0xA0, 0x52, 0x2D, 0xFF),
        "skyblue" => (0x87, 0xCE, 0xEB, 0xFF),
        "slateblue" => (0x6A, 0x5A, 0xCD, 0xFF),
        "slategray" | "slategrey" => (0x70, 0x80, 0x90, 0xFF),
        "snow" => (0xFF, 0xFA, 0xFA, 0xFF),
        "springgreen" => (0x00, 0xFF, 0x7F, 0xFF),
        "steelblue" => (0x46, 0x82, 0xB4, 0xFF),
        "tan" => (0xD2, 0xB4, 0x8C, 0xFF),
        "thistle" => (0xD8, 0xBF, 0xD8, 0xFF),
        "tomato" => (0xFF, 0x63, 0x47, 0xFF),
        "turquoise" => (0x40, 0xE0, 0xD0, 0xFF),
        "violet" => (0xEE, 0x82, 0xEE, 0xFF),
        "wheat" => (0xF5, 0xDE, 0xB3, 0xFF),
        "whitesmoke" => (0xF5, 0xF5, 0xF5, 0xFF),
        "yellowgreen" => (0x9A, 0xCD, 0x32, 0xFF),

        _ => return None,
    };

    Some(Color::srgba_u8(r, g, b, a))
}
