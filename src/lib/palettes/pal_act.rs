use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

use crate::palettes::MAX_PALETTE_COLORS;
use crate::palettes::palette::{Color, Palette, PaletteError};

// https://github.com/aseprite/aseprite/blob/8323a555007e1db9670b098ce4b1b9c5f8b3d7ad/src/doc/file/act_file.cpp

impl Palette {
	pub(crate) fn from_act_file<P: AsRef<Path>>(path: P) -> Result<Palette, PaletteError> {
		let mut f = File::open(path)?;
		let size = f.metadata().unwrap().len();

		let mut pal = Palette::default();

		let mut buf = [0_u8; 3];
		for _ in 0..MAX_PALETTE_COLORS {
			f.read_exact(&mut buf)?;
			pal.push_color(Color::from(buf));
		}

		if f.stream_position().unwrap() < size {
			// the two bytes after the first 256 3-byte colors are a u16 containing the total number of colors
			let mut buf = [0_u8; 2];
			f.read_exact(&mut buf)?;
			let num_colors = u16::from_be_bytes(buf);

			if num_colors as usize > pal.len() || num_colors as usize > MAX_PALETTE_COLORS {
				return Err(PaletteError::InvalidBinaryData {
					position: (f.stream_position().unwrap() - 2) as usize,
					msg: format!("Invalid footer value {num_colors:#X}"),
				});
			}

			pal.colors.truncate(num_colors as usize);
		}

		Ok(pal)
	}
}