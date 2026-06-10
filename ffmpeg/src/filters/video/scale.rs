use crate::ffmpeg::size::parse_ffmpeg_size;
use anyhow::Result;
use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, clap::ValueEnum, strum::Display, strum::EnumString)]
pub enum Algorithm {
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
pub enum ForceOriginalAspectRatio {
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

/// ICC rendering intent used when transforming between different color spaces.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum Intent {
	/// Use a perceptually guided tone and gamut mapping curve. The exact details of the mapping
	/// used may change at any time and should not be relied on as stable. This intent is
	/// recommended for final viewing of image/video content in typical viewing settings.
	#[strum(serialize = "perceptual")]
	Perceptual,

	/// Statically clip out-of-gamut colors using a colorimetric clipping curve which attempts to
	/// find the colorimetrically least dissimilar in-gamut color. This intent performs white point
	/// adaptation and black point adaptation. This is the default. This intent is recommended
	/// wherever faithful color reproduction is of the utmost importance, even at the cost of clipping.
	#[strum(serialize = "relative_colorimetric")]
	#[default]
	RelativeColorimetric,

	/// Hard clip out-of-gamut colors with no attempt at white or black point reproduction. This
	/// intent will reproduce in-gamut colors 1:1 on the output display as they would appear on the
	/// reference display, assuming the output display is appropriately calibrated.
	#[strum(serialize = "absolute_colorimetric")]
	AbsoluteColorimetric,

	/// Performs saturation mapping - that is, stretches the input color volume directly onto the
	/// output color volume, in non-linear fashion that preserves the original signal appearance as
	/// much as possible. This intent is recommended for signal content evaluation, as it will not
	/// lead to any clipping. It is roughly analogous to not performing any color mapping, although
	/// it still takes into account the mastering display primaries and any differences in encoding TRC.
	#[strum(serialize = "saturation")]
	Saturation,
}

/// YCbCr color space type used by the scale filter's in/out color matrix options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ColorMatrix {
	/// Choose automatically.
	#[strum(serialize = "auto")]
	Auto,

	/// Format conforming to International Telecommunication Union (ITU) Recommendation BT.709.
	#[strum(serialize = "bt709")]
	Bt709,

	/// Color space conforming to the United States Federal Communications Commission (FCC) Code of
	/// Federal Regulations (CFR) Title 47 (2003) 73.682 (a).
	#[strum(serialize = "fcc")]
	Fcc,

	/// Color space conforming to ITU Radiocommunication Sector (ITU-R) Recommendation BT.601.
	#[strum(serialize = "bt601")]
	Bt601,

	/// Color space conforming to ITU-R Rec. BT.470-6 (1998) Systems B, B1, and G.
	#[strum(serialize = "bt470")]
	Bt470,

	/// Color space conforming to Society of Motion Picture and Television Engineers (SMPTE) ST
	/// 170:2004.
	#[strum(serialize = "smpte170m")]
	Smpte170m,

	/// Color space conforming to SMPTE ST 240:1999.
	#[strum(serialize = "smpte240m")]
	Smpte240m,

	/// Color space conforming to ITU-R BT.2020 non-constant luminance system.
	#[strum(serialize = "bt2020")]
	Bt2020,
}

/// YCbCr sample range used by the scale filter's in/out range options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum Range {
	/// Choose automatically.
	#[strum(to_string = "auto", serialize = "unknown")]
	Auto,

	/// Full range (0-255 in case of 8-bit luma).
	#[strum(to_string = "full", serialize = "jpeg", serialize = "pc")]
	Full,

	/// "MPEG" range (16-235 in case of 8-bit luma).
	#[strum(to_string = "limited", serialize = "mpeg", serialize = "tv")]
	Limited,
}

/// Chroma sample location. Defaults to center-sited chroma.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ChromaLocation {
	/// Choose automatically.
	#[strum(to_string = "auto", serialize = "unknown")]
	Auto,

	#[strum(serialize = "left")]
	Left,

	#[strum(serialize = "center")]
	#[default]
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

