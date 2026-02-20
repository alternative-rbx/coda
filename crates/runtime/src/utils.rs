#[inline(always)]
pub fn is_alpha(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

#[inline(always)]
pub fn is_alphanumeric(c: u8) -> bool {
    is_alpha(c) || c.is_ascii_digit()
}

#[inline(always)]
pub fn slice_to_string(slice: &[u8]) -> String {
    std::str::from_utf8(slice).unwrap_or("").to_string()
}

#[inline(always)]
pub fn trim_quotes(s: &str) -> &str {
    s.strip_prefix('"').unwrap_or(s).strip_suffix('"').unwrap_or(s)
}
