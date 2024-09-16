use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::palettes::palette::{Color, Palette, PaletteError};
use crate::palettes::MAX_PALETTE_COLORS;

// https://github.com/aseprite/aseprite/blob/8323a555007e1db9670b098ce4b1b9c5f8b3d7ad/src/doc/file/col_file.cpp

const PRO_MAGIC: u16 = 0xB123;

fn div_rem<T: std::ops::Div<Output=T> + std::ops::Rem<Output=T> + Copy>(x: T, y: T) -> (T, T) {
	let quot = x / y;
	let rem = x % y;
	(quot, rem)
}

impl Palette {
	pub(crate) fn from_col_file<P: AsRef<Path>>(path: P) -> Result<Palette, PaletteError> {
		let mut f = File::open(path)?;
		let size = f.metadata().unwrap().len();
		let mut pal = Palette::default();

		let pro = size != 768;

		if pro && div_rem(size - 8, 3).1 != 0 {
			eprintln!("{pro}, {:?}", div_rem(size - 8, 3));
			return Err(PaletteError::InvalidBinaryData { position: 0, msg: "Not an Animator COL file".to_string() });
		}

		if pro { // Animator Pro checks
			f.seek(SeekFrom::Start(4))?; // skip file size

			let magic = f.read_u16::<LittleEndian>()?;
			if magic != PRO_MAGIC {
				return Err(PaletteError::InvalidBinaryData {
					position: (f.stream_position().unwrap() - 2) as usize,
					msg: format!("Invalid magic sequence {magic:#02X}"),
				});
			}

			let version = f.read_u16::<LittleEndian>()?;
			if version != 0 {
				return Err(PaletteError::InvalidBinaryData {
					position: (f.stream_position().unwrap() - 2) as usize,
					msg: format!("Invalid version {version:#02X}"),
				});
			}
		}

		let mut buf = [0_u8; 3];
		for _ in 0..MAX_PALETTE_COLORS {
			f.read_exact(&mut buf)?;
			if pro {
				pal.push_color(Color::from(buf));
			} else {
				pal.push_color(Color::from_6bits(buf));
			}
		}

		Ok(pal)
	}
}