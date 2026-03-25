use ffmpeg_macro::filter;

#[derive(Debug, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ColorspaceAll {
	/// BT.470M
	#[strum(serialize = "bt470m")]
	Bt470m,

	/// BT.470BG
	#[strum(serialize = "bt470bg")]
	Bt470bg,

	/// BT.601-6 525
	#[strum(serialize = "bt601-6-525")]
	Bt601_6_525,

	/// BT.601-6 625
	#[strum(serialize = "bt601-6-625")]
	Bt601_6_625,

	/// BT.709
	#[strum(serialize = "bt709")]
	Bt709,

	/// SMPTE-170M
	#[strum(serialize = "smpte170m")]
	Smpte170m,

	/// SMPTE-240M
	#[strum(serialize = "smpte240m")]
	Smpte240m,

	/// BT.2020
	#[strum(serialize = "bt2020")]
	Bt2020,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ColorspaceSpace {
	/// BT.709
	#[strum(serialize = "bt709")]
	Bt709,

	/// FCC
	#[strum(serialize = "fcc")]
	Fcc,

	/// BT.470BG or BT.601-6 625
	#[strum(serialize = "bt470bg")]
	Bt470bg,

	/// SMPTE-170M or BT.601-6 525
	#[strum(serialize = "smpte170m")]
	Smpte170m,

	/// SMPTE-240M
	#[strum(serialize = "smpte240m")]
	Smpte240m,

	/// YCgCo
	#[strum(serialize = "ycgco")]
	Ycgco,

	/// BT.2020 with non-constant luminance
	#[strum(serialize = "bt2020ncl")]
	Bt2020Ncl,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ColorspaceTransfer {
	/// BT.709
	#[strum(serialize = "bt709")]
	Bt709,

	/// BT.470M
	#[strum(serialize = "bt470m")]
	Bt470m,

	/// BT.470BG
	#[strum(serialize = "bt470bg")]
	Bt470bg,

	/// Constant gamma of 2.2
	#[strum(serialize = "gamma22")]
	Gamma22,

	/// Constant gamma of 2.8
	#[strum(serialize = "gamma28")]
	Gamma28,

	/// SMPTE-170M, BT.601-6 625 or BT.601-6 525
	#[strum(serialize = "smpte170m")]
	Smpte170m,

	/// SMPTE-240M
	#[strum(serialize = "smpte240m")]
	Smpte240m,

	/// SRGB
	#[strum(serialize = "srgb")]
	Srgb,

	/// iec61966-2-1
	#[strum(serialize = "iec61966-2-1")]
	Iec61966_2_1,

	/// iec61966-2-4
	#[strum(serialize = "iec61966-2-4")]
	Iec61966_2_4,

	/// xvycc
	#[strum(serialize = "xvycc")]
	Xvycc,

	/// BT.2020 for 10-bits content
	#[strum(serialize = "bt2020-10")]
	Bt2020_10,

	/// BT.2020 for 12-bits content
	#[strum(serialize = "bt2020-12")]
	Bt2020_12,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ColorspacePrimaries {
	/// BT.709
	#[strum(serialize = "bt709")]
	Bt709,

	/// BT.470M
	#[strum(serialize = "bt470m")]
	Bt470m,

	/// BT.470BG or BT.601-6 625
	#[strum(serialize = "bt470bg")]
	Bt470bg,

	/// SMPTE-170M or BT.601-6 525
	#[strum(serialize = "smpte170m")]
	Smpte170m,

	/// SMPTE-240M
	#[strum(serialize = "smpte240m")]
	Smpte240m,

	/// film
	#[strum(serialize = "film")]
	Film,

	/// SMPTE-431
	#[strum(serialize = "smpte431")]
	Smpte431,

	/// SMPTE-432
	#[strum(serialize = "smpte432")]
	Smpte432,

	/// BT.2020
	#[strum(serialize = "bt2020")]
	Bt2020,

	/// JEDEC P22 phosphors
	#[strum(serialize = "jedec-p22")]
	JedecP22,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ColorspaceRange {
	/// TV (restricted) range
	#[strum(serialize = "tv")]
	Tv,

	/// MPEG (restricted) range
	#[strum(serialize = "mpeg")]
	Mpeg,

	/// PC (full) range
	#[strum(serialize = "pc")]
	Pc,

	/// JPEG (full) range
	#[strum(serialize = "jpeg")]
	Jpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ColorspaceFormat {
	/// YUV 4:2:0 planar 8-bits
	#[strum(serialize = "yuv420p")]
	Yuv420p,

	/// YUV 4:2:0 planar 10-bits
	#[strum(serialize = "yuv420p10")]
	Yuv420p10,

	/// YUV 4:2:0 planar 12-bits
	#[strum(serialize = "yuv420p12")]
	Yuv420p12,

	/// YUV 4:2:2 planar 8-bits
	#[strum(serialize = "yuv422p")]
	Yuv422p,

	/// YUV 4:2:2 planar 10-bits
	#[strum(serialize = "yuv422p10")]
	Yuv422p10,

	/// YUV 4:2:2 planar 12-bits
	#[strum(serialize = "yuv422p12")]
	Yuv422p12,

	/// YUV 4:4:4 planar 8-bits
	#[strum(serialize = "yuv444p")]
	Yuv444p,

	/// YUV 4:4:4 planar 10-bits
	#[strum(serialize = "yuv444p10")]
	Yuv444p10,

	/// YUV 4:4:4 planar 12-bits
	#[strum(serialize = "yuv444p12")]
	Yuv444p12,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ColorspaceDither {
	/// No dithering
	#[strum(serialize = "none")]
	None,

	/// Floyd-Steinberg dithering
	#[strum(serialize = "fsb")]
	FloydSteinberg,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ColorspaceWhitepointAdapt {
	/// Bradford whitepoint adaptation
	#[strum(serialize = "bradford")]
	Bradford,

	/// von Kries whitepoint adaptation
	#[strum(serialize = "vonkries")]
	VonKries,

	/// identity whitepoint adaptation (i.e. no whitepoint adaptation)
	#[strum(serialize = "identity")]
	Identity,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum ColorspaceClipGamut {
	/// No clipping of out of gamut colors.
	#[strum(serialize = "none")]
	None,

	/// Clips the RGB values to the [0, 1] range when building the gamma transfer LUTs.
	#[strum(serialize = "rgb")]
	Rgb,
}

/// Convert colorspace, transfer characteristics or color primaries. Input video needs to have an even size.
#[filter(name = "colorspace")]
pub struct Colorspace {
	/// Specify all color properties at once.
	pub all: Option<ColorspaceAll>,

	/// Specify output colorspace.
	pub space: Option<ColorspaceSpace>,

	/// Specify output transfer characteristics.
	#[ffarg(name = "trc")]
	pub transfer: Option<ColorspaceTransfer>,

	/// Specify output color primaries.
	pub primaries: Option<ColorspacePrimaries>,

	/// Specify output color range.
	pub range: Option<ColorspaceRange>,

	/// Specify output color format.
	pub format: Option<ColorspaceFormat>,

	/// Do a fast conversion, which skips gamma/primary correction. This will take significantly
	/// less CPU, but will be mathematically incorrect. To get output compatible with that produced
	/// by the colormatrix filter, use fast=1.
	#[ffarg(omit_default)]
	pub fast: bool,

	/// Specify dithering mode.
	pub dither: Option<ColorspaceDither>,

	/// Whitepoint adaptation mode.
	#[ffarg(name = "wpadapt")]
	pub whitepoint_adapt: Option<ColorspaceWhitepointAdapt>,

	/// Controls how to clip out-of-gamut colors that arise as a result of colorspace conversion.
	#[ffarg(name = "clip_gamit")]
	pub clip_gamut: Option<ColorspaceClipGamut>,

	/// Override all input properties at once. Same accepted values as all.
	#[ffarg(name = "iall")]
	pub input_all: Option<ColorspaceAll>,

	/// Override input colorspace. Same accepted values as space.
	#[ffarg(name = "ispace")]
	pub input_space: Option<ColorspaceSpace>,

	/// Override input color primaries. Same accepted values as primaries.
	#[ffarg(name = "iprimaries")]
	pub input_primaries: Option<ColorspacePrimaries>,

	/// Override input transfer characteristics. Same accepted values as trc.
	#[ffarg(name = "itrc")]
	pub input_transfer: Option<ColorspaceTransfer>,

	/// Override input color range. Same accepted values as range.
	#[ffarg(name = "irange")]
	pub input_range: Option<ColorspaceRange>,
}

#[test]
fn filter_colorspace() {
	let filter = Colorspace::default();
	assert_eq!(filter.to_string(), "colorspace");
}

#[test]
fn filter_colorspace_params() {
	let filter = Colorspace {
		all: Some(ColorspaceAll::Bt709),
		transfer: Some(ColorspaceTransfer::Srgb),
		range: Some(ColorspaceRange::Pc),
		dither: Some(ColorspaceDither::FloydSteinberg),
		..Default::default()
	};
	assert_eq!(filter.to_string(), "colorspace=all=bt709:trc=srgb:range=pc:dither=fsb");
}