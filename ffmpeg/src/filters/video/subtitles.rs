use ffmpeg_macro::filter;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Draw subtitles on top of input video using the libass library.
#[filter(name = "subtitles")]
pub struct Subtitles {
	/// Set the filename of the subtitle file to read. It must be specified.
	/// Specify the name of a video file to be able to use `stream_index`.
	pub filename: PathBuf,

	/// Specify the size of the original video, the video for which the ASS file was composed.
	/// Due to a misdesign in ASS aspect ratio arithmetic, this is necessary to correctly scale the
	/// fonts if the aspect ratio has been changed.
	#[ffarg(omit_default)]
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
	pub stream_index: Option<usize>,

	/// Override default style or script info parameters of the subtitles. Accepts a string
	/// containing ASS style format KEY=VALUE couples separated by ",".
	#[ffarg(omit_default)]
	pub force_style: HashMap<String, String>,

	/// Break lines according to the Unicode Line Breaking Algorithm. Enabled by default except
	/// for native ASS.
	pub wrap_unicode: Option<bool>,
}

impl Subtitles {
	pub fn new_with_file<P: AsRef<Path>>(file: P) -> Self {
		Self {
			filename: file.as_ref().into(),
			..Default::default()
		}
	}

	pub fn new_with_file_and_stream_index<P: AsRef<Path>>(file: P, index: usize) -> Self {
		Self {
			filename: file.as_ref().into(),
			stream_index: Some(index),
			..Default::default()
		}
	}

	pub fn set_font<S: Into<String>>(&mut self, font_name: S) {
		self.force_style.insert("Fontname".into(), font_name.into());
	}
}

#[test]
fn filter_subtitles_filename() {
	let filter = Subtitles::new_with_file("/tmp/example.srt");

	assert_eq!(filter.to_string(), "subtitles=filename='/tmp/example.srt'");
}

#[test]
fn filter_subtitles_stream_index() {
	let filter = Subtitles::new_with_file_and_stream_index("/tmp/input.mkv", 1);

	assert_eq!(
		filter.to_string(),
		"subtitles=filename='/tmp/input.mkv':stream_index=1"
	);
}

#[test]
fn filter_subtitles_filename_force_style() {
	let mut filter = Subtitles::new_with_file("/tmp/example.srt");
	filter
		.force_style
		.insert("Fontname".into(), "DejaVu Serif".into());
	filter
		.force_style
		.insert("PrimaryColour".into(), "&HCCFF0000".into());

	assert_eq!(
		filter.to_string(),
		"subtitles=filename='/tmp/example.srt':force_style='Fontname=DejaVu Serif,PrimaryColour=&HCCFF0000'"
	);
}
