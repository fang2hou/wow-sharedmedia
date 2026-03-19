//! AddonData — the single source of truth, persisted as data.lua.

use crate::MediaEntry;

/// Current data.lua schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Top-level data structure persisted in data.lua.
///
/// This replaces the old Manifest. It contains everything the addon needs:
/// version tracking, generation timestamp, and all media entries.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddonData {
	/// Schema version of the `data.lua` format.
	pub schema_version: u32,
	/// Version of the tool that last wrote this file.
	pub version: String,
	#[serde(with = "chrono::serde::ts_milliseconds")]
	/// Timestamp of the last successful write, in UTC.
	pub generated_at: chrono::DateTime<chrono::Utc>,
	/// All registered media entries.
	pub entries: Vec<MediaEntry>,
}

impl AddonData {
	/// Create a new empty AddonData with the given tool version.
	///
	/// This is typically used by [`crate::ensure_addon_dir`] when bootstrapping
	/// a fresh addon directory.
	pub fn empty(tool_version: &str) -> Self {
		Self {
			schema_version: SCHEMA_VERSION,
			version: tool_version.to_string(),
			generated_at: chrono::Utc::now(),
			entries: Vec::new(),
		}
	}
}
