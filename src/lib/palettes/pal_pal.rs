use crate::palettes::palette::{Color, Palette, PaletteError};
use crate::palettes::MAX_PALETTE_COLORS;
use regex::{Captures, Regex};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// https://github.com/aseprite/aseprite/blob/8323a555007e1db9670b098ce4b1b9c5f8b3d7ad/src/doc/file/pal_file.cpp

const PAL_MAGIC: &str = "JASC-PAL";
const PAL_VERSION: &str = "0100";

impl Palette {
	pub(crate) fn from_pal_file<P: AsRef<Path>>(path: P) -> Result<Palette, PaletteError> {
		let re = Regex::new(r"^(?P<r>\d+)\s+(?P<g>\d+)\s+(?P<b>\d+)$").unwrap();

		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		let mut pal = Palette::default();

		let mut magic = String::new();
		reader.read_line(&mut magic)?;
		if magic.trim() != PAL_MAGIC {
			return Err(PaletteError::InvalidTextData {
				line: 1,
				msg: format!("Invalid magic sequence: {magic}").to_string(),
			});
		}

		let mut version = String::new();
		reader.read_line(&mut version)?;
		if version.trim() != PAL_VERSION {
			return Err(PaletteError::InvalidTextData {
				line: 2,
				msg: format!("Invalid version: {version}").to_string(),
			});
		}

		// ignore the line with the number of colors, it's not important here
		reader.read_line(&mut version)?;

		for (i, line) in reader.lines().enumerate() {
			let trimmed_line = line?.trim().to_owned();
			if trimmed_line.is_empty() || trimmed_line.starts_with("#") {
				continue;
			}

			let groups: Captures = match re.captures(&trimmed_line) {
				None => {
					return Err(PaletteError::InvalidTextData {
						line: i + 3,
						msg: "Malformed line".to_string(),
					});
				}
				Some(captures) => captures
			};

			let mut col = Color::default();
			if let (Some(r), Some(g), Some(b)) = (groups.name("r"), groups.name("g"), groups.name("b")) {
				col.r = r.as_str().parse::<u8>().map_err(|_| PaletteError::InvalidTextData { line: i + 3, msg: "Invalid red value".to_string() })?;
				col.g = g.as_str().parse::<u8>().map_err(|_| PaletteError::InvalidTextData { line: i + 3, msg: "Invalid green value".to_string() })?;
				col.b = b.as_str().parse::<u8>().map_err(|_| PaletteError::InvalidTextData { line: i + 3, msg: "Invalid blue value".to_string() })?;
			} else {
				return Err(PaletteError::InvalidTextData { line: i + 3, msg: "Malformed line".to_string() });
			}

			pal.push_color(col);

			if pal.len() > MAX_PALETTE_COLORS {
				return Err(PaletteError::TooManyColors);
			}
		}

		Ok(pal)
	}
}