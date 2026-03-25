use ffmpeg_macro::filter;

/// Convert the input video to one of the specified pixel formats.
/// Libavfilter will try to pick one that is suitable as input to the next filter.
#[filter(name = "format")]
pub struct Format {
	/// A list of pixel format names.
	#[ffarg(noname, separator = "|")]
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
	pub fn new<S: Into<String>>(pix_fmt: S) -> Self {
		Self {
			pix_fmts: vec![pix_fmt.into()],
			..Default::default()
		}
	}
}

#[test]
fn filter_format() {
	let filter = Format::new("rgb48");
	assert_eq!(filter.to_string(), "format=rgb48");
}