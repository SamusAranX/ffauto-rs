use ffmpeg_macro::filter;

/// Convert the video to specified constant frame rate by duplicating or dropping frames as necessary.
#[filter(name = "fps")]
pub struct Fps {
	/// The desired output frame rate.
	#[ffarg(default = 25.0)]
	pub fps: f64,
}

impl Fps {
	#[must_use]
	pub fn new(fps: f64) -> Self {
		Self { fps }
	}

	/// Takes an input framerate and a target framerate and returns a new framerate
	/// that's been divided down enough to be smaller than or equal to the input framerate.
	/// If the input framerate is already smaller than or equal to the input framerate,
	/// this returns the input framerate unchanged.
	/// Example: If `fps_in` is 60 and `fps_target` is 25, this returns 20 fps.
	#[must_use]
	pub fn target(fps_in: f64, fps_target: f64) -> Self {
		if fps_in <= fps_target {
			return Self::new(fps_in);
		}

		let divisor = (fps_in / fps_target).ceil();
		Self::new(fps_in / divisor)
	}
}
