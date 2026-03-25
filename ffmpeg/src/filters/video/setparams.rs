use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum SetParamsFieldMode {
	/// Keep the same field property (default).
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	/// Mark the frame as bottom-field-first.
	#[strum(serialize = "bff")]
	BottomFieldFirst,

	/// Mark the frame as top-field-first.
	#[strum(serialize = "tff")]
	TopFieldFirst,

	/// Mark the frame as progressive.
	#[strum(serialize = "prog")]
	Progressive,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum SetParamsRange {
	/// Keep the same color range property (default).
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	/// Mark the frame as unspecified color range.
	#[strum(serialize = "unspecified")]
	Unspecified,

	/// Mark the frame as limited range.
	#[strum(serialize = "limited")]
	Limited,

	/// Mark the frame as full range.
	#[strum(serialize = "full")]
	Full,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum SetParamsColorPrimaries {
	/// Keep the same color primaries property (default).
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	#[strum(serialize = "bt709")]
	Bt709,

	#[strum(serialize = "unknown")]
	Unknown,

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
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum SetParamsColorTransfer {
	/// Keep the same color trc property (default).
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	#[strum(serialize = "bt709")]
	Bt709,

	#[strum(serialize = "unknown")]
	Unknown,

	#[strum(serialize = "bt470m")]
	Bt470m,

	#[strum(serialize = "bt470bg")]
	Bt470bg,

	#[strum(serialize = "smpte170m")]
	Smpte170m,

	#[strum(serialize = "smpte240m")]
	Smpte240m,

	#[strum(serialize = "linear")]
	Linear,

	#[strum(serialize = "log100")]
	Log100,

	#[strum(serialize = "log316")]
	Log316,

	#[strum(serialize = "iec61966-2-4")]
	Iec61966_2_4,

	#[strum(serialize = "bt1361e")]
	Bt1361e,

	#[strum(serialize = "iec61966-2-1")]
	Iec61966_2_1,

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

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum SetParamsColorspace {
	/// Keep the same colorspace property (default).
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	#[strum(serialize = "gbr")]
	Gbr,

	#[strum(serialize = "bt709")]
	Bt709,

	#[strum(serialize = "unknown")]
	Unknown,

	#[strum(serialize = "fcc")]
	Fcc,

	#[strum(serialize = "bt470bg")]
	Bt470bg,

	#[strum(serialize = "smpte170m")]
	Smpte170m,

	#[strum(serialize = "smpte240m")]
	Smpte240m,

	#[strum(serialize = "ycgco")]
	Ycgco,

	#[strum(serialize = "bt2020nc")]
	Bt2020nc,

	#[strum(serialize = "bt2020c")]
	Bt2020c,

	#[strum(serialize = "smpte2085")]
	Smpte2085,

	#[strum(serialize = "chroma-derived-nc")]
	ChromaDerivedNc,

	#[strum(serialize = "chroma-derived-c")]
	ChromaDerivedC,

	#[strum(serialize = "ictcp")]
	Ictcp,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum SetParamsChromaLocation {
	/// Keep the same chroma location (default).
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	#[strum(serialize = "unspecified")]
	Unspecified,

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

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum SetParamsAlphaMode {
	/// Keep the same alpha mode (default).
	#[strum(serialize = "auto")]
	#[default]
	Auto,

	#[strum(serialize = "unspecified")]
	Unspecified,

	#[strum(serialize = "premultiplied")]
	Premultiplied,

	#[strum(serialize = "straight")]
	Straight,
}

/// Force frame parameters for the output video frame.
///
/// The setparams filter marks interlace and color range for the output frames. It does not change
/// the input frame, but only sets the corresponding property, which affects how the frame is
/// treated by filters/encoders.
#[filter(name = "setparams")]
pub struct SetParams {
	#[ffarg(omit_default)]
	pub field_mode: SetParamsFieldMode,

	#[ffarg(omit_default)]
	pub range: SetParamsRange,

	#[ffarg(omit_default)]
	pub color_primaries: SetParamsColorPrimaries,

	#[ffarg(omit_default)]
	pub color_trc: SetParamsColorTransfer,

	#[ffarg(omit_default)]
	pub colorspace: SetParamsColorspace,

	#[ffarg(omit_default)]
	pub chroma_location: SetParamsChromaLocation,

	#[ffarg(omit_default)]
	pub alpha_mode: SetParamsAlphaMode,
}

#[test]
fn filter_setparams() {
	let filter = SetParams::default();
	assert_eq!(filter.to_string(), "setparams");
}

#[test]
fn filter_setparams_params() {
	let filter = SetParams {
		colorspace: SetParamsColorspace::Bt709,
		color_primaries: SetParamsColorPrimaries::Bt709,
		color_trc: SetParamsColorTransfer::Iec61966_2_1,
		..Default::default()
	};
	assert_eq!(
		filter.to_string(),
		"setparams=color_primaries=bt709:color_trc=iec61966-2-1:colorspace=bt709"
	);
}
