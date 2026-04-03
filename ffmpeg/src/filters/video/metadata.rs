use ffmpeg_macro::filter;

#[derive(Debug, Default, Clone, Copy, PartialEq, strum::Display, strum::EnumString)]
pub enum Mode {
	/// If both value and key is set, select frames which have such metadata. If only key is set,
	/// select every frame that has such key in metadata.
	#[strum(serialize = "select")]
	#[default]
	Select,

	/// Add new metadata key and value. If key is already available do nothing.
	#[strum(serialize = "add")]
	Add,

	/// Modify value of already present key.
	#[strum(serialize = "modify")]
	Modify,

	/// If value is set, delete only keys that have such value. Otherwise, delete key. If key is
	/// not set, delete all metadata values in the frame.
	#[strum(serialize = "delete")]
	Delete,

	/// Print key and its value if metadata was found. If key is not set print all metadata values
	/// available in frame.
	#[strum(serialize = "print")]
	Print,
}

/// Manipulate frame metadata.
#[filter(name = "metadata")]
pub struct Metadata {
	/// Set mode of operation of the filter.
	pub mode: Mode,

	/// If specified in print mode, output is written to the named file. Instead of plain filename
	/// any writable url can be specified. Filename "-" is a shorthand for standard output. If
	/// file option is not set, output is written to the log with AV_LOG_INFO loglevel.
	pub file: Option<String>,

	/// Reduces buffering in print mode when output is written to a URL set using file.
	#[ffarg(omit_default)]
	pub direct: bool,
}

impl Metadata {
	#[must_use]
	pub fn new(mode: Mode, file: Option<String>) -> Self {
		Self { mode, file, ..Default::default() }
	}
}
