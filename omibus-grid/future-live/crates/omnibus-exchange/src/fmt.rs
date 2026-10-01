/// Format a f64 for exchange APIs: max N decimals, strip trailing zeros, strip dangling dot.
/// Fixes IEEE 754 artifacts where 0.04196 becomes "0.041960000000000003".
pub fn fmt_decimal(v: f64, max_decimals: usize) -> String {
    let s = format!("{:.*}", max_decimals, v);
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() || trimmed == "-" { "0".to_string() } else { trimmed.to_string() }
    } else { s }
}

/// Default price format: 8 decimals max. Suitable for most spot pairs.
pub fn fmt_price(v: f64) -> String { fmt_decimal(v, 8) }

/// Default quantity format: 8 decimals max.
pub fn fmt_qty(v: f64) -> String { fmt_decimal(v, 8) }
