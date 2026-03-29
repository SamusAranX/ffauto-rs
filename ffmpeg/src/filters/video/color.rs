use ffmpeg_macro::filter;

/// Provides a uniformly colored input.
#[filter(name = "color")]
pub struct Color {
	/// Specify the color of the source.
	#[ffarg(name = "color", default = "magenta")]
	pub color: String,

	// TODO: investigate how to have width/height fields that are synthesized into one size field at Display time
	/// Specify the size of the sourced video. The default value is 320x240.
	#[ffarg(name = "size", default = "320x240")]
	pub size: String,

	/// Specify the frame rate of the sourced video, as the number of frames generated per second.
	/// It has to be a string in the format frame_rate_num/frame_rate_den, an integer number, a
	/// floating point number or a valid video frame rate abbreviation.
	#[ffarg(name = "rate", default = 25.0)]
	pub rate: f64,

	/// Set the duration of the sourced video. If not specified, or the expressed duration is
	/// negative, the video is supposed to be generated forever. If the specified duration is not a
	/// multiple of the frame duration, it will be rounded up.
	#[ffarg(omit_default)]
	pub duration: f64,

	/// Set the sample aspect ratio of the sourced video.
	#[ffarg(omit_default)]
	pub sar: String,
}

impl Color {
	pub fn new<S: Into<String>>(width: u64, height: u64, color: S) -> Self {
		Self {
			color: color.into(),
			size: format!("{width}x{height}"),
			..Default::default()
		}
	}

	pub fn pixel<S: Into<String>>(color: S) -> Self {
		Self {
			color: color.into(),
			size: "1x1".to_string(),
			rate: 1.0,
			..Default::default()
		}
	}
}

#[test]
fn filter_color() {
	let filter = Color::default();
	assert_eq!(filter.to_string(), "color=color=magenta:size=320x240:rate=25");
}

#[test]
fn filter_color_params() {
	let filter = Color {
		color: "black".to_string(),
		size: "1920x1080".to_string(),
		rate: 60.0,
		duration: 10.0,
		..Default::default()
	};
	assert_eq!(
		filter.to_string(),
		"color=color=black:size=1920x1080:rate=60:duration=10"
	);
}
