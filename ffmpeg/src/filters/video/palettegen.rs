use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum StatsMode {
	/// Compute full frame histograms.
	#[default]
	#[strum(serialize = "full")]
	Full,

	/// Compute histograms only for the part that differs from previous frame.
	/// This might be relevant to give more importance to the moving part of your input if the background is static.
	#[strum(serialize = "diff")]
	Diff,

	/// Compute new histogram for each frame.
	#[strum(serialize = "single")]
	Single,
}

/// Generate one palette for a whole video stream.
#[filter(name = "palettegen")]
pub struct Palettegen {
	/// Set the maximum number of colors to quantize in the palette.
	/// Note: the palette will still contain 256 colors; the unused palette entries will be black.
	#[ffarg(default = 256)]
	pub max_colors: u16,

	/// Create a palette of 255 colors maximum and reserve the last one for transparency.
	/// Reserving the transparency color is useful for GIF optimization.
	/// If not set, the maximum of colors in the palette will be 256.
	/// You probably want to disable this option for a standalone image. Set by default.
	#[ffarg(default = true, omit_default)]
	pub reserve_transparent: bool,

	/// Set the color that will be used as background for transparency.
	#[ffarg(default = "lime", omit_default)]
	pub transparency_color: String,

	/// Set statistics mode.
	#[ffarg(default = StatsMode::Full, omit_default)]
	pub stats_mode: StatsMode,
}

#[test]
fn filter_palettegen() {
	let filter = Palettegen::default();
	assert_eq!(filter.to_string(), "palettegen=max_colors=256");
}

#[test]
fn filter_palettegen_params() {
	let filter = Palettegen {
		max_colors: 64,
		reserve_transparent: false,
		stats_mode: StatsMode::Single,
		..Default::default()
	};
	assert_eq!(
		filter.to_string(),
		"palettegen=max_colors=64:reserve_transparent=0:stats_mode=single"
	);
}
