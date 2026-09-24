//! Hex colours ("03A9F4") to CSS `rgba(...)`.

/// Red, green, blue of a hex colour. Like the original, a 3-digit colour uses each digit
/// unscaled (so "fff" is 15,15,15, not 255,255,255).
fn hex_to_rgb(hex: &str) -> [u32; 3] {
    let hex = hex.trim_start_matches('#');
    let channel = |digits: &str| u32::from_str_radix(digits, 16).unwrap_or(0);
    match hex.len() {
        6 => [channel(&hex[0..2]), channel(&hex[2..4]), channel(&hex[4..6])],
        3 => [channel(&hex[0..1]), channel(&hex[1..2]), channel(&hex[2..3])],
        _ => [0, 0, 0],
    }
}

/// `rgba(r,g,b,opacity)` for a hex colour and an opacity between 0 and 1.
pub fn rgba(hex: &str, opacity: f64) -> String {
    let [red, green, blue] = hex_to_rgb(hex);
    format!("rgba({red},{green},{blue},{opacity})")
}

#[cfg(test)]
#[path = "../../../tests/unit/presentation/theme/color_conversion.rs"]
mod tests;
