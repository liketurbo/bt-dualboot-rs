pub fn is_valid_64_hex(input: &str) -> bool {
	input.len() == 12 && input.chars().all(|c| c.is_ascii_hexdigit())
}