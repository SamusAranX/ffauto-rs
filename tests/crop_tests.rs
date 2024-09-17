// 1608
//
// 3840x1608
//
// 1234x5678,9;0

use ffauto_rs::ffmpeg::enums::Crop;

fn crop_valid() -> Vec<(String, Crop)> {
	vec![
		(String::from("1608"), Crop { height: 1608, ..Crop::default() }),
		(String::from("3840x1608"), Crop { width: 3840, height: 1608, ..Crop::default() }),
		(String::from("3840x1608;32x64"), Crop { width: 3840, height: 1608, x: 32, y: 64 }),
		(String::from("3840x1608x32x64"), Crop { width: 3840, height: 1608, x: 32, y: 64 }),
		(String::from("3840:1608:32:64"), Crop { width: 3840, height: 1608, x: 32, y: 64 }),
	]
}

fn crop_invalid() -> Vec<String> {
	vec![
		String::new(),
		String::from("0"),
		String::from("0x0"),
		String::from("0x0x0x0"),
		String::from("-9000"),
		String::from("3840x1608;32"),
		String::from("3840x-1608;32x64"),
	]
}

#[test]
fn crop_parsing() {
	for (crop_str, expected) in crop_valid() {
		let crop = Crop::new(crop_str).unwrap();
		assert_eq!(crop, expected);
	}

	for crop_str in crop_invalid() {
		let _ = Crop::new(crop_str).unwrap_err();
	}
}