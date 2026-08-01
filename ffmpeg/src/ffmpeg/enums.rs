#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq, strum::Display, strum::EnumString)]
pub enum VideoCodec {
	#[default]
	#[strum(serialize = "h264")]
	H264,

	#[strum(serialize = "h265")]
	H265,

	#[strum(serialize = "h265-10")]
	H265_10,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, strum::Display, strum::EnumString)]
pub enum OptimizeTarget {
	#[strum(serialize = "ipod5")]
	Ipod5, // earliest video-capable iPod

	#[strum(serialize = "ipod")]
	Ipod, // newer video-capable iPods

	#[strum(serialize = "psp")]
	Psp,

	#[strum(serialize = "psvita")]
	PsVita,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq, strum::Display, strum::EnumString)]
pub enum BarcodeMode {
	#[default]
	#[strum(serialize = "frames")]
	Frames,
	#[strum(serialize = "colors")]
	Colors,
}

/// Encoder speed/compression tradeoff. Shared by libx264 and libx265.
/// ffmpeg's actual default is "medium" but we'll default to "slow" because "slower" is *too* slow.
#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Preset {
	UltraFast,
	SuperFast,
	VeryFast,
	Faster,
	Fast,
	Medium,
	#[default]
	Slow,
	Slower,
	VerySlow,
	Placebo,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, strum::Display, strum::EnumString)]
pub enum TargetVideoRange {
	#[strum(serialize = "full")]
	Full,
	#[strum(serialize = "limited")]
	Limited,
}
