use anyhow::Result;
use ffauto_rs::palettes::palette::PaletteError::{InvalidBinaryData, InvalidTextData};
use ffauto_rs::palettes::palette::{Palette, PaletteFormat};
use std::path::PathBuf;

#[test]
fn palette_parsing() -> Result<()> {
	for palette_type in PaletteFormat::VALUES {
		let mut test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		test_file.push("tests");
		test_file.push("palettes");
		test_file.push(format!("palette.{palette_type}"));

		println!("Testing {}…", palette_type.to_string().to_uppercase());

		let pal = Palette::load_from_file(&test_file).unwrap();

		if palette_type == PaletteFormat::AnimatorProCol {
			// the .col format always yields 256 colors
			assert_eq!(pal.len(), 256);
		} else {
			assert_eq!(pal.len(), 64);
		}

		if palette_type == PaletteFormat::Gpl {
			// the .gpl format supports names
			let names = pal.colors.iter().cloned().map(|c| c.name).collect::<Vec<String>>();
			assert!(names.iter().all(|n| n == "Untitled"));
		}

		assert_eq!(pal.colors[0].color.to_string(), "#1E3D54");
		assert_eq!(pal.colors[63].color.to_string(), "#E2EDF5");
	}

	Ok(())
}

#[test]
fn palette_parsing_errors() -> Result<()> {
	for palette_type in PaletteFormat::VALUES {
		let mut test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		test_file.push("tests");
		test_file.push(format!("palette_broken.{palette_type}"));

		println!("Testing broken {}…", palette_type.to_string().to_uppercase());

		let err = Palette::load_from_file(&test_file).unwrap_err();

		#[allow(unused_variables)]
		match palette_type {
			PaletteFormat::AdobeAct => {
				let expected = InvalidBinaryData { position: 768, msg: "Invalid footer value 0xFFFF".to_string() };
				assert!(matches!(err, expected));
			}
			PaletteFormat::AnimatorProCol => {
				let expected = InvalidBinaryData { position: 4, msg: "Invalid magic sequence 0xB124".to_string() };
				assert!(matches!(err, expected));
			}
			PaletteFormat::Gpl => {
				let expected = InvalidTextData { line: 4, msg: "Malformed line".to_string() };
				assert!(matches!(err, expected));
			}
			PaletteFormat::Hex => {
				let expected = InvalidTextData { line: 1, msg: "Not a hexadecimal color value".to_string() };
				assert!(matches!(err, expected));
			}
			PaletteFormat::Pal => {
				let expected = InvalidTextData { line: 4, msg: "Malformed line".to_string() };
				assert!(matches!(err, expected));
			}
		}
	}

	Ok(())
}