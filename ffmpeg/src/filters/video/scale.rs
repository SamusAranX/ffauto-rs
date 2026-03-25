use ffmpeg_macro::filter;

// TODO: tie scale algorithm into the filter struct somehow

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ScaleAlgorithm {
	/// Fast bilinear scaling algorithm.
	#[strum(serialize = "fast_bilinear")]
	FastBilinear,

	/// Bilinear scaling algorithm.
	#[strum(serialize = "bilinear")]
	Bilinear,

	/// Bicubic scaling algorithm.
	#[strum(serialize = "bicubic")]
	#[default]
	Bicubic,

	/// Nearest neighbor rescaling algorithm.
	#[strum(serialize = "neighbor")]
	Neighbor,

	/// Averaging area rescaling algorithm.
	#[strum(serialize = "area")]
	Area,

	/// Bicubic scaling algorithm for the luma component, bilinear for chroma components.
	#[strum(serialize = "bicublin")]
	Bicublin,

	/// Gaussian rescaling algorithm.
	#[strum(serialize = "gauss")]
	Gauss,

	/// Sinc rescaling algorithm.
	#[strum(serialize = "sinc")]
	Sinc,

	/// Lanczos rescaling algorithm.
	/// The default width (alpha) is 3 and can be changed by setting param0.
	#[strum(serialize = "lanczos")]
	Lanczos,

	/// Natural bicubic spline rescaling algorithm.
	#[strum(serialize = "spline")]
	Spline,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScaleForceOriginalAspectRatio {
	/// Scale the video as specified and disable this feature.
	#[strum(serialize = "disable")]
	#[default]
	Disable,

	/// The output video dimensions will automatically be decreased if needed.
	#[strum(serialize = "decrease")]
	Decrease,

	/// The output video dimensions will automatically be increased if needed.
	#[strum(serialize = "increase")]
	Increase,
}

/// Scale (resize) the input video, using the libswscale library.
///
/// The scale filter forces the output display aspect ratio to be the same of the input, by
/// changing the output sample aspect ratio.
///
/// If the input image format is different from the format requested by the next filter, the scale
/// filter will convert the input to the requested format.
#[filter(name = "scale")]
pub struct Scale {
	/// The output video width expression. Default value is the input dimension. If the value is 0,
	/// the input width is used for the output. If one and only one of w/h is -n with n >= 1, the
	/// scale filter will use a value that maintains the aspect ratio of the input image, calculated
	/// from the other specified dimension, divisible by n.
	#[ffarg(name = "w")]
	pub width: i32,

	/// The output video height expression. Default value is the input dimension. If the value is
	/// 0, the input height is used for the output. If one and only one of w/h is -n with n >= 1,
	/// the scale filter will use a value that maintains the aspect ratio of the input image,
	/// calculated from the other specified dimension, divisible by n.
	#[ffarg(name = "h")]
	pub height: i32,

	/// Set the video scaling algorithm.
	pub scale_algorithm: ScaleAlgorithm,

	/// Set libswscale scaling flags. If not explicitly specified the filter applies the default flags.
	#[ffarg(separator = "+", default_from = scale_algorithm, extra_flags = ["accurate_rnd", "full_chroma_int", "full_chroma_inp"])]
	pub flags: Vec<String>,

	/// Set libswscale input parameters for scaling algorithms that need them. If not explicitly
	/// specified the filter applies empty parameters.
	#[ffarg(omit_default)]
	pub param0: String,

	/// Set libswscale input parameters for scaling algorithms that need them. If not explicitly
	/// specified the filter applies empty parameters.
	#[ffarg(omit_default)]
	pub param1: String,

	/// Enable decreasing or increasing output video width or height if necessary to keep the
	/// original aspect ratio.
	#[ffarg(omit_default)]
	pub force_original_aspect_ratio: ScaleForceOriginalAspectRatio,

	/// Ensures that both the output dimensions, width and height, are divisible by the given integer
	/// when used together with force_original_aspect_ratio. This works similar to using -n in the w and h options.
	// This option respects the value set for force_original_aspect_ratio, increasing or decreasing the resolution accordingly.
	// The video’s aspect ratio may be slightly modified.
	// This option can be handy if you need to have a video fit within or exceed a defined resolution using
	// force_original_aspect_ratio but also have encoder restrictions on width or height divisibility.
	#[ffarg(default = 1, omit_default)]
	pub force_divisible_by: u8,

	/// When enabled, the output SAR is reset to 1. Additionally, if proportional scaling is
	/// requested, the input DAR is taken into account and the output is scaled to produce square
	/// pixels.
	#[ffarg(omit_default)]
	pub reset_sar: bool,
}

#[test]
fn filter_scale() {
	let filter = Scale::default();
	assert_eq!(
		filter.to_string(),
		"scale=w=0:h=0:flags=bicubic+accurate_rnd+full_chroma_int+full_chroma_inp"
	);
}

#[test]
fn filter_scale_params() {
	let filter = Scale {
		width: 1920,
		height: 1080,
		scale_algorithm: ScaleAlgorithm::Spline,
		force_divisible_by: 2,
		..Default::default()
	};
	assert_eq!(
		filter.to_string(),
		"scale=w=1920:h=1080:flags=spline+accurate_rnd+full_chroma_int+full_chroma_inp:force_divisible_by=2"
	);
}
