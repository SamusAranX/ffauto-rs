use ffmpeg_macro::filter;

/// Draw subtitles on top of input video using the libass library.
#[filter(name = "subtitles")]
pub struct Subtitles {
	/// Set the filename of the subtitle file to read. It must be specified.
	#[ffarg()]
	pub filename: String,

	/// Specify the size of the original video, the video for which the ASS file was composed.
	/// Due to a misdesign in ASS aspect ratio arithmetic, this is necessary to correctly scale the
	/// fonts if the aspect ratio has been changed.
	#[ffarg()]
	pub original_size: String,

	/// Set a directory path containing fonts that can be used by the filter. These fonts will be
	/// used in addition to whatever the font provider uses.
	#[ffarg(omit_default)]
	pub fontsdir: String,

	/// Process alpha channel. By default the alpha channel is untouched.
	#[ffarg(omit_default)]
	pub alpha: bool,

	/// Set subtitles input character encoding. Only useful if not UTF-8.
	#[ffarg(omit_default)]
	pub charenc: String,

	/// Set subtitles stream index.
	#[ffarg(name = "stream_index")]
	pub stream_index: i64,

	/// Override default style or script info parameters of the subtitles. Accepts a string
	/// containing ASS style format KEY=VALUE couples separated by ",".
	#[ffarg(omit_default)]
	pub force_style: String,

	/// Break lines according to the Unicode Line Breaking Algorithm. Enabled by default except
	/// for native ASS.
	#[ffarg(default = true)]
	pub wrap_unicode: bool,
}

#[test]
fn filter_subtitles() {
	let filter = Subtitles {
		filename: "/tmp/example.srt".to_string(),
		original_size: "1920x1080".to_string(),
		fontsdir: "/tmp/fonts/".to_string(),
		stream_index: 0,
		..Default::default()
	};
	assert_eq!(
		filter.to_string(),
		"subtitles=filename=/tmp/example.srt:original_size=1920x1080:fontsdir=/tmp/fonts/:stream_index=0:wrap_unicode=1"
	);
}
