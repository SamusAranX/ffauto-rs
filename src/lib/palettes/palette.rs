use std::fmt::{Display, Formatter};
use std::path::Path;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

impl From<[u8; 3]> for Color {
	fn from(v: [u8; 3]) -> Self {
		Self {
			r: v[0],
			g: v[1],
			b: v[2],
		}
	}
}

impl From<u32> for Color {
	fn from(v: u32) -> Self {
		Self {
			r: ((v >> 16) & 0xFF) as u8,
			g: ((v >> 8) & 0xFF) as u8,
			b: (v & 0xFF) as u8,
		}
	}
}

fn scale_6bits_to_8bits(v: u8) -> u8 {
	let v = v & 0b111111;
	(v << 2) | (v >> 4)
}

impl Color {
	pub(crate) fn from_6bits(v: [u8; 3]) -> Self {
		Self {
			r: scale_6bits_to_8bits(v[0]),
			g: scale_6bits_to_8bits(v[1]),
			b: scale_6bits_to_8bits(v[2]),
		}
	}
}

impl Display for Color {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut rgb = self.r as u32;
		rgb = (rgb << 8) | self.g as u32;
		rgb = (rgb << 8) | self.b as u32;
		write!(f, "#{:06X}", rgb)
	}
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Entry {
	pub color: Color,
	pub name: String,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Palette {
	pub colors: Vec<Entry>,
}

impl Palette {
	pub fn push_color(&mut self, c: Color) {
		self.colors.push(Entry { color: c, ..Default::default() });
	}

	pub fn push_named_color(&mut self, c: Color, name: String) {
		self.colors.push(Entry { color: c, name });
	}

	pub fn len(&self) -> usize {
		self.colors.len()
	}

	pub fn is_empty(&self) -> bool {
		self.colors.is_empty()
	}

	fn guess_format<P: AsRef<Path>>(path: P) -> Option<PaletteFormat> {
		let p = path.as_ref();
		let ext = p.extension()?
			.to_str().unwrap()
			.to_lowercase();

		match ext.as_str() {
			"act" => Some(PaletteFormat::AdobeAct),
			"col" => Some(PaletteFormat::AnimatorProCol),
			"gpl" => Some(PaletteFormat::Gpl),
			"hex" => Some(PaletteFormat::Hex),
			"json" => Some(PaletteFormat::Json),
			"pal" => Some(PaletteFormat::Pal),
			_ => None
		}
	}

	pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Palette, PaletteError> {
		match Self::guess_format(&path) {
			Some(PaletteFormat::AdobeAct) => Self::from_act_file(&path),
			Some(PaletteFormat::AnimatorProCol) => Self::from_col_file(&path),
			Some(PaletteFormat::Gpl) => Self::from_gpl_file(&path),
			Some(PaletteFormat::Hex) => Self::from_hex_file(&path),
			Some(PaletteFormat::Json) => Self::from_json_file(&path),
			Some(PaletteFormat::Pal) => Self::from_pal_file(&path),
			_ => Err(PaletteError::InvalidFile),
		}
	}

	pub fn load_from_string<S: Into<String>>(s: S, format: PaletteFormat) -> Result<Palette, PaletteError> {
		match format {
			PaletteFormat::Gpl => Self::from_gpl_string(s.into()),
			PaletteFormat::Hex => Self::from_hex_string(s.into()),
			PaletteFormat::Json => Self::from_json_string(s.into()),
			PaletteFormat::Pal => Self::from_pal_string(s.into()),
			_ => Err(PaletteError::UnsupportedFormat),
		}
	}
}

impl From<Vec<u32>> for Palette {
	fn from(v: Vec<u32>) -> Self {
		let mut pal = Palette::default();
		for c in v {
			pal.push_color(Color::from(c));
		}
		pal
	}
}

impl From<Vec<Color>> for Palette {
	fn from(v: Vec<Color>) -> Self {
		let mut pal = Palette::default();
		for c in v {
			pal.push_color(c);
		}
		pal
	}
}

#[derive(Debug)]
pub enum PaletteError {
	Empty,
	TooManyColors,
	UnsupportedFormat,
	InvalidFile,
	InvalidBinaryData { position: usize, msg: String },
	InvalidTextLine { line: usize, msg: String },
	InvalidJsonEntry { index: usize, msg: String },
	IoErr(std::io::Error),
}

impl Display for PaletteError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			PaletteError::Empty => write!(f, "The loaded palette is empty"),
			PaletteError::TooManyColors => write!(f, "The palette file contains more than 256 colors"),
			PaletteError::UnsupportedFormat => write!(f, "Tried reading a binary format as text or vice versa, which is not supported"),
			PaletteError::InvalidFile => write!(f, "Invalid file"),
			PaletteError::InvalidBinaryData { position, msg } => write!(f, "Invalid data at byte {position:#X}: {msg}"),
			PaletteError::InvalidTextLine { line, msg } => write!(f, "Invalid data in line {line}: {msg}"),
			PaletteError::InvalidJsonEntry { index, msg } => write!(f, "Invalid JSON array item at index {index}: {msg}"),
			PaletteError::IoErr(e) => write!(f, "io error: {e}"),
		}
	}
}

impl From<std::io::Error> for PaletteError {
	fn from(e: std::io::Error) -> Self {
		PaletteError::IoErr(e)
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaletteFormat {
	AdobeAct, // .act
	AnimatorProCol, // .col
	Gpl, // .gpl
	Hex, // .hex
	Json, // .json
	Pal, // .pal
}

impl Display for PaletteFormat {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			PaletteFormat::AdobeAct => write!(f, "act"),
			PaletteFormat::AnimatorProCol => write!(f, "col"),
			PaletteFormat::Gpl => write!(f, "gpl"),
			PaletteFormat::Hex => write!(f, "hex"),
			PaletteFormat::Json => write!(f, "json"),
			PaletteFormat::Pal => write!(f, "pal"),
		}
	}
}