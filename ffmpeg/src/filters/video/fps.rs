use ffmpeg_macro::filter;

/// Convert the video to specified constant frame rate by duplicating or dropping frames as necessary.
#[filter(name = "fps")]
pub struct Fps {
	/// The desired output frame rate.
	#[ffarg(default = 25.0)]
	pub fps: f64,
}