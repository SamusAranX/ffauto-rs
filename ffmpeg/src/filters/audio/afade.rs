use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum AfadeType {
	/// Fade-in effect.
	#[strum(serialize = "in")]
	#[default]
	In,

	/// Fade-out effect.
	#[strum(serialize = "out")]
	Out,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum AfadeCurve {
	/// Triangular, linear slope.
	#[strum(serialize = "tri")]
	#[default]
	Triangular,

	/// Quarter of sine wave.
	#[strum(serialize = "qsin")]
	QuarterSine,

	/// Half of sine wave.
	#[strum(serialize = "hsin")]
	HalfSine,

	/// Exponential sine wave.
	#[strum(serialize = "esin")]
	ExponentialSine,

	/// Logarithmic.
	#[strum(serialize = "log")]
	Logarithmic,

	/// Inverted parabola.
	#[strum(serialize = "ipar")]
	InvertedParabola,

	/// Quadratic.
	#[strum(serialize = "qua")]
	Quadratic,

	/// Cubic.
	#[strum(serialize = "cub")]
	Cubic,

	/// Square root.
	#[strum(serialize = "squ")]
	SquareRoot,

	/// Cubic root.
	#[strum(serialize = "cbr")]
	CubicRoot,

	/// Parabola.
	#[strum(serialize = "par")]
	Parabola,

	/// Exponential.
	#[strum(serialize = "exp")]
	Exponential,

	/// Inverted quarter of sine wave.
	#[strum(serialize = "iqsin")]
	InvertedQuarterSine,

	/// Inverted half of sine wave.
	#[strum(serialize = "ihsin")]
	InvertedHalfSine,

	/// Double-exponential seat.
	#[strum(serialize = "dese")]
	DoubleExponentialSeat,

	/// Double-exponential sigmoid.
	#[strum(serialize = "desi")]
	DoubleExponentialSigmoid,

	/// Logistic sigmoid.
	#[strum(serialize = "losi")]
	LogisticSigmoid,

	/// Sine cardinal function.
	#[strum(serialize = "sinc")]
	Sinc,

	/// Inverted sine cardinal function.
	#[strum(serialize = "isinc")]
	InvertedSinc,

	/// Quartic.
	#[strum(serialize = "quat")]
	Quartic,

	/// Quartic root.
	#[strum(serialize = "quatr")]
	QuarticRoot,

	/// Squared quarter of sine wave.
	#[strum(serialize = "qsin2")]
	SquaredQuarterSine,

	/// Squared half of sine wave.
	#[strum(serialize = "hsin2")]
	SquaredHalfSine,

	/// No fade applied.
	#[strum(serialize = "nofade")]
	NoFade,
}

/// Apply fade-in/out effect to input audio.
#[filter(name = "afade")]
pub struct Afade {
	/// Specify the effect type, either fade-in or fade-out.
	#[ffarg(name = "type", default = AfadeType::In)]
	pub r#type: AfadeType,

	// we're not using the sample-based timings so these are elided here
	/// Specify the start time of the fade effect. The value must be specified as a time duration.
	/// If set, this option is used instead of start_sample.
	#[ffarg(default = 0.0, omit_default)]
	pub start_time: f64,

	/// Specify the duration of the fade effect. At the end of the fade-in effect the output audio
	/// will have the same volume as the input audio; at the end of the fade-out transition the
	/// output audio will be silence. If set, this option is used instead of nb_samples.
	#[ffarg(default = 0.0, omit_default)]
	pub duration: f64,

	/// Set curve for fade transition.

	pub curve: AfadeCurve,

	/// Set the initial gain for fade-in or final gain for fade-out.
	#[ffarg(omit_default)]
	pub silence: f64,

	/// Set the initial gain for fade-out or final gain for fade-in.
	#[ffarg(default = 1.0, omit_default)]
	pub unity: f64,
}

#[test]
fn filter_afade() {
	let filter = Afade::default();
	assert_eq!(filter.to_string(), "afade=type=in:curve=tri");
}

#[test]
fn filter_afade_in_params() {
	let filter = Afade {
		r#type: AfadeType::In,
		start_time: 2.0,
		duration: 5.0,
		..Default::default()
	};
	assert_eq!(filter.to_string(), "afade=type=in:start_time=2:duration=5:curve=tri");
}

#[test]
fn filter_afade_out_params() {
	let filter = Afade {
		r#type: AfadeType::Out,
		start_time: 2.0,
		duration: 5.0,
		..Default::default()
	};
	assert_eq!(filter.to_string(), "afade=type=out:start_time=2:duration=5:curve=tri");
}
