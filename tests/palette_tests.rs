use std::fs;
use std::path::PathBuf;

use ffauto_rs::palettes::palette::{Palette, PaletteFormat};

const FILE_FORMATS: [PaletteFormat; 6] = [
	PaletteFormat::AdobeAct,
	PaletteFormat::AnimatorProCol,
	PaletteFormat::Gpl,
	PaletteFormat::Hex,
	PaletteFormat::Json,
	PaletteFormat::Pal
];
const TEXT_FORMATS: [PaletteFormat; 4] = [PaletteFormat::Gpl, PaletteFormat::Hex, PaletteFormat::Json, PaletteFormat::Pal];

#[test]
fn palette_parsing() {
	for palette_type in FILE_FORMATS {
		let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join(format!("tests/palettes/palette.{palette_type}"));

		println!("Testing {} from file…", palette_type.to_string().to_uppercase());

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
}

#[test]
fn palette_parsing_from_string() {
	for palette_type in TEXT_FORMATS {
		let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join(format!("tests/palettes/palette.{palette_type}"));

		println!("Testing {} from String…", palette_type.to_string().to_uppercase());

		let pal_contents = fs::read_to_string(test_file).unwrap();
		let pal = Palette::load_from_string(pal_contents, palette_type).unwrap();

		assert_eq!(pal.len(), 64);

		if palette_type == PaletteFormat::Gpl {
			// the .gpl format supports names
			let names = pal.colors.iter().cloned().map(|c| c.name).collect::<Vec<String>>();
			assert!(names.iter().all(|n| n == "Untitled"));
		}

		assert_eq!(pal.colors[0].color.to_string(), "#1E3D54");
		assert_eq!(pal.colors[63].color.to_string(), "#E2EDF5");
	}
}

#[test]
#[should_panic(expected = "InvalidBinaryData { position: 768, msg: \"Invalid footer value 0xFFFF\" }")]
fn palette_parsing_broken_act() {
	println!("Testing broken ACT…");
	let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("tests/palettes/palette_broken.act");

	Palette::load_from_file(&test_file).unwrap();
}

#[test]
#[should_panic(expected = "InvalidBinaryData { position: 4, msg: \"Invalid magic sequence 0xB124\" }")]
fn palette_parsing_broken_col() {
	println!("Testing broken COL…");
	let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("tests/palettes/palette_broken.col");

	Palette::load_from_file(&test_file).unwrap();
}

#[test]
#[should_panic(expected = "InvalidTextLine { line: 4, msg: \"Malformed line\" }")]
fn palette_parsing_broken_gpl() {
	println!("Testing broken GPL…");
	let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("tests/palettes/palette_broken.gpl");

	Palette::load_from_file(&test_file).unwrap();
}

#[test]
#[should_panic(expected = "InvalidTextLine { line: 1, msg: \"Not a hexadecimal color value\" }")]
fn palette_parsing_broken_hex() {
	println!("Testing broken HEX…");
	let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("tests/palettes/palette_broken.hex");

	Palette::load_from_file(&test_file).unwrap();
}

#[test]
#[should_panic(expected = "InvalidJsonEntry { index: 1, msg: \"\\\"not a color\\\" is not a valid hexadecimal color value\" }")]
fn palette_parsing_broken_json() {
	println!("Testing broken JSON…");
	let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("tests/palettes/palette_broken.json");

	Palette::load_from_file(&test_file).unwrap();
}

#[test]
#[should_panic(expected = "InvalidTextLine { line: 4, msg: \"Malformed line\" }")]
fn palette_parsing_broken_pal() {
	println!("Testing broken PAL…");
	let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("tests/palettes/palette_broken.pal");

	Palette::load_from_file(&test_file).unwrap();
}