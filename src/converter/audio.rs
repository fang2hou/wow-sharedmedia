//! Audio format conversion (MP3/WAV/FLAC → OGG Vorbis).

use std::num::NonZeroU8;
use std::num::NonZeroU32;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::Error;

/// Convert an audio file to OGG Vorbis format for WoW.
///
/// If the input is already .ogg, copies it directly (no re-encoding).
/// Otherwise, decodes via symphonia and re-encodes as OGG Vorbis via vorbis_rs.
/// Default quality: 0.4.
pub fn convert_to_ogg(input: &Path, output: &Path) -> Result<AudioConvertResult, Error> {
	convert_to_ogg_with_quality(input, output, 0.4)
}

/// Convert an audio file to OGG Vorbis with configurable quality.
///
/// Quality range: 0.0 (lowest) to 1.0 (highest).
/// If the input is already .ogg, copies it directly (quality parameter ignored).
pub fn convert_to_ogg_with_quality(input: &Path, output: &Path, quality: f32) -> Result<AudioConvertResult, Error> {
	let ext = input
		.extension()
		.and_then(|e| e.to_str())
		.map(|e| e.to_lowercase())
		.unwrap_or_default();

	// Pass-through for .ogg
	if ext == "ogg" {
		std::fs::copy(input, output).map_err(|e| Error::Io {
			source: e,
			path: input.to_path_buf(),
		})?;
		return probe_audio(output);
	}

	// Open source file
	let src = std::fs::File::open(input).map_err(|e| Error::Io {
		source: e,
		path: input.to_path_buf(),
	})?;

	// Create media source stream
	let mss = MediaSourceStream::new(Box::new(src), Default::default());

	// Create hint from file extension
	let mut hint = Hint::new();
	if !ext.is_empty() {
		hint.with_extension(&ext);
	}

	// Probe the format
	let meta_opts: MetadataOptions = Default::default();
	let fmt_opts: FormatOptions = Default::default();

	let probed = symphonia::default::get_probe()
		.format(&hint, mss, &fmt_opts, &meta_opts)
		.map_err(|e| Error::InvalidAudio(format!("Cannot detect audio format: {e}")))?;

	let mut format = probed.format;

	// Find the first audio track with a known codec
	let track = format
		.tracks()
		.iter()
		.find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
		.ok_or_else(|| Error::InvalidAudio("No supported audio track found".to_string()))?;

	let track_id = track.id;
	let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
	let channels = track.codec_params.channels.map(|c| c.count() as u32).unwrap_or(2);

	// Create decoder
	let dec_opts: DecoderOptions = Default::default();
	let mut decoder = symphonia::default::get_codecs()
		.make(&track.codec_params, &dec_opts)
		.map_err(|e| Error::InvalidAudio(format!("Unsupported codec: {e}")))?;

	// Decode all packets and collect interleaved f32 PCM
	let mut all_samples: Vec<f32> = Vec::new();
	let mut total_duration_frames: u64 = 0;

	loop {
		let packet = match format.next_packet() {
			Ok(packet) => packet,
			Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
				break;
			}
			Err(e) => {
				return Err(Error::InvalidAudio(format!("Format error: {e}")));
			}
		};

		if packet.track_id() != track_id {
			continue;
		}

		match decoder.decode(&packet) {
			Ok(decoded) => {
				let spec = *decoded.spec();
				let frames = decoded.frames() as u64;
				let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
				sample_buf.copy_interleaved_ref(decoded);
				all_samples.extend_from_slice(sample_buf.samples());
				total_duration_frames += frames;
			}
			Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
			Err(symphonia::core::errors::Error::IoError(_)) => continue,
			Err(e) => {
				return Err(Error::InvalidAudio(format!("Decode error: {e}")));
			}
		}
	}

	// Calculate duration
	let duration_secs = if sample_rate > 0 && channels > 0 {
		total_duration_frames as f64 / sample_rate as f64
	} else {
		0.0
	};

	// Create output directory if needed
	if let Some(parent) = output.parent() {
		std::fs::create_dir_all(parent).map_err(|e| Error::Io {
			source: e,
			path: parent.to_path_buf(),
		})?;
	}

	let ogg_file = std::fs::File::create(output).map_err(|e| Error::Io {
		source: e,
		path: output.to_path_buf(),
	})?;

	// vorbis_rs 0.5 API: builder with NonZero types
	let nz_sample_rate = NonZeroU32::new(sample_rate).unwrap_or(NonZeroU32::new(44100).unwrap());
	let nz_channels = NonZeroU8::new(channels as u8).unwrap_or(NonZeroU8::new(2).unwrap());

	let mut builder = vorbis_rs::VorbisEncoderBuilder::new_with_serial(nz_sample_rate, nz_channels, ogg_file, 1);
	builder.bitrate_management_strategy(vorbis_rs::VorbisBitrateManagementStrategy::QualityVbr {
		target_quality: quality.clamp(0.0, 1.0),
	});

	let mut encoder = builder
		.build()
		.map_err(|e| Error::AudioConversion(format!("Failed to build encoder: {e}")))?;

	// encode_audio_block takes planar format: &[&[f32]] (channels × samples)
	// We have interleaved PCM, so de-interleave
	let num_channels = channels as usize;
	if num_channels > 0 && !all_samples.is_empty() {
		let samples_per_channel = all_samples.len() / num_channels;

		// Build planar buffers
		let mut planar: Vec<Vec<f32>> = vec![Vec::with_capacity(samples_per_channel); num_channels];
		for (i, sample) in all_samples.iter().enumerate() {
			planar[i % num_channels].push(*sample);
		}

		// Encode in blocks of up to 1024 samples (libvorbis recommended)
		let block_size = 1024;
		let mut offset = 0;
		while offset < samples_per_channel {
			let end = (offset + block_size).min(samples_per_channel);
			let block: Vec<&[f32]> = planar.iter().map(|ch| &ch[offset..end]).collect();
			encoder
				.encode_audio_block(&block)
				.map_err(|e| Error::AudioConversion(format!("Encoding error: {e}")))?;
			offset = end;
		}
	}

	// Finalize
	encoder
		.finish()
		.map_err(|e| Error::AudioConversion(format!("Failed to finalize: {e}")))?;

	Ok(AudioConvertResult {
		duration_secs,
		sample_rate,
		channels: channels as u32,
	})
}

