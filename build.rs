use std::path::PathBuf;

const REQUIRED_VENDOR_FILES: &[&str] = &[
	"vendor/serpent/serpent.lua",
	"vendor/libsharedmedia-3.0/LibStub/LibStub.lua",
	"vendor/libsharedmedia-3.0/CallbackHandler-1.0/CallbackHandler-1.0.lua",
	"vendor/libsharedmedia-3.0/LibSharedMedia-3.0/LibSharedMedia-3.0.lua",
	"vendor/libsharedmedia-3.0/LibSharedMedia-3.0/lib.xml",
];

fn main() {
	println!("cargo:rerun-if-changed=vendor.lock.json");
	for path in REQUIRED_VENDOR_FILES {
		println!("cargo:rerun-if-changed={path}");
	}

	let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
	let missing: Vec<&str> = REQUIRED_VENDOR_FILES
		.iter()
		.copied()
		.filter(|path| !root.join(path).exists())
		.collect();

	if !missing.is_empty() {
		panic!(
			"missing vendored assets required for compile-time embedding: {}\nRun `bun run update-vendor` to materialize the pinned snapshot from vendor.lock.json before building.",
			missing.join(", "),
		);
	}
}
