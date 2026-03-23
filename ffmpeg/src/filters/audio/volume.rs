use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum VolumePrecision {
	/// 8-bit fixed-point; this limits input sample format to U8, S16, and S32.
	#[strum(serialize = "fixed")]
	Fixed,

	/// 32-bit floating-point; this limits input sample format to FLT.
	#[strum(serialize = "float")]
	#[default]
	Float,

	/// 64-bit floating-point; this limits input sample format to DBL.
	#[strum(serialize = "double")]
	Double,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum VolumeReplaygain {
	/// Remove ReplayGain side data, ignoring its contents.
	#[strum(serialize = "drop")]
	#[default]
	Drop,

	/// Ignore ReplayGain side data, but leave it in the frame.
	#[strum(serialize = "ignore")]
	Ignore,

	/// Prefer the track gain, if present.
	#[strum(serialize = "track")]
	Track,

	/// Prefer the album gain, if present.
	#[strum(serialize = "album")]
	Album,
}

/// Adjust the input audio volume.
///
/// The output audio volume is given by the relation: `output_volume = volume * input_volume`.
/// Output values are clipped to the maximum value.
#[filter(name = "volume")]
pub struct Volume {
	/// Set audio volume expression. The default value is "1.0".
	#[ffarg(name = "volume", default = "1.0")]
	pub volume: String,

	/// The mathematical precision, which determines which input sample formats will be allowed,
	/// affecting the precision of the volume scaling.
	#[ffarg(omit_default)]
	pub precision: VolumePrecision,

	/// Choose the behaviour on encountering ReplayGain side data in input frames.
	#[ffarg(omit_default)]
	pub replaygain: VolumeReplaygain,

	/// Pre-amplification gain in dB to apply to the selected replaygain gain.
	#[ffarg(omit_default)]
	pub replaygain_preamp: f64,

	/// Prevent clipping by limiting the gain applied.
	#[ffarg(default = true, omit_default)]
	pub replaygain_noclip: bool,
}