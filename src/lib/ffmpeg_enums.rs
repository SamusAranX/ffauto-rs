#[derive(clap::ValueEnum, Clone, Default, Debug)]
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

#[derive(clap::ValueEnum, Clone, Default, Debug)]
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

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum VideoCodec {
	#[default] H264,
	H265,
	H265_10,
	GIF,
}

#[derive(Clone, Default, Debug)]
pub struct Crop {
	pub width: u64,
	pub height: u64,
	pub x: u64,
	pub y: u64,
}