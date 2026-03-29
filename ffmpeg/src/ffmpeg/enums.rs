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
