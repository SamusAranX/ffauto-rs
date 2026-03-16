mod fade;

use std::fmt::Display;

trait FFmpegFilter: Display {
	const NAME: &str;

	fn to_filter_string(&self) -> String;
}
