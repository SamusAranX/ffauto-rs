use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum Mode {
	/// Detect black pixels surrounding the playing video. For fine control use option limit.
	#[strum(serialize = "black")]
	#[default]
	Black,

	/// Detect the playing video by the motion vectors inside the video and scanning for edge
	/// pixels typically forming the border of a playing video.
	#[strum(serialize = "mvedges")]
	MvEdges,
	// TODO: MvEdges needs either -flags2 +export_mvs or the mestimate filter before the cropdetect filter in the chain
}

/// Auto-detect the crop size.
///
/// It calculates the necessary cropping parameters and prints the recommended parameters via the
/// logging system. The detected dimensions correspond to the non-black or video area of the input
/// video according to mode.
#[filter(name = "cropdetect")]
pub struct Cropdetect {
	/// Depending on mode crop detection is based on either the mere black value of surrounding
	/// pixels or a combination of motion vectors and edge pixels.
	pub mode: Mode,

	/// Set higher black value threshold, which can be optionally specified from nothing (0) to
	/// everything (255 for 8-bit based formats). An intensity value greater to the set value is
	/// considered non-black. You can also specify a value between 0.0 and 1.0 which will be
	/// scaled depending on the bitdepth of the pixel format.
	#[ffarg(default = 0.09411765)]
	pub limit: f64,

	/// The value which the width/height should be divisible by. The offset is automatically
	/// adjusted to center the video. Use 2 to get only even dimensions (needed for 4:2:2 video).
	/// 16 is best when encoding to most video codecs.
	#[ffarg(default = 16)]
	pub round: u32,

	/// Set the number of initial frames for which evaluation is skipped.
	#[ffarg(default = 2)]
	pub skip: u32,

	/// Set the counter that determines after how many frames cropdetect will reset the previously
	/// detected largest video area and start over to detect the current optimal crop area. 0
	/// indicates 'never reset', and returns the largest area encountered during playback. This can
	/// be useful when channel logos distort the video area.
	#[ffarg(name = "reset_count")]
	pub reset_count: u32,

	/// Set motion in pixel units as threshold for motion detection.
	#[ffarg(default = 8)]
	pub mv_threshold: u32,

	/// Set low threshold value used by the Canny thresholding algorithm. The low threshold
	/// selects the "weak" edge pixels which are connected to "strong" edge pixels selected by the
	/// high threshold. Must be in the range [0, 1] and lesser or equal to high.
	#[ffarg(default = 0.01960784)]
	pub low: f64,

	/// Set high threshold value used by the Canny thresholding algorithm. The high threshold
	/// selects the "strong" edge pixels, which are then connected through 8-connectivity with the
	/// "weak" edge pixels selected by the low threshold. Must be in the range [0, 1].
	#[ffarg(default = 0.05882353)]
	pub high: f64,
}

impl Cropdetect {
	#[must_use]
	pub fn new(mode: Mode) -> Self {
		Self { mode, ..Default::default() }
	}
}
