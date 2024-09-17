use ffauto_rs::ffmpeg::timestamps::{format_ffmpeg_timestamp, parse_ffmpeg_timestamp};

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
	for (ts_str, ts_float) in timestamp_data() {
		// print!("parsing string \"{timestamp_str}\", expecting duration {timestamp_float:?}… ");
		let dur = parse_ffmpeg_timestamp(&ts_str);
		assert!(dur.is_some());
		let dur_float = dur.unwrap().as_secs_f64();
		// println!("got {dur_float:?}");
		assert_eq!(dur_float, ts_float);

		// print!("formatting duration {dur_float:?}, expecting string \"{timestamp_str}\"… ");
		let ts = format_ffmpeg_timestamp(dur.unwrap());
		// println!("got \"{ts}\"");
		assert_eq!(ts, ts_str);

		// println!("---");
	}
}