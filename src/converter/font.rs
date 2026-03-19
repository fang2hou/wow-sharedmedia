//! Font metadata extraction via ttf-parser.

use std::path::Path;

use crate::Error;

/// Valid locale flag names recognized by LSM.
pub const LOCALE_NAMES: &[&str] = &["koKR", "ruRU", "zhCN", "zhTW", "western"];

/// Default locale list when none is specified for a font.
pub const DEFAULT_LOCALES: &[&str] = &["western"];

/// Validate that all locale names in the list are recognized.
pub fn validate_locale_names(names: &[&str]) -> Result<Vec<String>, Error> {
	let valid: std::collections::HashSet<&str> = LOCALE_NAMES.iter().copied().collect();
	let mut invalid = Vec::new();
	for name in names {
		if !valid.contains(name) {
			invalid.push(name.to_string());
		}
	}
	if invalid.is_empty() {
		Ok(names.iter().map(|s| s.to_string()).collect())
	} else {
		Err(Error::InvalidLocale(format!(
			"Invalid locale names: {}",
			invalid.join(", ")
		)))
	}
}

/// Extract metadata from a font file.
pub fn extract_font_metadata(path: &Path) -> Result<FontMetadata, Error> {
	let data = std::fs::read(path).map_err(|e| Error::Io {
		source: e,
		path: path.to_path_buf(),
	})?;

	if data.is_empty() {
		return Err(Error::InvalidFont(format!("Font file is empty: {}", path.display())));
	}

	let face =
		ttf_parser::Face::parse(&data, 0).map_err(|e| Error::InvalidFont(format!("Failed to parse font: {e}")))?;

	let family_name = face
		.names()
		.into_iter()
		.find(|n| n.name_id == ttf_parser::name_id::FAMILY && n.is_unicode())
		.and_then(|n| n.to_string())
		.unwrap_or_default();

	let style_name = face
		.names()
		.into_iter()
		.find(|n| n.name_id == ttf_parser::name_id::SUBFAMILY && n.is_unicode())
		.and_then(|n| n.to_string())
		.unwrap_or_default();

	Ok(FontMetadata {
		family_name,
		style_name,
		is_monospace: face.is_monospaced(),
		num_glyphs: face.number_of_glyphs() as u32,
		is_variable_font: face.is_variable(),
	})
}

/// Validate that a font file is a valid TTF or OTF.
///
/// Checks: file extension, non-empty, ttf-parser can parse header.
pub fn validate_font(path: &Path) -> Result<(), Error> {
	let ext = path
		.extension()
		.and_then(|e| e.to_str())
		.map(|e| e.to_lowercase())
		.unwrap_or_default();

	if ext != "ttf" && ext != "otf" {
		return Err(Error::InvalidFont(format!(
			"Unsupported font extension: .{} (expected .ttf or .otf)",
			ext
		)));
	}

	let data = std::fs::read(path).map_err(|e| Error::Io {
		source: e,
		path: path.to_path_buf(),
	})?;

	if data.is_empty() {
		return Err(Error::InvalidFont("Font file is empty".to_string()));
	}

	ttf_parser::Face::parse(&data, 0).map_err(|e| Error::InvalidFont(format!("Failed to parse font: {e}")))?;

	Ok(())
}

/// Metadata extracted from a valid font file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontMetadata {
	/// Font family name.
	pub family_name: String,
	/// Font style name.
	pub style_name: String,
	/// Whether the font reports itself as monospaced.
	pub is_monospace: bool,
	/// Number of glyphs reported by the face.
	pub num_glyphs: u32,
	/// Whether the font is a variable font.
	pub is_variable_font: bool,
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::TempDir;

	#[test]
	fn test_validate_locale_names_valid() {
		assert!(validate_locale_names(&["western", "zhCN"]).is_ok());
	}

	#[test]
	fn test_validate_locale_names_invalid() {
		assert!(validate_locale_names(&["western", "invalid"]).is_err());
	}

	#[test]
	fn test_validate_locale_names_empty() {
		let result = validate_locale_names(&[]);
		assert!(result.is_ok());
		assert!(result.unwrap().is_empty());
	}

	#[test]
	fn test_default_locales() {
		let locales: Vec<String> = DEFAULT_LOCALES.iter().map(|s| s.to_string()).collect();
		assert_eq!(locales, vec!["western"]);
	}

	#[test]
	fn test_validate_font_rejects_wrong_extension() {
		let dir = TempDir::new().unwrap();
		let path = dir.path().join("fake.txt");
		std::fs::write(&path, b"not a font").unwrap();

		let result = validate_font(&path);
		assert!(result.is_err());
		match result.unwrap_err() {
			Error::InvalidFont(msg) => assert!(msg.contains("Unsupported font extension")),
			other => panic!("Expected InvalidFont, got: {other}"),
		}
	}

	#[test]
	fn test_validate_font_rejects_empty_file() {
		let dir = TempDir::new().unwrap();
		let path = dir.path().join("empty.ttf");
		std::fs::write(&path, b"").unwrap();

		let result = validate_font(&path);
		assert!(result.is_err());
		match result.unwrap_err() {
			Error::InvalidFont(msg) => assert!(msg.contains("empty")),
			other => panic!("Expected InvalidFont, got: {other}"),
		}
	}

	#[cfg(target_os = "windows")]
	#[test]
	fn test_extract_font_metadata_from_system_font_smoke() {
		let candidates = [
			std::path::Path::new(r"C:\Windows\Fonts\arial.ttf"),
			std::path::Path::new(r"C:\Windows\Fonts\calibri.ttf"),
			std::path::Path::new(r"C:\Windows\Fonts\consola.ttf"),
		];

		let font = candidates
			.iter()
			.find(|p| p.exists())
			.expect("expected at least one standard Windows font to exist");

		validate_font(font).unwrap();
		let meta = extract_font_metadata(font).unwrap();

		assert!(!meta.family_name.is_empty() || !meta.style_name.is_empty());
		assert!(meta.num_glyphs > 0);
	}
}
