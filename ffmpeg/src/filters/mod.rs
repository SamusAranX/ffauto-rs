mod crop;
mod fade;
mod palettegen;
mod scale;
mod tonemap;

pub use crop::{Crop};
pub use fade::{Fade, FadeType};
pub use palettegen::{Palettegen, StatsMode};
pub use scale::{Scale};
pub use tonemap::{Tonemap};

use std::fmt::Display;

pub trait FFmpegFilter: Display {
	const NAME: &str;
}
