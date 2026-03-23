mod audio;
mod video;

pub use audio::afade::{Afade, AfadeCurve, AfadeType};
pub use audio::loudnorm::{Loudnorm, LoudnormPrintFormat};
pub use audio::volume::{Volume, VolumePrecision, VolumeReplaygain};

pub use video::color::Color;
pub use video::crop::Crop;
pub use video::eq::Eq;
pub use video::fade::{Fade, FadeType};
pub use video::format::Format;
pub use video::fps::Fps;
pub use video::palettegen::{Palettegen, PalettegenStatsMode};
pub use video::paletteuse::{Paletteuse, PaletteuseDiffMode, PaletteuseDither};
pub use video::scale::{Scale, ScaleAlgorithm, ScaleForceOriginalAspectRatio};
pub use video::tonemap::{Tonemap, TonemapAlgorithm};
pub use video::unsharp::Unsharp;
pub use video::zscale::{
	Zscale, ZscaleChromaLoc, ZscaleDither, ZscaleFilter, ZscaleMatrix, ZscalePrimaries, ZscaleRange, ZscaleTransfer,
};

use std::fmt::Display;

pub trait FFmpegFilter: Display {
	const NAME: &str;
}
