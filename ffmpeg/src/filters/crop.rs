use ffmpeg_macro::filter;

/// Crop the input video to given dimensions.
#[filter(name = "crop")]
pub struct Crop {
	/// The width of the output video. This expression is evaluated only once during the filter
	/// configuration, or when the 'w' or 'out_w' command is sent.
	#[ffarg(name = "out_w")]
	pub width: i32,

	/// The height of the output video. This expression is evaluated only once during the filter
	/// configuration, or when the 'h' or 'out_h' command is sent.
	#[ffarg(name = "out_h")]
	pub height: i32,

	/// The horizontal position, in the input video, of the left edge of the output video.
	/// This expression is evaluated per-frame.
	#[ffarg()]
	pub x: i32,

	/// The vertical position, in the input video, of the top edge of the output video.
	/// This expression is evaluated per-frame.
	#[ffarg()]
	pub y: i32,

	/// If set to `true` will force the output display aspect ratio to be the same of the input,
	/// by changing the output sample aspect ratio.
	#[ffarg(default = false, omit_default)]
	pub keep_aspect: bool,

	/// Enable exact cropping. If enabled, subsampled videos will be cropped at exact
	/// width/height/x/y as specified and will not be rounded to nearest smaller value.
	#[ffarg(default = false, omit_default)]
	pub exact: bool,
}

impl Crop {
	pub fn new(width: i32, height: i32, x: i32, y: i32) -> Self {
		Self {
			width,
			height,
			x,
			y,
			..Default::default()
		}
	}
}