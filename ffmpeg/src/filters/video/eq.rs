use ffmpeg_macro::filter;

/// Set brightness, contrast, saturation and approximate gamma adjustment.
#[filter(name = "eq")]
pub struct Eq {
	/// Set the contrast expression. The value must be a float value in range -1000.0 to 1000.0.
	#[ffarg(default = 1.0)]
	pub contrast: f64,

	/// Set the brightness expression. The value must be a float value in range -1.0 to 1.0.
	#[ffarg()]
	pub brightness: f64,

	/// Set the saturation expression. The value must be a float in range 0.0 to 3.0.
	#[ffarg(default = 1.0)]
	pub saturation: f64,

	/// Set the gamma expression. The value must be a float in range 0.1 to 10.0.
	#[ffarg(default = 1.0)]
	pub gamma: f64,
}