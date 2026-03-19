use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScaleEval {
	/// Only evaluate expressions once during the filter initialization or when a command is
	/// processed.
	#[strum(serialize = "init")]
	#[default]
	Init,

	/// Evaluate expressions for each incoming frame.
	#[strum(serialize = "frame")]
	Frame,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScaleInterlacing {
	/// Force interlaced aware scaling.
	#[strum(serialize = "1")]
	Interlaced,

	/// Do not apply interlaced scaling.
	#[strum(serialize = "0")]
	#[default]
	None,

	/// Select interlaced aware scaling depending on whether the source frames are flagged as
	/// interlaced or not.
	#[strum(serialize = "-1")]
	Auto,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScaleIntent {
	/// Use a perceptually guided tone and gamut mapping curve. The exact details of the mapping
	/// used may change at any time and should not be relied on as stable. This intent is
	/// recommended for final viewing of image/video content in typical viewing settings.
	#[strum(serialize = "perceptual")]
	Perceptual,

	/// Statically clip out-of-gamut colors using a colorimetric clipping curve which attempts to
	/// find the colorimetrically least dissimilar in-gamut color. This intent performs white point
	/// adaptation and black point adaptation. This intent is recommended wherever faithful color
	/// reproduction is of the utmost importance, even at the cost of clipping.
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
	/// lead to any clipping.
	#[strum(serialize = "saturation")]
	Saturation,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScaleColorMatrix {
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	#[strum(serialize = "bt709")]
	Bt709,

	#[strum(serialize = "fcc")]
	Fcc,

	/// Conforms to ITU-R Rec. BT.601, ITU-R Rec. BT.470-6 (1998) Systems B, B1, and G, and
	/// SMPTE ST 170:2004.
	#[strum(serialize = "bt601")]
	Bt601,

	#[strum(serialize = "smpte240m")]
	Smpte240m,

	#[strum(serialize = "bt2020")]
	Bt2020,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScaleRange {
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	/// Full range (0-255 in case of 8-bit luma).
	#[strum(serialize = "jpeg")]
	Full,

	/// "MPEG" range (16-235 in case of 8-bit luma).
	#[strum(serialize = "mpeg")]
	Limited,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScaleChromaLoc {
	#[strum(serialize = "auto")]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScalePrimaries {
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScaleTransfer {
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
	Iec6196621,

	#[strum(serialize = "srgb")]
	Srgb,

	#[strum(serialize = "iec61966-2-4")]
	Iec6196624,

	#[strum(serialize = "xvycc")]
	Xvycc,

	#[strum(serialize = "bt1361e")]
	Bt1361e,

	#[strum(serialize = "bt2020-10")]
	Bt202010,

	#[strum(serialize = "bt2020-12")]
	Bt202012,

	#[strum(serialize = "smpte2084")]
	Smpte2084,

	#[strum(serialize = "smpte428")]
	Smpte428,

	#[strum(serialize = "arib-std-b67")]
	AribStdB67,
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
	pub width: f64,

	/// The output video height expression. Default value is the input dimension. If the value is
	/// 0, the input height is used for the output. If one and only one of w/h is -n with n >= 1,
	/// the scale filter will use a value that maintains the aspect ratio of the input image,
	/// calculated from the other specified dimension, divisible by n.
	#[ffarg(name = "h")]
	pub height: f64,

	/// Specify when to evaluate width and height expressions.
	#[ffarg()]
	pub eval: ScaleEval,

	/// Set the interlacing mode.
	#[ffarg()]
	pub interl: ScaleInterlacing,

	/// Set libswscale scaling flags. If not explicitly specified the filter applies the default flags.
	#[ffarg(separator = "+", extra_flags = ["accurate_rnd", "full_chroma_int", "full_chroma_inp"])]
	pub flags: Vec<String>,

	/// Set libswscale input parameters for scaling algorithms that need them. If not explicitly
	/// specified the filter applies empty parameters.
	#[ffarg()]
	pub param0: String,

	/// Set libswscale input parameters for scaling algorithms that need them. If not explicitly
	/// specified the filter applies empty parameters.
	#[ffarg()]
	pub param1: String,

	/// Set the ICC rendering intent to use when transforming between different color spaces.
	#[ffarg()]
	pub intent: ScaleIntent,

	/// Set the video size (width and height together).
	// #[ffarg(name = "s")]
	// pub size: String,

	/// Set input YCbCr color space type.
	#[ffarg()]
	pub in_color_matrix: ScaleColorMatrix,

	/// Set output YCbCr color space type.
	#[ffarg()]
	pub out_color_matrix: ScaleColorMatrix,

	/// Set input YCbCr sample range.
	#[ffarg()]
	pub in_range: ScaleRange,

	/// Set output YCbCr sample range.
	#[ffarg()]
	pub out_range: ScaleRange,

	/// Set input chroma sample location.
	#[ffarg()]
	pub in_chroma_loc: ScaleChromaLoc,

	/// Set output chroma sample location.
	#[ffarg()]
	pub out_chroma_loc: ScaleChromaLoc,

	/// Set input RGB primaries.
	#[ffarg()]
	pub in_primaries: ScalePrimaries,

	/// Set output RGB primaries.
	#[ffarg()]
	pub out_primaries: ScalePrimaries,

	/// Set input transfer response curve (TRC).
	#[ffarg()]
	pub in_transfer: ScaleTransfer,

	/// Set output transfer response curve (TRC).
	#[ffarg()]
	pub out_transfer: ScaleTransfer,

	/// Enable decreasing or increasing output video width or height if necessary to keep the
	/// original aspect ratio.
	#[ffarg()]
	pub force_original_aspect_ratio: ScaleForceOriginalAspectRatio,

	/// Ensures that both the output dimensions, width and height, are divisible by the given integer
	/// when used together with force_original_aspect_ratio. This works similar to using -n in the w and h options.
	// This option respects the value set for force_original_aspect_ratio, increasing or decreasing the resolution accordingly.
	// The video’s aspect ratio may be slightly modified.
	// This option can be handy if you need to have a video fit within or exceed a defined resolution using
	// force_original_aspect_ratio but also have encoder restrictions on width or height divisibility.
	#[ffarg(default = 1)]
	pub force_divisible_by: u64,

	/// When enabled, the output SAR is reset to 1. Additionally, if proportional scaling is
	/// requested, the input DAR is taken into account and the output is scaled to produce square
	/// pixels.
	#[ffarg()]
	pub reset_sar: bool,
}