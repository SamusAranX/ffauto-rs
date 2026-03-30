use ffmpeg_macro::filter;
use std::collections::HashMap;

/// Draw subtitles on top of input video using the libass library.
#[filter(name = "subtitles")]
pub struct Subtitles {
	/// Set the filename of the subtitle file to read. It must be specified.
	pub filename: String,

	/// Specify the size of the original video, the video for which the ASS file was composed.
	/// Due to a misdesign in ASS aspect ratio arithmetic, this is necessary to correctly scale the
	/// fonts if the aspect ratio has been changed.
	pub original_size: String,

	/// Set a directory path containing fonts that can be used by the filter. These fonts will be
	/// used in addition to whatever the font provider uses.
	#[ffarg(name = "fontsdir", omit_default)]
	pub fonts_dir: String,

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
	pub force_style: HashMap<String, String>,

	/// Break lines according to the Unicode Line Breaking Algorithm. Enabled by default except
	/// for native ASS.
	#[ffarg(default = true)]
	pub wrap_unicode: bool,
}

impl Subtitles {
	pub fn new<N: Into<String>, S: Into<String>, F: Into<String>>(
		filename: N,
		original_size: S,
		fonts_dir: F,
	) -> Self {
		Self {
			filename: filename.into(),
			original_size: original_size.into(),
			fonts_dir: fonts_dir.into(),
			..Default::default()
		}
	}
}

#[test]
fn filter_subtitles() {
	let filter = Subtitles::new("/tmp/example.srt", "1920x1080", "/tmp/fonts/");

	assert_eq!(
		filter.to_string(),
		"subtitles=filename=/tmp/example.srt:original_size=1920x1080:fontsdir=/tmp/fonts/:stream_index=0:wrap_unicode=1"
	);
}

#[test]
fn filter_subtitles_force_style() {
	let mut filter = Subtitles::new("/tmp/example.srt", "1920x1080", "/tmp/fonts/");
	filter
		.force_style
		.insert("Fontname".into(), "DejaVu Serif".into());
	filter
		.force_style
		.insert("PrimaryColour".into(), "&HCCFF0000".into());

	assert_eq!(
		filter.to_string(),
		"subtitles=filename=/tmp/example.srt:original_size=1920x1080:fontsdir=/tmp/fonts/:stream_index=0:force_style='Fontname=DejaVu Serif,PrimaryColour=&HCCFF0000':wrap_unicode=1"
	);
}
