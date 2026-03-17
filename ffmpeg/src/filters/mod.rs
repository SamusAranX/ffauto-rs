mod fade;
mod palettegen;

use std::fmt::Display;

pub trait FFmpegFilter: Display {
	const NAME: &str;
}
