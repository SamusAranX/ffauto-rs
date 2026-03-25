use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum PrintFormat {
	/// Print a human-readable summary.
	#[strum(serialize = "summary")]
	Summary,

	/// Print stats in JSON format.
	#[strum(serialize = "json")]
	#[default] // the actual default is "none" but we always want json for this
	Json,

	/// Do not print stats.
	#[strum(serialize = "none")]
	None,
}

/// EBU R128 loudness normalization. Includes both dynamic and linear normalization modes. Support
/// for both single pass (livestreams, files) and double pass (files) modes. This algorithm can
/// target IL, LRA, and maximum true peak. In dynamic mode, to accurately detect true peaks, the
/// audio stream will be upsampled to 192 kHz.
#[filter(name = "loudnorm")]
pub struct Loudnorm {
	/// Set integrated loudness target. Range is -70.0 - -5.0.
	#[ffarg(name = "i", default = -24.0)]
	pub integrated_loudness: f64,

	/// Set loudness range target. Range is 1.0 - 50.0.
	#[ffarg(name = "lra", default = 7.0)]
	pub loudness_range: f64,

	/// Set maximum true peak. Range is -9.0 - +0.0.
	#[ffarg(name = "tp", default = -2.0)]
	pub true_peak: f64,

	/// Measured integrated loudness of input file. Range is -99.0 - +0.0.
	#[ffarg(name = "measured_i")]
	pub measured_integrated_loudness: f64,

	/// Measured loudness range of input file. Range is 0.0 - 99.0.
	#[ffarg(name = "measured_lra")]
	pub measured_loudness_range: f64,

	/// Measured true peak of input file. Range is -99.0 - +99.0.
	#[ffarg(name = "measured_tp")]
	pub measured_true_peak: f64,

	/// Measured threshold of input file. Range is -99.0 - +0.0.
	#[ffarg(name = "measured_thresh")]
	pub measured_threshold: f64,

	/// Set offset gain. Gain is applied before the true-peak limiter. Range is -99.0 - +99.0.
	pub offset: f64,

	/// Normalize by linearly scaling the source audio. measured_I, measured_LRA, measured_TP, and
	/// measured_thresh must all be specified. Target LRA shouldn't be lower than source LRA and
	/// the change in integrated loudness shouldn't result in a true peak which exceeds the target
	/// TP. If any of these conditions aren't met, normalization mode will revert to dynamic.
	#[ffarg(default = true, omit_default)]
	pub linear: bool,

	/// Treat mono input files as "dual-mono". If a mono file is intended for playback on a stereo
	/// system, its EBU R128 measurement will be perceptually incorrect. If set to true, this
	/// option will compensate for this effect. Multi-channel input files are not affected by this
	/// option.
	#[ffarg(omit_default)]
	pub dual_mono: bool,

	/// Set print format for stats.
	pub print_format: PrintFormat,

	/// Write stats to specified file. Format is controlled by print_format, which must be set.
	/// Specify "-" to write to standard output.
	pub stats_file: String,
}
