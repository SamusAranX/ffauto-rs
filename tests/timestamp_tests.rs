use ffauto_rs::timestamps::{format_ffmpeg_timestamp, parse_ffmpeg_timestamp};

fn timestamp_data() -> Vec<(String, f64)> {
	vec![
		(String::from("01:59:24.32"), 7164.32),
		(String::from("01:59:24"   ), 7164.0 ),
		(String::from(   "59:24.32"), 3564.32),
		(String::from(   "59:24"   ), 3564.0 ),
		(String::from(      "24.32"),   24.32),
		(String::from(      "24"   ),   24.0 ),
		(String::from(      "20.32"),   20.32),
		(String::from(      "20"   ),   20.0 ),
		(String::from(       "2.32"),    2.32),
		(String::from(       "0.1" ),    0.1 ),
		(String::from(       "0.01"),    0.01),
		(String::from(       "0"   ),    0.0 ),
	]
}

#[test]
fn timestamp_parsing() {
	for test_datum in timestamp_data() {
		let timestamp_str = test_datum.0;
		let timestamp_float = test_datum.1;

		let dur = parse_ffmpeg_timestamp(&timestamp_str);
		assert_eq!(dur.as_secs_f64(), timestamp_float);

		let ts = format_ffmpeg_timestamp(dur);
		assert_eq!(ts, timestamp_str);
	}
}