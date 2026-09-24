//! Character references in text: `&amp;`, `&lt;`, `&#39;`, `&#x1F600;` ...

/// Replaces the references a browser would have decoded. Unknown references stay as written.
pub(super) fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        decoded.push_str(&rest[..start]);
        rest = &rest[start..];
        match rest.find(';').filter(|&end| end <= 10).and_then(|end| decode_reference(&rest[1..end]).map(|c| (c, end))) {
            Some((character, end)) => {
                decoded.push(character);
                rest = &rest[end + 1..];
            }
            None => {
                decoded.push('&');
                rest = &rest[1..];
            }
        }
    }
    decoded.push_str(rest);
    decoded
}

/// The character for the text between `&` and `;`.
fn decode_reference(reference: &str) -> Option<char> {
    match reference {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{a0}'),
        _ => {
            let number = reference.strip_prefix('#')?;
            let code = match number.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/presentation/ui/shared/safe_markup/entities.rs"]
mod tests;