/// RGB primaries used by the scale filter's in/out primaries options.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum Primaries {
	/// Choose automatically. This is the default.
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	#[strum(serialize = "bt709")]
	Bt709,

	#[strum(serialize = "bt470m")]
	Bt470m,

	#[strum(serialize = "bt470bg")]
	Bt470bg,

	#[strum(serialize = "smpte170m")]
	Smpte170m,

	#[strum(serialize = "smpte240m")]
	Smpte240m,

	#[strum(serialize = "film")]
	Film,

	#[strum(serialize = "bt2020")]
	Bt2020,

	#[strum(serialize = "smpte428")]
	Smpte428,

	#[strum(serialize = "smpte431")]
	Smpte431,

	#[strum(serialize = "smpte432")]
	Smpte432,

	#[strum(serialize = "jedec-p22")]
	JedecP22,

	#[strum(serialize = "ebu3213")]
	Ebu3213,
}

/// Transfer response curve (TRC) used by the scale filter's in/out transfer options.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum Transfer {
	/// Choose automatically. This is the default.
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	#[strum(serialize = "bt709")]
	Bt709,

	#[strum(serialize = "bt470m")]
	Bt470m,

	#[strum(serialize = "gamma22")]
	Gamma22,

	#[strum(serialize = "bt470bg")]
	Bt470bg,

	#[strum(serialize = "gamma28")]
	Gamma28,

	#[strum(serialize = "smpte170m")]
	Smpte170m,

	#[strum(serialize = "smpte240m")]
	Smpte240m,

	#[strum(serialize = "linear")]
	Linear,

	#[strum(serialize = "iec61966-2-1")]
	Iec61966_2_1,

	#[strum(serialize = "srgb")]
	Srgb,

	#[strum(serialize = "iec61966-2-4")]
	Iec61966_2_4,

	#[strum(serialize = "xvycc")]
	Xvycc,

	#[strum(serialize = "bt1361e")]
	Bt1361e,

	#[strum(serialize = "bt2020-10")]
	Bt2020_10,

	#[strum(serialize = "bt2020-12")]
	Bt2020_12,

	#[strum(serialize = "smpte2084")]
	Smpte2084,

	#[strum(serialize = "smpte428")]
	Smpte428,

	#[strum(serialize = "arib-std-b67")]
	AribStdB67,
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
	#[ffarg(name = "w", omit_default)]
	pub width: i32,

	/// The output video height expression. Default value is the input dimension. If the value is
	/// 0, the input height is used for the output. If one and only one of w/h is -n with n >= 1,
	/// the scale filter will use a value that maintains the aspect ratio of the input image,
	/// calculated from the other specified dimension, divisible by n.
	#[ffarg(name = "h", omit_default)]
	pub height: i32,

	/// Set the video scaling algorithm.
	pub scale_algorithm: Algorithm,

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
	pub force_original_aspect_ratio: ForceOriginalAspectRatio,

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

	/// Set the ICC rendering intent to use when transforming between different color spaces.
	/// Defaults to relative colorimetric.
	#[ffarg(omit_default)]
	pub intent: Intent,

	/// Override the input YCbCr color space type. Allows the autodetected value to be overridden
	/// as well as allows forcing a specific value used for the output and encoder. If not
	/// specified, the color space type depends on the pixel format.
	pub in_color_matrix: Option<ColorMatrix>,

	/// Set the output YCbCr color space type. Allows the autodetected value to be overridden as
	/// well as allows forcing a specific value used for the output and encoder. If not specified,
	/// the color space type depends on the pixel format.
	pub out_color_matrix: Option<ColorMatrix>,

	/// Override the input YCbCr sample range. Allows the autodetected value to be overridden as
	/// well as allows forcing a specific value used for the output and encoder. If not specified,
	/// the range depends on the pixel format.
	pub in_range: Option<Range>,

	/// Set the output YCbCr sample range. Allows the autodetected value to be overridden as well
	/// as allows forcing a specific value used for the output and encoder. If not specified, the
	/// range depends on the pixel format.
	pub out_range: Option<Range>,

	/// Override the input chroma sample location. Defaults to center-sited chroma.
	#[ffarg(omit_default)]
	pub in_chroma_loc: ChromaLocation,

	/// Set the output chroma sample location. Defaults to center-sited chroma.
	#[ffarg(omit_default)]
	pub out_chroma_loc: ChromaLocation,

	/// Override the input RGB primaries. Allows the autodetected value to be overridden as well as
	/// allows forcing a specific value used for the output and encoder. Defaults to auto-detect.
	#[ffarg(omit_default)]
	pub in_primaries: Primaries,

	/// Set the output RGB primaries. Allows the autodetected value to be overridden as well as
	/// allows forcing a specific value used for the output and encoder. Defaults to auto-detect.
	#[ffarg(omit_default)]
	pub out_primaries: Primaries,

	/// Override the input transfer response curve (TRC). Allows the autodetected value to be
	/// overridden as well as allows forcing a specific value used for the output and encoder.
	/// Defaults to auto-detect.
	#[ffarg(omit_default)]
	pub in_transfer: Transfer,

	/// Set the output transfer response curve (TRC). Allows the autodetected value to be
	/// overridden as well as allows forcing a specific value used for the output and encoder.
	/// Defaults to auto-detect.
	#[ffarg(omit_default)]
	pub out_transfer: Transfer,
}

