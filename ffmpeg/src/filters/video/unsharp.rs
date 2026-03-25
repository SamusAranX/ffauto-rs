use ffmpeg_macro::filter;

/// Sharpen or blur the input video.
#[filter(name = "unsharp")]
pub struct Unsharp {
	/// Set the luma matrix horizontal size. It must be an odd integer between 3 and 23.
	#[ffarg(name = "lx", default = 5, omit_default)]
	pub luma_msize_x: u8,

	/// Set the luma matrix vertical size. It must be an odd integer between 3 and 23.
	#[ffarg(name = "ly", default = 5, omit_default)]
	pub luma_msize_y: u8,

	/// Set the luma effect strength. Negative values will blur the input video, while positive
	/// values will sharpen it. A value of zero will disable the effect. Reasonable values lay
	/// between -1.5 and 1.5.
	#[ffarg(name = "la", default = 1.0)]
	pub luma_amount: f64,

	/// Set the chroma matrix horizontal size. It must be an odd integer between 3 and 23.
	#[ffarg(name = "cx", default = 5, omit_default)]
	pub chroma_msize_x: u8,

	/// Set the chroma matrix vertical size. It must be an odd integer between 3 and 23.
	#[ffarg(name = "cy", default = 5, omit_default)]
	pub chroma_msize_y: u8,

	/// Set the chroma effect strength. Negative values will blur the input video, while positive
	/// values will sharpen it. A value of zero will disable the effect. Reasonable values lay
	/// between -1.5 and 1.5.
	#[ffarg(name = "ca")]
	pub chroma_amount: f64,

	/// Set the alpha matrix horizontal size. It must be an odd integer between 3 and 23.
	#[ffarg(name = "ax", default = 5, omit_default)]
	pub alpha_msize_x: u8,

	/// Set the alpha matrix vertical size. It must be an odd integer between 3 and 23.
	#[ffarg(name = "ay", default = 5, omit_default)]
	pub alpha_msize_y: u8,

	/// Set the alpha effect strength. Negative values will blur the input video, while positive
	/// values will sharpen it. A value of zero will disable the effect. Reasonable values lay
	/// between -1.5 and 1.5.
	#[ffarg(name = "aa")]
	pub alpha_amount: f64,
}
