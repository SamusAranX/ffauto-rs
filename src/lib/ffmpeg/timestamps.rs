use std::time::Duration;

use regex::{Captures, Regex};

/// Takes an ffmpeg-esque duration string and parses it into a [Duration].
/// Invalid input will return [None].
pub fn parse_ffmpeg_duration<S: Into<String>>(timestamp: S) -> Option<Duration> {
	let timestamp = timestamp.into();

	if timestamp == "N/A" {
		return None;
	}

	if let Ok(f) = timestamp.parse::<f64>() {
		return Some(Duration::from_secs_f64(f));
	}

	let re = Regex::new(r"^(?:(?:(?P<hours>\d+):)?(?P<minutes>\d+):)?(?P<seconds>\d+)(?:\.?(?P<millis>\d+))?$").unwrap();

	let groups: Captures = match re.captures(&timestamp) {
		None => {
			#[cfg(debug_assertions)]
			eprintln!("invalid duration string: {timestamp}");
			return None;
		}
		Some(captures) => captures,
	};

	let mut duration = Duration::ZERO;
	if let Some(hours) = groups.name("hours") {
		duration += Duration::from_secs(hours.as_str().parse::<u64>().unwrap_or_default() * 3600);
	}

	if let Some(minutes) = groups.name("minutes") {
		duration += Duration::from_secs(minutes.as_str().parse::<u64>().unwrap_or_default() * 60);
	}

	if let Some(seconds) = groups.name("seconds") {
		duration += Duration::from_secs(seconds.as_str().parse::<u64>().unwrap_or_default());
	}

	if let Some(millis) = groups.name("millis") {
		let millis_str = format!("{:0<3}", millis.as_str());
		let millis = millis_str.parse::<u64>().unwrap_or_default();
		if millis >= 1000000000 {
			// picoseconds. idk if ffmpeg/ffprobe return these but just in case we'll ignore these
			eprintln!("ignoring picoseconds in duration string")
		} else if millis >= 1000000 {
			duration += Duration::from_nanos(millis);
		} else if millis >= 1000 {
			duration += Duration::from_micros(millis);
		} else {
			duration += Duration::from_millis(millis);
		}
	}

	Some(duration)
}

pub enum TimestampFormat {
	Auto,
	Full,
	TwoDigits,
}

/// Takes a [Duration] and formats it like a timestamp ffmpeg would use. Mainly for display purposes.
///
/// Specifying `TimestampFormat::Auto` will make this function return `SS.fff` , then `MM:SS.fff`, then `HH:MM:SS.fff`.
///
/// Specifying `TimestampFormat::Full` will make this function always return a timestamp of format `HH:MM:SS.ffffff`.
///
/// Specifying `TimestampFormat::TwoDigits` will make this function always return a timestamp of format `HH:MM:SS.ff`.
pub fn format_ffmpeg_timestamp(duration: Duration, format: TimestampFormat) -> String {
	let secs_total = duration.as_secs() as f64;
	let hours = (secs_total / 3600.0).floor();
	let minutes = (secs_total % 3600.0 / 60.0).floor();
	let secs = (secs_total % 60.0).floor() as u64;
	let millis = duration.subsec_millis();

	match format {
		TimestampFormat::Auto => {
			let millis_str = format!(".{millis:0>3}");
			let millis_str = millis_str.trim_end_matches("0").trim_end_matches(".");
			if secs_total >= 3600.0 {
				format!("{:0>2}:{:0>2}:{secs:0>2}{millis_str}", hours as u64, minutes as u64)
			} else if secs_total >= 60.0 {
				format!("{:0>2}:{secs:0>2}{millis_str}", minutes as u64)
			} else {
				format!("{secs}{millis_str}")
			}
		}
		TimestampFormat::Full => {
			let micros = duration.subsec_micros();
			format!("{:0>2}:{:0>2}:{secs:0>2}.{micros:0>6}", hours as u64, minutes as u64)
		}
		TimestampFormat::TwoDigits => {
			let millis_two_digits = (duration.as_secs_f64().fract() * 100.0).floor() as u64;
			format!("{:0>2}:{:0>2}:{secs:0>2}.{millis_two_digits:0>2}", hours as u64, minutes as u64)
		}
	}
}
