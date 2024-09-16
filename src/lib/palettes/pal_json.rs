use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::palettes::palette::{Color, Palette, PaletteError};
use crate::palettes::MAX_PALETTE_COLORS;

impl Palette {
	fn from_json_internal<R: Read + BufRead>(reader: R) -> Result<Palette, PaletteError> {
		let colors: Vec<String> = serde_json::from_reader(reader)
			.map_err(|_| { return PaletteError::InvalidFile; })?;

		if colors.len() > MAX_PALETTE_COLORS {
			return Err(PaletteError::TooManyColors);
		}

		let colors = colors.iter().enumerate().map(|(i, c)| {
			let trimmed = c.trim();
			let stripped = trimmed.strip_prefix("0x").unwrap_or(&trimmed);
			let stripped = stripped.strip_prefix("#").unwrap_or(&stripped);

			let parsed_int = u32::from_str_radix(stripped, 16)
				.map_err(|_| PaletteError::InvalidJsonEntry {
					index: i,
					msg: format!("\"{stripped}\" is not a valid hexadecimal color value")
				})?;

			Ok(Color::from(parsed_int))
		}).collect::<Result<Vec<Color>, PaletteError>>()?;

		Ok(Palette::from(colors))
	}

	pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Palette, PaletteError> {
		let f = File::open(path)?;
		let reader = BufReader::new(f);
		Self::from_json_internal(reader)
	}

	pub fn from_json_string<S: Into<String>>(s: S) -> Result<Palette, PaletteError> {
		let s = s.into();
		let mut reader = BufReader::new(s.as_bytes());
		Self::from_json_internal(&mut reader)
	}
}