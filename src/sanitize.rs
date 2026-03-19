//! Filename sanitization helpers.
//!
//! This module provides a conservative sanitizer suitable for generated addon
//! asset names. ASCII letters are normalized to lowercase while CJK, Hangul,
//! Hiragana, and Katakana characters are preserved.

/// Check if a char is a CJK character (Chinese, Japanese, Korean).
#[inline]
fn is_cjk_or_hangul(ch: char) -> bool {
	matches!(ch,
		'\u{4e00}'..='\u{9fff}' |     // CJK Unified Ideographs
		'\u{3400}'..='\u{4dbf}' |     // CJK Extension A
		'\u{f900}'..='\u{faff}' |     // CJK Compatibility Ideographs
		'\u{ac00}'..='\u{d7af}' |     // Hangul Syllables
		'\u{3040}'..='\u{309f}' |     // Hiragana
		'\u{30a0}'..='\u{30ff}' |     // Katakana
		'\u{31f0}'..='\u{31ff}'       // Katakana Phonetic Extensions
	)
}

/// Sanitize a filename for safe filesystem storage.
///
/// - Lowercase ASCII characters only (CJK preserved as-is)
/// - Replace spaces and special chars with `_`
/// - Preserve CJK characters, dots (extensions), hyphens, underscores
/// - Collapse consecutive underscores
/// - Trim leading/trailing underscores
pub fn sanitize_filename(name: &str) -> String {
	let mut result = String::with_capacity(name.len());
	let mut last_was_underscore = false;

	for ch in name.chars() {
		if ch.is_ascii_lowercase() || ch.is_ascii_digit() || is_cjk_or_hangul(ch) || ch == '.' || ch == '-' {
			result.push(ch);
			last_was_underscore = false;
		} else if ch.is_ascii_uppercase() {
			// Lowercase ASCII letters
			result.push(ch.to_ascii_lowercase());
			last_was_underscore = false;
		} else if !last_was_underscore {
			result.push('_');
			last_was_underscore = true;
		}
	}

	while result.ends_with('_') {
		result.pop();
	}
	while result.starts_with('_') {
		result.remove(0);
	}

	if result.is_empty() {
		"unnamed".to_string()
	} else {
		result
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_chinese_preserved() {
		assert_eq!(sanitize_filename("中文材质.tga"), "中文材质.tga");
	}

	#[test]
	fn test_special_chars_stripped() {
		assert_eq!(sanitize_filename("My Cool Texture!! 2.png"), "my_cool_texture_2.png");
	}

	#[test]
	fn test_consecutive_underscores() {
		assert_eq!(sanitize_filename("hello___world"), "hello_world");
	}

	#[test]
	fn test_empty_string() {
		assert_eq!(sanitize_filename(""), "unnamed");
		assert_eq!(sanitize_filename("!!!"), "unnamed");
	}

	#[test]
	fn test_trimming() {
		assert_eq!(sanitize_filename("_hello_"), "hello");
	}

	#[test]
	fn test_korean_preserved() {
		assert_eq!(sanitize_filename("한글폰트.ttf"), "한글폰트.ttf");
	}

	#[test]
	fn test_japanese_preserved() {
		assert_eq!(sanitize_filename("フォント.otf"), "フォント.otf");
	}
}
