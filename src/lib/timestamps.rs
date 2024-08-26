use std::time::Duration;

use regex::{Captures, Regex};

/// Takes an ffmpeg-esque timestamp and parses it into a Duration.
pub fn parse_ffmpeg_timestamp(timestamp: &str) -> Duration {
	let re = Regex::new(r"^(?:(?:(?P<hours>\d+):)?(?P<minutes>\d+):)?(?P<seconds>\d+)(?:\.?(?P<millis>\d+))?$").unwrap();

	let groups: Captures = match re.captures(timestamp) {
		None => { return Duration::ZERO; }
		Some(captures) => captures
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
		duration += Duration::from_millis(millis_str.parse::<u64>().unwrap_or_default());
	}

	duration
}

/// Takes a Duration and formats it like a timestamp ffmpeg would use. Mainly for display purposes.
pub fn format_ffmpeg_timestamp(duration: Duration) -> String {
	let secs_total = duration.as_secs_f64();
	let hours = (secs_total / 3600.0).floor();
	let minutes = (secs_total % 3600.0 / 60.0).floor();
	let secs = (secs_total % 60.0).floor() as u64;
	let millis = secs_total.fract();

	// trim the leading zero off the millis string, also remove any trailing zeroes and if applicable, the remaining period as well
	let millis_str = format!("{millis:0.2}");
	let millis_trimmed = millis_str.trim_start_matches("0").trim_end_matches(['.', '0']);

	let formatted: String;
	if secs_total >= 3600.0 {
		formatted = format!("{hours:0>2}:{minutes:0>2}:{secs}{millis_trimmed}");
	} else if secs_total >= 60.0 {
		formatted = format!("{minutes:0>2}:{secs}{millis_trimmed}");
	} else {
		formatted = format!("{secs}{millis_trimmed}");
	}

	formatted
}