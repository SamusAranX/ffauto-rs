use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum Mode {
	#[strum(serialize = "addition")]
	Addition,

	#[strum(serialize = "and")]
	And,

	#[strum(serialize = "average")]
	Average,

	#[strum(serialize = "bleach")]
	Bleach,

	#[strum(serialize = "burn")]
	Burn,

	#[strum(serialize = "darken")]
	Darken,

	#[strum(serialize = "difference")]
	Difference,

	#[strum(serialize = "divide")]
	Divide,

	#[strum(serialize = "dodge")]
	Dodge,

	#[strum(serialize = "exclusion")]
	Exclusion,

	#[strum(serialize = "extremity")]
	Extremity,

	#[strum(serialize = "freeze")]
	Freeze,

	#[strum(serialize = "geometric")]
	Geometric,

	#[strum(serialize = "glow")]
	Glow,

	#[strum(serialize = "grainextract")]
	GrainExtract,

	#[strum(serialize = "grainmerge")]
	GrainMerge,

	#[strum(serialize = "hardlight")]
	HardLight,

	#[strum(serialize = "hardmix")]
	HardMix,

	#[strum(serialize = "hardoverlay")]
	HardOverlay,

	#[strum(serialize = "harmonic")]
	Harmonic,

	#[strum(serialize = "heat")]
	Heat,

	#[strum(serialize = "interpolate")]
	Interpolate,

	#[strum(serialize = "lighten")]
	Lighten,

	#[strum(serialize = "linearlight")]
	LinearLight,

	#[strum(serialize = "multiply")]
	Multiply,

	#[strum(serialize = "multiply128")]
	Multiply128,

	#[strum(serialize = "negation")]
	Negation,

	#[strum(serialize = "normal")]
	#[default]
	Normal,

	#[strum(serialize = "or")]
	Or,

	#[strum(serialize = "overlay")]
	Overlay,

	#[strum(serialize = "phoenix")]
	Phoenix,

	#[strum(serialize = "pinlight")]
	PinLight,

	#[strum(serialize = "reflect")]
	Reflect,

	#[strum(serialize = "screen")]
	Screen,

	#[strum(serialize = "softdifference")]
	SoftDifference,

	#[strum(serialize = "softlight")]
	SoftLight,

	#[strum(serialize = "stain")]
	Stain,

	#[strum(serialize = "subtract")]
	Subtract,

	#[strum(serialize = "vividlight")]
	VividLight,

	#[strum(serialize = "xor")]
	Xor,
}

/// Blend two video frames into each other.
///
/// The blend filter takes two input streams and outputs one stream, the first input is the "top"
/// layer and second input is "bottom" layer. By default, the output terminates when the longest
/// input terminates.
#[filter(name = "blend")]
pub struct Blend {
	/// Set blend mode for pixel component 0. Default value is normal.
	#[ffarg(default = Mode::Normal, omit_default)]
	pub c0_mode: Mode,

	/// Set blend mode for pixel component 1. Default value is normal.
	#[ffarg(default = Mode::Normal, omit_default)]
	pub c1_mode: Mode,

	/// Set blend mode for pixel component 2. Default value is normal.
	#[ffarg(default = Mode::Normal, omit_default)]
	pub c2_mode: Mode,

	/// Set blend mode for pixel component 3. Default value is normal.
	#[ffarg(default = Mode::Normal, omit_default)]
	pub c3_mode: Mode,

	/// Set blend mode for all pixel components. Default value is normal.
	#[ffarg(default = Mode::Normal, omit_default)]
	pub all_mode: Mode,

	/// Set blend opacity for pixel component 0. Only used in combination with pixel component
	/// blend modes.
	pub c0_opacity: Option<f64>,

	/// Set blend opacity for pixel component 1. Only used in combination with pixel component
	/// blend modes.
	pub c1_opacity: Option<f64>,

	/// Set blend opacity for pixel component 2. Only used in combination with pixel component
	/// blend modes.
	pub c2_opacity: Option<f64>,

	/// Set blend opacity for pixel component 3. Only used in combination with pixel component
	/// blend modes.
	pub c3_opacity: Option<f64>,

	/// Set blend opacity for all pixel components. Only used in combination with pixel component
	/// blend modes.
	pub all_opacity: Option<f64>,
}

impl Blend {
	#[must_use]
	pub fn new(mode: Mode) -> Self {
		Self { all_mode: mode, ..Default::default() }
	}
}

#[test]
fn filter_blend() {
	let filter = Blend::default();
	assert_eq!(filter.to_string(), "blend");
}

#[test]
fn filter_blend_oarams() {
	let filter = Blend::new(Mode::SoftLight);
	assert_eq!(filter.to_string(), "blend=all_mode=softlight");
}
