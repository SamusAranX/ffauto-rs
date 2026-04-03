use ffmpeg_macro::filter;

/// Normalize RGB video (aka histogram stretching, contrast stretching).
///
/// For each channel of each frame, the filter computes the input range and maps it linearly to
/// the user-specified output range. The output range defaults to the full dynamic range from pure
/// black to pure white.
///
/// Temporal smoothing can be used on the input range to reduce flickering (rapid changes in
/// brightness) caused when small dark or bright objects enter or leave the scene. This is similar
/// to the auto-exposure (automatic gain control) on a video camera, and, like a video camera, it
/// may cause a period of over- or under-exposure of the video.
///
/// The R,G,B channels can be normalized independently, which may cause some color shifting, or
/// linked together as a single channel, which prevents color shifting. Linked normalization
/// preserves hue. Independent normalization does not, so it can be used to remove some color
/// casts. Independent and linked normalization can be combined in any ratio.
#[filter(name = "normalize")]
pub struct Normalize {
	/// Color which defines the bottom of the output range. The minimum input value is mapped to
	/// this color. Specifying white for blackpt and black for whitept will give color-inverted,
	/// normalized video. Shades of grey can be used to reduce the dynamic range (contrast).
	/// Specifying saturated colors here can create some interesting effects.
	#[ffarg(default = "black")]
	pub blackpt: String,

	/// Color which defines the top of the output range. The maximum input value is mapped to this
	/// color. Specifying white for blackpt and black for whitept will give color-inverted,
	/// normalized video. Shades of grey can be used to reduce the dynamic range (contrast).
	/// Specifying saturated colors here can create some interesting effects.
	#[ffarg(default = "white")]
	pub whitept: String,

	/// The number of previous frames to use for temporal smoothing. The input range of each
	/// channel is smoothed using a rolling average over the current frame and the smoothing
	/// previous frames.
	pub smoothing: u32,

	/// Controls the ratio of independent (color shifting) channel normalization to linked (color
	/// preserving) normalization. 0.0 is fully linked, 1.0 is fully independent.
	#[ffarg(default = 1.0)]
	pub independence: f64,

	/// Overall strength of the filter. 1.0 is full strength. 0.0 is a rather expensive no-op.
	#[ffarg(default = 1.0)]
	pub strength: f64,
}

impl Normalize {
	#[must_use]
	pub fn new_smooth(smoothing: u32) -> Self {
		Self { smoothing, ..Default::default() }
	}
}
