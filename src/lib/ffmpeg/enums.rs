#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq)]
pub enum ScaleMode {
	Nearest,
	Bilinear,
	FastBilinear,
	Bicublin,
	#[default] Bicubic,
	Area,
	Gauss,
	Sinc,
	Lanczos,
	Spline,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq)]
pub enum Preset {
	UltraFast,
	SuperFast,
	VeryFast,
	Faster,
	Fast,
	Medium,
	#[default] Slow,
	Slower,
	VerySlow,
	Placebo,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq)]
pub enum VideoCodec {
	#[default] H264,
	H265,
	H265_10,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Size {
	pub width: u64,
	pub height: u64,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Crop {
	pub width: u64,
	pub height: u64,
	pub x: u64,
	pub y: u64,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq)]
pub enum StatsMode {
	#[default] Full,
	Diff,
	Single,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq)]
pub enum DitherMode {
	Bayer,
	Heckbert,
	FloydSteinberg,
	Sierra2,
	#[default] Sierra2_4a,
	Sierra3,
	Burkes,
	Atkinson,
	None,
}