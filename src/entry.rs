/// Media type enumeration for LibSharedMedia categories.
///
/// Each variant maps to an LSM registration type and has
/// associated file handling rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
	/// A LibSharedMedia statusbar texture.
	Statusbar,
	/// A LibSharedMedia background texture.
	Background,
	/// A LibSharedMedia border texture.
	Border,
	/// A LibSharedMedia font face.
	Font,
	/// A LibSharedMedia sound asset.
	Sound,
}

impl MediaType {
	/// Get the folder name for this media type.
	pub fn folder_name(&self) -> &'static str {
		match self {
			Self::Statusbar => "statusbar",
			Self::Background => "background",
			Self::Border => "border",
			Self::Font => "font",
			Self::Sound => "sound",
		}
	}

	/// Get the LSM registration type string.
	pub fn lsm_type(&self) -> &'static str {
		match self {
			Self::Statusbar => "statusbar",
			Self::Background => "background",
			Self::Border => "border",
			Self::Font => "font",
			Self::Sound => "sound",
		}
	}

	/// Get accepted input file extensions.
	pub fn accepted_extensions(&self) -> &'static [&'static str] {
		match self {
			Self::Statusbar | Self::Background | Self::Border => &[".tga", ".png", ".webp", ".jpg", ".jpeg", ".blp"],
			Self::Font => &[".ttf", ".otf"],
			Self::Sound => &[".ogg", ".mp3", ".wav"],
		}
	}

	/// Get the output file extension for WoW storage.
	pub fn output_extension(&self) -> &'static str {
		match self {
			Self::Statusbar | Self::Background | Self::Border => ".tga",
			Self::Font => "",
			Self::Sound => ".ogg",
		}
	}

	/// Whether this type supports locale masks.
	pub fn supports_locale(&self) -> bool {
		matches!(self, Self::Font)
	}
}

impl std::fmt::Display for MediaType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.lsm_type())
	}
}

impl std::str::FromStr for MediaType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"statusbar" => Ok(Self::Statusbar),
			"background" => Ok(Self::Background),
			"border" => Ok(Self::Border),
			"font" => Ok(Self::Font),
			"sound" => Ok(Self::Sound),
			_ => Err(format!("Unknown media type: {s}")),
		}
	}
}

/// Type-specific metadata extracted at import time.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct EntryMetadata {
	// Image fields (statusbar, background, border)
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Width of an imported image after conversion.
	pub image_width: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Height of an imported image after conversion.
	pub image_height: Option<u32>,

	// Font fields
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Font family name extracted from the font metadata.
	pub font_family: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Font style name extracted from the font metadata.
	pub font_style: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Whether the font is monospaced.
	pub font_is_monospace: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Total glyph count reported by the font.
	pub font_num_glyphs: Option<u32>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	/// Locale masks used when registering a font with LibSharedMedia.
	pub locales: Vec<String>,

	// Audio fields
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Audio duration in seconds.
	pub audio_duration_secs: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Audio sample rate in Hz.
	pub audio_sample_rate: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Number of audio channels.
	pub audio_channels: Option<u32>,
}

/// A single media entry in the addon registry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MediaEntry {
	/// Stable UUID for the entry.
	pub id: uuid::Uuid,
	#[serde(rename = "type")]
	/// LibSharedMedia asset type.
	pub media_type: MediaType,
	/// Display key used for registration in LibSharedMedia.
	pub key: String,
	/// Relative file path inside the addon directory.
	pub file: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Original file name provided by the user, if retained.
	pub original_name: Option<String>,
	/// Import timestamp in UTC.
	pub imported_at: chrono::DateTime<chrono::Utc>,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Optional content checksum for duplicate detection and auditing.
	pub checksum: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Optional type-specific metadata extracted during import.
	pub metadata: Option<EntryMetadata>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	/// User-defined tags associated with the entry.
	pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_media_type_parse_case_insensitive() {
		assert_eq!("statusbar".parse::<MediaType>().unwrap(), MediaType::Statusbar);
		assert_eq!("STATUSBAR".parse::<MediaType>().unwrap(), MediaType::Statusbar);
		assert_eq!("Font".parse::<MediaType>().unwrap(), MediaType::Font);
	}

	#[test]
	fn test_media_type_parse_invalid() {
		let result = "video".parse::<MediaType>();
		assert!(result.is_err());
		assert!(result.unwrap_err().contains("Unknown media type"));
	}

	#[test]
	fn test_media_type_extension_contracts() {
		assert_eq!(MediaType::Statusbar.output_extension(), ".tga");
		assert_eq!(MediaType::Background.output_extension(), ".tga");
		assert_eq!(MediaType::Border.output_extension(), ".tga");
		assert_eq!(MediaType::Font.output_extension(), "");
		assert_eq!(MediaType::Sound.output_extension(), ".ogg");

		assert!(MediaType::Statusbar.accepted_extensions().contains(&".png"));
		assert!(MediaType::Statusbar.accepted_extensions().contains(&".blp"));
		assert!(MediaType::Font.accepted_extensions().contains(&".ttf"));
		assert!(MediaType::Sound.accepted_extensions().contains(&".wav"));
	}

	#[test]
	fn test_media_type_locale_support_contract() {
		assert!(!MediaType::Statusbar.supports_locale());
		assert!(!MediaType::Background.supports_locale());
		assert!(!MediaType::Border.supports_locale());
		assert!(MediaType::Font.supports_locale());
		assert!(!MediaType::Sound.supports_locale());
	}
}