impl Scale {
	#[must_use]
	pub fn new(width: i32, height: i32, algorithm: Algorithm) -> Self {
		Self {
			width,
			height,
			scale_algorithm: algorithm,
			force_divisible_by: 2,
			..Default::default()
		}
	}

	#[must_use]
	pub fn preserve_aspect_ratio_width(width: i32, algorithm: Algorithm) -> Self {
		let mut scale = Self::new(width, -2, algorithm);
		scale.force_original_aspect_ratio = ForceOriginalAspectRatio::Decrease;
		scale
	}

	#[must_use]
	pub fn preserve_aspect_ratio_height(height: i32, algorithm: Algorithm) -> Self {
		let mut scale = Self::new(-2, height, algorithm);
		scale.force_original_aspect_ratio = ForceOriginalAspectRatio::Decrease;
		scale
	}

	#[allow(clippy::cast_possible_truncation)]
	pub fn from_size(
		size: String,
		aspect_ratio: ForceOriginalAspectRatio,
		algorithm: Algorithm,
	) -> Result<Self> {
		let parsed_size = parse_ffmpeg_size(size)?;
		Ok(Self {
			width: parsed_size.width as i32,
			height: parsed_size.height as i32,
			scale_algorithm: algorithm,
			force_original_aspect_ratio: aspect_ratio,
			force_divisible_by: 2,
			..Default::default()
		})
	}

	#[must_use]
	pub fn row(width: i32, algorithm: Algorithm) -> Self {
		Self::new(width, 1, algorithm)
	}

	#[must_use]
	pub fn column(height: i32, algorithm: Algorithm) -> Self {
		Self::new(1, height, algorithm)
	}
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
		scale_algorithm: Algorithm::Spline,
		force_divisible_by: 2,
		..Default::default()
	};
	assert_eq!(
		filter.to_string(),
		"scale=w=1920:h=1080:flags=spline+accurate_rnd+full_chroma_int+full_chroma_inp:force_divisible_by=2"
	);
}

#[test]
fn filter_scale_params_inout() {
	let filter = Scale {
		width: 1920,
		height: 1080,
		intent: Intent::Perceptual,
		in_color_matrix: Some(ColorMatrix::Bt470),
		in_range: Some(Range::Full),
		in_chroma_loc: ChromaLocation::Top,
		in_primaries: Primaries::Bt470m,
		in_transfer: Transfer::Bt470m,
		out_color_matrix: Some(ColorMatrix::Bt709),
		out_range: Some(Range::Limited),
		out_chroma_loc: ChromaLocation::Bottom,
		out_primaries: Primaries::Bt709,
		out_transfer: Transfer::Bt709,
		..Default::default()
	};
	assert_eq!(
		filter.to_string(),
		"scale=w=1920:h=1080:flags=bicubic+accurate_rnd+full_chroma_int+full_chroma_inp:intent=perceptual:in_color_matrix=bt470:out_color_matrix=bt709:in_range=full:out_range=limited:in_chroma_loc=top:out_chroma_loc=bottom:in_primaries=bt470m:out_primaries=bt709:in_transfer=bt470m:out_transfer=bt709"
	);
}
