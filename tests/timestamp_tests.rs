use ffauto_rs::ffmpeg::timestamps::TimestampFormat::{Auto, Full};
use ffauto_rs::ffmpeg::timestamps::{format_ffmpeg_timestamp, parse_ffmpeg_duration};
use std::time::Duration;

struct TimestampTest {
	pub input_timestamp: String,
	pub expected_duration: Duration,
	pub expected_full_timestamp: String,
}

impl TimestampTest {
	fn new<S: Into<String>>(input_ts: S, expected_dur: f64, expected_full_ts: S) -> Self {
		Self {
			input_timestamp: input_ts.into(),
			expected_duration: Duration::from_secs_f64(expected_dur),
			expected_full_timestamp: expected_full_ts.into(),
		}
	}
}

fn timestamp_data() -> Vec<TimestampTest> {
	vec![
		// rust does not have negative durations so negative timestamps can't be parsed currently
		// TimestampTest::new("-01:59:24.320000000", -7164.32, "-01:59:24.320000"),
		TimestampTest::new( "01:59:24.320000000",  7164.32,  "01:59:24.320000"),
		TimestampTest::new( "01:59:24.32",         7164.32,  "01:59:24.320000"),
		TimestampTest::new( "01:59:24"   ,         7164.0 ,  "01:59:24.000000"),
		TimestampTest::new( "01:59:02"   ,         7142.0 ,  "01:59:02.000000"),
		TimestampTest::new(    "59:24.32",         3564.32,  "00:59:24.320000"),
		TimestampTest::new(    "59:24"   ,         3564.0 ,  "00:59:24.000000"),
		TimestampTest::new(       "24.32",           24.32,  "00:00:24.320000"),
		TimestampTest::new(       "24"   ,           24.0 ,  "00:00:24.000000"),
		TimestampTest::new(       "20.32",           20.32,  "00:00:20.320000"),
		TimestampTest::new(       "20"   ,           20.0 ,  "00:00:20.000000"),
		TimestampTest::new(        "2.32",            2.32,  "00:00:02.320000"),
		TimestampTest::new(        "0.1" ,            0.1 ,  "00:00:00.100000"),
		TimestampTest::new(        "0.01",            0.01,  "00:00:00.010000"),
		TimestampTest::new(        "0"   ,            0.0 ,  "00:00:00.000000"),
	]
}

#[test]
fn timestamp_parsing() {
	for (i, test) in timestamp_data().iter().enumerate() {
		let dur = parse_ffmpeg_duration(&test.input_timestamp);
		assert!(dur.is_some(), "{i}: parsing failed!");
		assert_eq!(&dur.unwrap(), &test.expected_duration, "{i}: durations aren't equal!");

		if !test.input_timestamp.ends_with("320000000") {
			let ts = format_ffmpeg_timestamp(dur.unwrap(), Auto);
			assert_eq!(&ts, &test.input_timestamp, "{i}: formatted duration string doesn't match!");
		}

		let ts = format_ffmpeg_timestamp(dur.unwrap(), Full);
		assert_eq!(&ts, &test.expected_full_timestamp, "{i}: full formatted duration string doesn't match!");

		// println!("---");
	}
}
