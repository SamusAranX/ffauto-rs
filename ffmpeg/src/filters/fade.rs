use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[derive(strum::Display, strum::EnumString)]
pub enum FadeType {
	#[default]
	#[strum(serialize = "in")]
	In,
	#[strum(serialize = "out")]
	Out,
}

/// Apply a fade-in/out effect to the input video.
#[filter(name = "fade")]
pub struct Fade {
	/// The effect type can be either `FadeType::In` for a fade-in, or `FadeType::Out` for a fade-out effect.
	/// Default is `FadeType::In`.
	#[ffarg(name = "type", default = FadeType::In)]
	pub r#type: FadeType,

	/// Specify the number of the frame to start applying the fade effect at.
	/// Default is 0.
	#[ffarg(default = 0)]
	pub start_frame: u32,

	/// The number of frames that the fade effect lasts.
	/// At the end of the fade-in effect, the output video will have the same intensity as the input video.
	/// At the end of the fade-out transition, the output video will be filled with the selected color.
	/// Default is 25.
	#[ffarg(default = 25)]
	pub nb_frames: u32,

	/// If set to true, fade only alpha channel, if one exists on the input.
	/// Default value is false.
	#[ffarg(default = false)]
	pub alpha: bool,

	/// Specify the timestamp (in seconds) of the frame to start to apply the fade effect.
	/// If both `start_frame` and `start_time` are specified, the fade will start at whichever comes last.
	/// Default is 0.
	#[ffarg(default = 0.0)]
	pub start_time: f64,

	/// The number of seconds for which the fade effect has to last.
	/// At the end of the fade-in effect the output video will have the same intensity as the input video,
	/// at the end of the fade-out transition the output video will be filled with the selected color.
	/// If both `duration` and `nb_frames` are specified, `duration` is used.
	/// Default is 0 (`nb_frames` is used by default).
	#[ffarg(default = 0.0)]
	pub duration: f64,

	/// Specify the color of the fade.
	/// Default is "black".
	#[ffarg(default = "black")]
	pub color: String,
}
