//! BLP texture format reading (BLP2 → DynamicImage).

use crate::Error;

/// Read a BLP2 file and convert to a DynamicImage.
pub fn read_blp(path: &std::path::Path) -> Result<image::DynamicImage, Error> {
	let data = std::fs::read(path).map_err(|e| Error::Io {
		source: e,
		path: path.to_path_buf(),
	})?;

	let blp_image =
		wow_blp::parser::parse_blp(&data).map_err(|e| Error::ImageConversion(format!("Failed to parse BLP: {e}")))?;

	let decoded = wow_blp::convert::blp_to_image(&blp_image, 0)
		.map_err(|e| Error::ImageConversion(format!("Failed to decode BLP: {e}")))?;

	Ok(decoded)
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::TempDir;

	#[test]
	fn test_read_blp_missing_file() {
		let dir = TempDir::new().unwrap();
		let path = dir.path().join("missing.blp");

		let result = read_blp(&path);
		assert!(result.is_err());
		match result.unwrap_err() {
			Error::Io { path: err_path, .. } => assert_eq!(err_path, path),
			other => panic!("Expected Io error, got: {other}"),
		}
	}

	#[test]
	fn test_read_blp_invalid_payload_maps_to_image_conversion() {
		let dir = TempDir::new().unwrap();
		let path = dir.path().join("invalid.blp");
		std::fs::write(&path, b"definitely not a real blp file").unwrap();

		let result = read_blp(&path);
		assert!(result.is_err());
		match result.unwrap_err() {
			Error::ImageConversion(msg) => assert!(msg.contains("Failed to parse BLP")),
			other => panic!("Expected ImageConversion, got: {other}"),
		}
	}
}