/// Validate an audio file and extract metadata without conversion.
///
/// For OGG files, reads metadata via symphonia. For other formats,
/// performs a full decode to get accurate duration/sample rate/channels.
pub(crate) fn probe_audio(input: &Path) -> Result<AudioConvertResult, Error> {
	let ext = input
		.extension()
		.and_then(|e| e.to_str())
		.map(|e| e.to_lowercase())
		.unwrap_or_default();

	if ext != "ogg" {
		// For non-OGG, do a full decode to get accurate metadata.
		let tmp_output = input.with_extension("tmp.ogg");
		match convert_to_ogg(input, &tmp_output) {
			Ok(result) => {
				let _ = std::fs::remove_file(&tmp_output);
				return Ok(result);
			}
			Err(error) => {
				let _ = std::fs::remove_file(&tmp_output);
				return Err(error);
			}
		}
	}

	// For OGG files, probe via symphonia
	let src = std::fs::File::open(input).map_err(|e| Error::Io {
		source: e,
		path: input.to_path_buf(),
	})?;

	let mss = MediaSourceStream::new(Box::new(src), Default::default());

	let mut hint = Hint::new();
	hint.with_extension("ogg");

	let meta_opts: MetadataOptions = Default::default();
	let fmt_opts: FormatOptions = Default::default();

	let probed = match symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts) {
		Ok(p) => p,
		Err(error) => {
			return Err(Error::InvalidAudio(format!("Cannot probe ogg metadata: {error}")));
		}
	};

	let mut format = probed.format;

	// Find first audio track
	let track = match format.tracks().iter().find(|t| t.codec_params.codec != CODEC_TYPE_NULL) {
		Some(t) => t,
		None => {
			return Err(Error::InvalidAudio(
				"No supported audio track found in ogg file".to_string(),
			));
		}
	};

	let sample_rate = track.codec_params.sample_rate.unwrap_or(0);
	let channels = track.codec_params.channels.map(|c| c.count() as u32).unwrap_or(2);

	// Create decoder to count frames for accurate duration
	let dec_opts: DecoderOptions = Default::default();
	let track_id = track.id;

	let mut decoder = match symphonia::default::get_codecs().make(&track.codec_params, &dec_opts) {
		Ok(d) => d,
		Err(error) => {
			return Err(Error::InvalidAudio(format!("Unsupported ogg codec: {error}")));
		}
	};

	let mut total_frames: u64 = 0;

	loop {
		let packet = match format.next_packet() {
			Ok(p) => p,
			Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
				break;
			}
			Err(_) => break,
		};

		if packet.track_id() != track_id {
			continue;
		}

		match decoder.decode(&packet) {
			Ok(decoded) => {
				total_frames += decoded.frames() as u64;
			}
			Err(_) => continue,
		}
	}

	let duration_secs = if sample_rate > 0 {
		total_frames as f64 / sample_rate as f64
	} else {
		0.0
	};

	Ok(AudioConvertResult {
		duration_secs,
		sample_rate,
		channels,
	})
}

