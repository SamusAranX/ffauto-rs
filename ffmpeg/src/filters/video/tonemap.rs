use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum TonemapAlgorithm {
	/// Do not apply any tone map, only desaturate overbright pixels.
	#[strum(serialize = "none")]
	#[default]
	None,

	/// Hard-clip any out-of-range values. Use it for perfect color accuracy for in-range values,
	/// while distorting out-of-range values.
	#[strum(serialize = "clip")]
	Clip,

	/// Stretch the entire reference gamut to a linear multiple of the display.
	#[strum(serialize = "linear")]
	Linear,

	/// Fit a logarithmic transfer between the tone curves.
	#[strum(serialize = "gamma")]
	Gamma,

	/// Preserve overall image brightness with a simple curve, using nonlinear contrast, which
	/// results in flattening details and degrading color accuracy.
	#[strum(serialize = "reinhard")]
	Reinhard,

	/// Preserve both dark and bright details better than reinhard, at the cost of slightly
	/// darkening everything. Use it when detail preservation is more important than color and
	/// brightness accuracy.
	#[strum(serialize = "hable")]
	Hable,

	/// Smoothly map out-of-range values, while retaining contrast and colors for in-range material
	/// as much as possible. Use it when color accuracy is more important than detail preservation.
	#[strum(serialize = "mobius")]
	Mobius,
}

/// Tone map colors from different dynamic ranges.
///
/// This filter expects data in single precision floating point, as it needs to operate on (and can
/// output) out-of-range values. Another filter, such as zscale, is needed to convert the resulting
/// frame to a usable format.
///
/// The tonemapping algorithms implemented only work on linear light, so input data should be
/// linearized beforehand (and possibly correctly tagged).
#[filter(name = "tonemap")]
pub struct Tonemap {
	/// Set the tone map algorithm to use.
	#[ffarg(name = "tonemap", omit_default)]
	pub algorithm: TonemapAlgorithm,

	/// Apply desaturation for highlights that exceed this level of brightness. The higher the
	/// parameter, the more color information will be preserved. This helps prevent unnaturally
	/// blown-out colors for super-highlights by smoothly turning them white instead. A setting of
	/// 0.0 disables this option. This option works only if the input frame has a supported color
	/// tag.
	#[ffarg(default = 2.0, omit_default)]
	pub desat: f64,

	/// Override signal/nominal/reference peak with this value. Useful when the embedded peak
	/// information in display metadata is not reliable or when tone mapping from a lower range to
	/// a higher range.
	#[ffarg(omit_default)]
	pub peak: f64,
}