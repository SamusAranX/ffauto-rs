use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::palettes::MAX_PALETTE_COLORS;
use crate::palettes::palette::{Color, Palette, PaletteError};

// https://github.com/aseprite/aseprite/blob/8323a555007e1db9670b098ce4b1b9c5f8b3d7ad/src/doc/file/hex_file.cpp

impl Palette {
	fn from_hex_internal<R: Read + BufRead>(reader: R) -> Result<Palette, PaletteError> {
		let mut pal = Palette::default();

		for (i, line) in reader.lines().enumerate() {
			let trimmed_line = line?.trim().to_owned();
			if trimmed_line.is_empty() || trimmed_line.starts_with("#") {
				continue;
			}

			// remove common hexadecimal prefixes from the string prior to parsing
			let trimmed_line = trimmed_line.strip_prefix("0x").unwrap_or(&trimmed_line);
			let trimmed_line = trimmed_line.strip_prefix("#").unwrap_or(trimmed_line);

			let parsed_int = u32::from_str_radix(trimmed_line, 16)
				.map_err(|_| PaletteError::InvalidTextLine { line: i, msg: "Not a hexadecimal color value".to_string() })?;

			pal.push_color(Color::from(parsed_int));

			if pal.len() > MAX_PALETTE_COLORS {
				return Err(PaletteError::TooManyColors);
			}
		}

		Ok(pal)
	}

	pub fn from_hex_file<P: AsRef<Path>>(path: P) -> Result<Palette, PaletteError> {
		let f = File::open(path)?;
		let reader = BufReader::new(f);
		Self::from_hex_internal(reader)
	}

	pub fn from_hex_string<S: Into<String>>(s: S) -> Result<Palette, PaletteError> {
		let s = s.into();
		let mut reader = BufReader::new(s.as_bytes());
		Self::from_hex_internal(&mut reader)
	}
}