/// Result of an audio conversion or probe operation.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioConvertResult {
	/// Duration in seconds.
	pub duration_secs: f64,
	/// Sample rate in Hz.
	pub sample_rate: u32,
	/// Number of channels in the decoded stream.
	pub channels: u32,
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::TempDir;

	fn write_test_wav(path: &Path, sample_rate: u32, channels: u16, samples: &[i16]) {
		let bits_per_sample: u16 = 16;
		let block_align: u16 = channels * (bits_per_sample / 8);
		let byte_rate: u32 = sample_rate * block_align as u32;
		let data_size: u32 = std::mem::size_of_val(samples) as u32;
		let riff_size: u32 = 36 + data_size;

		let mut bytes = Vec::with_capacity((44 + data_size) as usize);
		bytes.extend_from_slice(b"RIFF");
		bytes.extend_from_slice(&riff_size.to_le_bytes());
		bytes.extend_from_slice(b"WAVE");
		bytes.extend_from_slice(b"fmt ");
		bytes.extend_from_slice(&16u32.to_le_bytes());
		bytes.extend_from_slice(&1u16.to_le_bytes());
		bytes.extend_from_slice(&channels.to_le_bytes());
		bytes.extend_from_slice(&sample_rate.to_le_bytes());
		bytes.extend_from_slice(&byte_rate.to_le_bytes());
		bytes.extend_from_slice(&block_align.to_le_bytes());
		bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
		bytes.extend_from_slice(b"data");
		bytes.extend_from_slice(&data_size.to_le_bytes());
		for sample in samples {
			bytes.extend_from_slice(&sample.to_le_bytes());
		}

		std::fs::write(path, bytes).unwrap();
	}

	#[test]
	fn test_convert_wav_to_ogg_and_probe() {
		let dir = TempDir::new().unwrap();
		let input = dir.path().join("input.wav");
		let output = dir.path().join("output.ogg");

		let samples = [0i16, 8192, -8192, 4096, -4096, 0, 2048, -2048];
		write_test_wav(&input, 44_100, 1, &samples);

		let result = convert_to_ogg(&input, &output).unwrap();
		assert!(output.exists());
		assert_eq!(result.sample_rate, 44_100);
		assert_eq!(result.channels, 1);
		assert!(result.duration_secs >= 0.0);

		let probed = probe_audio(&output).unwrap();
		assert!(probed.sample_rate > 0);
		assert!(probed.channels > 0);
	}

	#[test]
	fn test_convert_ogg_passthrough() {
		let dir = TempDir::new().unwrap();
		let wav = dir.path().join("input.wav");
		let ogg = dir.path().join("input.ogg");
		let copied = dir.path().join("copied.ogg");

		let samples = [0i16, 4096, -4096, 0];
		write_test_wav(&wav, 22_050, 1, &samples);
		convert_to_ogg(&wav, &ogg).unwrap();

		let original_bytes = std::fs::read(&ogg).unwrap();
		let result = convert_to_ogg(&ogg, &copied).unwrap();
		let copied_bytes = std::fs::read(&copied).unwrap();

		assert_eq!(original_bytes, copied_bytes);
		assert!(result.sample_rate > 0);
	}

	#[test]
	fn test_invalid_audio_errors() {
		let dir = TempDir::new().unwrap();
		let input = dir.path().join("bad.wav");
		let output = dir.path().join("bad.ogg");
		std::fs::write(&input, b"not really a wav").unwrap();

		let result = convert_to_ogg(&input, &output);
		assert!(result.is_err());
		match result.unwrap_err() {
			Error::InvalidAudio(_) => {}
			other => panic!("Expected InvalidAudio, got: {other}"),
		}
	}

	#[test]
	fn test_probe_invalid_ogg_errors() {
		let dir = TempDir::new().unwrap();
		let input = dir.path().join("bad.ogg");
		std::fs::write(&input, b"not really an ogg").unwrap();

		let result = probe_audio(&input);
		assert!(result.is_err());
		match result.unwrap_err() {
			Error::InvalidAudio(_) => {}
			other => panic!("Expected InvalidAudio, got: {other}"),
		}
	}
}
