use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use regex::{Captures, Regex};

use crate::palettes::palette::{Color, Palette, PaletteError};
use crate::palettes::MAX_PALETTE_COLORS;

// https://github.com/aseprite/aseprite/blob/8323a555007e1db9670b098ce4b1b9c5f8b3d7ad/src/doc/file/gpl_file.cpp

const GIMP_MAGIC: &str = "GIMP Palette";

impl Palette {
	fn from_gpl_internal<R: Read + BufRead>(reader: &mut R) -> Result<Palette, PaletteError> {
		let re = Regex::new(r"^\s*(?P<r>\d+)\s+(?P<g>\d+)\s+(?P<b>\d+)(?:\s+(?P<a>\d+))?\s+(?P<name>.*)?$").unwrap();

		let mut pal = Palette::default();

		let mut magic = String::new();
		reader.read_line(&mut magic)?;
		if magic.trim() != GIMP_MAGIC {
			return Err(PaletteError::InvalidTextLine {
				line: 1,
				msg: format!("Invalid magic sequence: \"{magic}\"").to_string(),
			});
		}

		for (i, line) in reader.lines().enumerate() {
			let trimmed_line = line?.trim().to_owned();
			if trimmed_line.starts_with("#") || trimmed_line.is_empty() ||
				trimmed_line.starts_with("Name: ") || trimmed_line.starts_with("Columns: ") {
				continue;
			}

			let groups: Captures = match re.captures(&trimmed_line) {
				None => {
					return Err(PaletteError::InvalidTextLine {
						line: i + 2,
						msg: "Malformed line".to_string(),
					});
				}
				Some(captures) => captures
			};

			let mut col = Color::default();
			if let (Some(r), Some(g), Some(b)) = (groups.name("r"), groups.name("g"), groups.name("b")) {
				col.r = r.as_str().parse::<u8>().map_err(|_| PaletteError::InvalidTextLine { line: i + 2, msg: "Invalid red value".to_string() })?;
				col.g = g.as_str().parse::<u8>().map_err(|_| PaletteError::InvalidTextLine { line: i + 2, msg: "Invalid green value".to_string() })?;
				col.b = b.as_str().parse::<u8>().map_err(|_| PaletteError::InvalidTextLine { line: i + 2, msg: "Invalid blue value".to_string() })?;
			} else {
				return Err(PaletteError::InvalidTextLine { line: i + 2, msg: "Malformed line".to_string() });
			}

			if let Some(name) = groups.name("name") {
				pal.push_named_color(col, name.as_str().to_string());
			} else {
				pal.push_color(col);
			}

			if pal.len() > MAX_PALETTE_COLORS {
				return Err(PaletteError::TooManyColors);
			}
		}

		Ok(pal)
	}

	pub(crate) fn from_gpl_file<P: AsRef<Path>>(path: P) -> Result<Palette, PaletteError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		Self::from_gpl_internal(&mut reader)
	}

	pub fn from_gpl_string<S: Into<String>>(s: S) -> Result<Palette, PaletteError> {
		let s = s.into();
		let mut reader = BufReader::new(s.as_bytes());
		Self::from_gpl_internal(&mut reader)
	}
}