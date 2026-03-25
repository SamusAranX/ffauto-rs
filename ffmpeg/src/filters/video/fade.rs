use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum Type {
	#[default]
	#[strum(serialize = "in")]
	In,
	#[strum(serialize = "out")]
	Out,
}

/// Apply a fade-in/out effect to the input video.
#[filter(name = "fade")]
pub struct Fade {
	/// The effect type can be either `Type::In` for a fade-in, or `Type::Out` for a fade-out effect.
	/// Default is `Type::In`.
	#[ffarg(name = "type", default = Type::In)]
	pub r#type: Type,

	// we're not using the sample-based timings so these are elided here
	/// If set to true, fade only alpha channel, if one exists on the input.
	/// Default value is false.
	#[ffarg(default = false, omit_default)]
	pub alpha: bool,

	/// Specify the timestamp (in seconds) of the frame to start to apply the fade effect.
	/// If both `start_frame` and `start_time` are specified, the fade will start at whichever comes last.
	/// Default is 0.
	#[ffarg(default = 0.0, omit_default)]
	pub start_time: f64,

	/// The number of seconds for which the fade effect has to last.
	/// At the end of the fade-in effect the output video will have the same intensity as the input video,
	/// at the end of the fade-out transition the output video will be filled with the selected color.
	/// If both `duration` and `nb_frames` are specified, `duration` is used.
	/// Default is 0 (`nb_frames` is used by default).
	#[ffarg(default = 0.0, omit_default)]
	pub duration: f64,

	/// Specify the color of the fade.
	/// Default is "black".
	#[ffarg(default = "black", omit_default)]
	pub color: String,
}

#[test]
fn filter_fade() {
	let filter = Fade::default();
	assert_eq!(filter.to_string(), "fade=type=in");
}

#[test]
fn filter_fade_in_params() {
	let filter = Fade {
		r#type: Type::In,
		start_time: 2.0,
		duration: 5.0,
		..Default::default()
	};
	assert_eq!(filter.to_string(), "fade=type=in:start_time=2:duration=5");
}

#[test]
fn filter_fade_out_params() {
	let filter = Fade {
		r#type: Type::Out,
		start_time: 2.0,
		duration: 5.0,
		..Default::default()
	};
	assert_eq!(filter.to_string(), "fade=type=out:start_time=2:duration=5");
}
