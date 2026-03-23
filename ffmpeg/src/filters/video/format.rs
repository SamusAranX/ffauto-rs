use ffmpeg_macro::filter;

/// Convert the input video to one of the specified pixel formats.
/// Libavfilter will try to pick one that is suitable as input to the next filter.
#[filter(name = "format")]
pub struct Format {
	/// A list of pixel format names.
	#[ffarg(separator = "|")]
	pub pix_fmts: Vec<String>,

	/// A list of pixel format names.
	#[ffarg(separator = "|")]
	pub color_spaces: Vec<String>,

	/// A list of pixel format names.
	#[ffarg(separator = "|")]
	pub color_ranges: Vec<String>,

	/// A list of pixel format names.
	#[ffarg(separator = "|")]
	pub alpha_modes: Vec<String>,
}

impl Format {
	#[must_use]
	pub fn new(pix_fmt: String) -> Self {
		Self {
			pix_fmts: vec![pix_fmt],
			..Default::default()
		}
	}
}