mod crop;
mod fade;
mod palettegen;
mod scale;

pub use fade::{Fade, FadeType};
pub use palettegen::{Palettegen, StatsMode};

use std::fmt::Display;

pub trait FFmpegFilter: Display {
	const NAME: &str;
}
