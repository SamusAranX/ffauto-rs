use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ZscaleDither {
	#[strum(serialize = "none")]
	#[default]
	None,

	#[strum(serialize = "ordered")]
	Ordered,

	#[strum(serialize = "random")]
	Random,

	#[strum(serialize = "error_diffusion")]
	ErrorDiffusion,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ZscaleFilter {
	#[strum(serialize = "point")]
	Point,

	#[strum(serialize = "bilinear")]
	#[default]
	Bilinear,

	#[strum(serialize = "bicubic")]
	Bicubic,

	#[strum(serialize = "spline16")]
	Spline16,

	#[strum(serialize = "spline36")]
	Spline36,

	#[strum(serialize = "spline64")]
	Spline64,

	#[strum(serialize = "lanczos")]
	Lanczos,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ZscaleRange {
	/// Same as input.
	#[strum(serialize = "input")]
	#[default]
	Input,

	#[strum(serialize = "limited")]
	Limited,

	#[strum(serialize = "full")]
	Full,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ZscalePrimaries {
	/// Same as input.
	#[strum(serialize = "input")]
	#[default]
	Input,

	#[strum(serialize = "709")]
	Bt709,

	#[strum(serialize = "unspecified")]
	Unspecified,

	#[strum(serialize = "170m")]
	Smpte170m,

	#[strum(serialize = "240m")]
	Smpte240m,

	#[strum(serialize = "2020")]
	Bt2020,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ZscaleTransfer {
	/// Same as input.
	#[strum(serialize = "input")]
	#[default]
	Input,

	#[strum(serialize = "709")]
	Bt709,

	#[strum(serialize = "unspecified")]
	Unspecified,

	#[strum(serialize = "601")]
	Bt601,

	#[strum(serialize = "linear")]
	Linear,

	#[strum(serialize = "2020_10")]
	Bt202010,

	#[strum(serialize = "2020_12")]
	Bt202012,

	#[strum(serialize = "smpte2084")]
	Smpte2084,

	#[strum(serialize = "iec61966-2-1")]
	Iec6196621,

	#[strum(serialize = "arib-std-b67")]
	AribStdB67,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ZscaleTransferIn {
	/// Same as input.
	#[strum(serialize = "input")]
	#[default]
	Input,

	#[strum(serialize = "709")]
	Bt709,

	#[strum(serialize = "unspecified")]
	Unspecified,

	#[strum(serialize = "601")]
	Bt601,

	#[strum(serialize = "linear")]
	Linear,

	#[strum(serialize = "2020_10")]
	Bt202010,

	#[strum(serialize = "2020_12")]
	Bt202012,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ZscaleMatrix {
	/// Same as input.
	#[strum(serialize = "input")]
	#[default]
	Input,

	#[strum(serialize = "709")]
	Bt709,

	#[strum(serialize = "unspecified")]
	Unspecified,

	#[strum(serialize = "470bg")]
	Bt470bg,

	#[strum(serialize = "170m")]
	Smpte170m,

	#[strum(serialize = "2020_ncl")]
	Bt2020Ncl,

	#[strum(serialize = "2020_cl")]
	Bt2020Cl,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ZscaleChromaLoc {
	/// Same as input.
	#[strum(serialize = "input")]
	#[default]
	Input,

	#[strum(serialize = "left")]
	Left,

	#[strum(serialize = "center")]
	Center,

	#[strum(serialize = "topleft")]
	TopLeft,

	#[strum(serialize = "top")]
	Top,

	#[strum(serialize = "bottomleft")]
	BottomLeft,

	#[strum(serialize = "bottom")]
	Bottom,
}

/// Scale (resize) the input video, using the z.lib library. The zscale filter forces the output
/// display aspect ratio to be the same as the input, by changing the output sample aspect ratio.
///
/// If the input image format is different from the format requested by the next filter, the zscale
/// filter will convert the input to the requested format.
#[filter(name = "zscale")]
pub struct Zscale {
	/// The output video width expression. Default value is the input dimension. If the value is 0,
	/// the input width is used for the output. If one and only one of w/h is -n with n >= 1, the
	/// zscale filter will use a value that maintains the aspect ratio of the input image,
	/// calculated from the other specified dimension, divisible by n.
	#[ffarg(name = "w")]
	pub width: String,

	/// The output video height expression. Default value is the input dimension. If the value is
	/// 0, the input height is used for the output. If one and only one of w/h is -n with n >= 1,
	/// the zscale filter will use a value that maintains the aspect ratio of the input image,
	/// calculated from the other specified dimension, divisible by n.
	#[ffarg(name = "h")]
	pub height: String,

	/// Set the video size (width and height together).
	#[ffarg(name = "s")]
	pub size: String,

	/// Set the dither type.
	#[ffarg()]
	pub dither: ZscaleDither,

	/// Set the resize filter type.
	#[ffarg()]
	pub filter: ZscaleFilter,

	/// Set the output color range.
	#[ffarg()]
	pub range: ZscaleRange,

	/// Set the output color primaries.
	#[ffarg()]
	pub primaries: ZscalePrimaries,

	/// Set the output transfer characteristics.
	#[ffarg()]
	pub transfer: ZscaleTransfer,

	/// Set the output colorspace matrix.
	#[ffarg()]
	pub matrix: ZscaleMatrix,

	/// Set the input color range.
	#[ffarg(name = "rangein")]
	pub range_in: ZscaleRange,

	/// Set the input color primaries.
	#[ffarg(name = "primariesin")]
	pub primaries_in: ZscalePrimaries,

	/// Set the input transfer characteristics.
	#[ffarg(name = "transferin")]
	pub transfer_in: ZscaleTransferIn,

	/// Set the input colorspace matrix.
	#[ffarg(name = "matrixin")]
	pub matrix_in: ZscaleMatrix,

	/// Set the output chroma location.
	#[ffarg(name = "chromal")]
	pub chroma_loc: ZscaleChromaLoc,

	/// Set the input chroma location.
	#[ffarg(name = "chromalin")]
	pub chroma_loc_in: ZscaleChromaLoc,

	/// Set the nominal peak luminance.
	#[ffarg(omit_default)]
	pub npl: f64,
}