use ffmpeg_macro::filter;

/// Provides a uniformly colored input.
#[filter(name = "color")]
pub struct Color {
	/// Specify the color of the source.
	#[ffarg(name = "color")]
	pub color: String,

	/// Specify the size of the sourced video. The default value is 320x240.
	#[ffarg(name = "size", default = "320x240")]
	pub size: String,

	/// Specify the frame rate of the sourced video, as the number of frames generated per second.
	/// It has to be a string in the format frame_rate_num/frame_rate_den, an integer number, a
	/// floating point number or a valid video frame rate abbreviation.
	#[ffarg(name = "rate", default = "25")]
	pub rate: String,

	/// Set the duration of the sourced video. If not specified, or the expressed duration is
	/// negative, the video is supposed to be generated forever. If the specified duration is not a
	/// multiple of the frame duration, it will be rounded up.
	#[ffarg(omit_default)]
	pub duration: f64,

	/// Set the sample aspect ratio of the sourced video.
	#[ffarg(omit_default)]
	pub sar: String,
}