use crate::ffmpeg_enums::*;
use regex::{Captures, Regex};
use std::fmt;

impl fmt::Display for ScaleMode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ScaleMode::Nearest => write!(f, "neighbor"),
			ScaleMode::Bilinear => write!(f, "bilinear"),
			ScaleMode::FastBilinear => write!(f, "fast_bilinear"),
			ScaleMode::Bicublin => write!(f, "bicublin"),
			ScaleMode::Bicubic => write!(f, "bicubic"),
			ScaleMode::Area => write!(f, "area"),
			ScaleMode::Gauss => write!(f, "gauss"),
			ScaleMode::Sinc => write!(f, "sinc"),
			ScaleMode::Lanczos => write!(f, "lanczos"),
			ScaleMode::Spline => write!(f, "spline"),
		}
	}
}

impl fmt::Display for Preset {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Preset::UltraFast => write!(f, "ultrafast"),
			Preset::SuperFast => write!(f, "superfast"),
			Preset::VeryFast => write!(f, "veryfast"),
			Preset::Faster => write!(f, "faster"),
			Preset::Fast => write!(f, "fast"),
			Preset::Medium => write!(f, "medium"),
			Preset::Slow => write!(f, "slow"),
			Preset::Slower => write!(f, "slower"),
			Preset::VerySlow => write!(f, "veryslow"),
			Preset::Placebo => write!(f, "placebo"),
		}
	}
}

impl fmt::Display for VideoCodec {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			VideoCodec::H264 => write!(f, "h264"),
			VideoCodec::H265 | VideoCodec::H265_10 => write!(f, "h265"),
		}
	}
}

impl VideoCodec {
	pub fn video_codec(&self) -> &str {
		match self {
			VideoCodec::H264 => "libx264",
			VideoCodec::H265 | VideoCodec::H265_10 => "libx265",
		}
	}

	pub fn audio_codec(&self) -> &str {
		match self {
			VideoCodec::H264 => "aac",
			VideoCodec::H265 | VideoCodec::H265_10 => "aac",
		}
	}

	pub fn pix_fmt(&self) -> &str {
		match self {
			VideoCodec::H264 | VideoCodec::H265 => "yuv420p",
			VideoCodec::H265_10 => "yuv420p10le",
		}
	}

	pub fn default_crf(&self) -> u8 {
		match self {
			VideoCodec::H264 => 23,
			VideoCodec::H265 | VideoCodec::H265_10 => 28,
		}
	}

	pub fn crf_with_garbage(&self, garbage: u8) -> u8 {
		match self {
			_ => (self.default_crf() + (garbage * 3)).clamp(0, 51),
		}
	}
}

impl Crop {
	pub fn new(crop_str: &str) -> Option<Self> {
		let re = Regex::new(r"^(?P<W>\d+)\D(?P<H>\d+)(?:\D?(?P<X>\d+)\D(?P<Y>\d+))?$").unwrap();

		let mut crop = Self::default();

		let groups: Captures = match re.captures(crop_str) {
			None => { return None; }
			Some(captures) => captures
		};

		if let (Some(w), Some(h)) = (groups.name("W"), groups.name("H")) {
			crop.width = w.as_str().parse::<u64>().unwrap_or_default();
			crop.height = h.as_str().parse::<u64>().unwrap_or_default();
		}

		if let (Some(x), Some(y)) = (groups.name("X"), groups.name("Y")) {
			crop.x = x.as_str().parse::<u64>().unwrap_or_default();
			crop.y = y.as_str().parse::<u64>().unwrap_or_default();
		}

		Some(crop)
	}
}

impl fmt::Display for Crop {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// crop=w=100:h=100:x=12:y=34
		let width = if self.width > 0 { self.width.to_string() } else { "iw".to_string() };
		let height = if self.height > 0 { self.height.to_string() } else { "ih".to_string() };
		let mut crop_str = format!("w={width}:h={height}");

		if self.x > 0 {
			crop_str += format!(":x={}", self.x).as_str();
		}
		if self.y > 0 {
			crop_str += format!(":y={}", self.y).as_str();
		}

		write!(f, "{crop_str}")
	}
}