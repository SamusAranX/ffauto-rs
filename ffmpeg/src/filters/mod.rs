#![allow(clippy::doc_markdown)]

use std::fmt::Display;

mod audio;
mod video;

pub trait FFmpegFilter: Display {
	const NAME: &str;
	
	// fn name(&self) -> String;
}

pub use audio::afade::{Afade, Curve as AfadeCurve, Type as AfadeType};
pub use audio::asplit::Asplit;
pub use audio::loudnorm::{Loudnorm, PrintFormat as LoudnormPrintFormat};
pub use audio::volume::{Precision as VolumePrecision, ReplayGain as VolumeReplayGain, Volume};

pub use video::blend::{Blend, Mode as BlendMode};
pub use video::color::Color;
pub use video::colorspace::{
	All as ColorspaceAll, ClipGamut as ColorspaceClipGamut, Colorspace, Dither as ColorspaceDither,
	Format as ColorspaceFormat, Primaries as ColorspacePrimaries, Range as ColorspaceRange, Space as ColorspaceSpace,
	Transfer as ColorspaceTransfer, WhitepointAdapt as ColorspaceWhitepointAdapt,
};
pub use video::crop::Crop;
pub use video::eq::Eq;
pub use video::fade::{Fade, Type as FadeType};
pub use video::format::Format;
pub use video::fps::Fps;
pub use video::palettegen::{Palettegen, StatsMode as PalettegenStatsMode};
pub use video::paletteuse::{DiffMode as PaletteuseDiffMode, Dither as PaletteuseDither, Paletteuse};
pub use video::scale::{Algorithm as ScaleAlgorithm, ForceOriginalAspectRatio as ScaleForceOriginalAspectRatio, Scale};
pub use video::select::Select;
pub use video::setparams::{
	AlphaMode as SetParamsAlphaMode, ChromaLocation as SetParamsChromaLocation,
	ColorPrimaries as SetParamsColorPrimaries, ColorTransfer as SetParamsColorTransfer,
	Colorspace as SetParamsColorspace, FieldMode as SetParamsFieldMode, Range as SetParamsRange, SetParams,
};
pub use video::setsar::SetSar;
pub use video::split::Split;
pub use video::subtitles::Subtitles;
pub use video::tile::Tile;
pub use video::tonemap::{Algorithm as TonemapAlgorithm, Tonemap};
pub use video::unsharp::Unsharp;
pub use video::zscale::{
	ChromaLoc as ZscaleChromaLoc, Dither as ZscaleDither, Filter as ZscaleFilter, Matrix as ZscaleMatrix,
	Primaries as ZscalePrimaries, Range as ZscaleRange, Transfer as ZscaleTransfer, Zscale,
};
