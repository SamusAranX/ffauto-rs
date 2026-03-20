mod audio;
mod video;

pub use video::crop::{Crop};
pub use video::fade::{Fade, FadeType};
pub use video::palettegen::{Palettegen, StatsMode};
pub use video::scale::{Scale};
pub use video::tonemap::{Tonemap};

use std::fmt::Display;

pub trait FFmpegFilter: Display {
	const NAME: &str;
}
