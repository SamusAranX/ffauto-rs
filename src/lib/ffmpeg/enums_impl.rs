use anyhow::{anyhow, Result};
use regex::Regex;
use std::fmt;

use crate::ffmpeg::enums::*;

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
		(self.default_crf() + (garbage * 3)).clamp(0, 51)
	}
}

impl Size {
	pub(crate) fn new(width: u64, height: u64) -> Self {
		Size { width, height }
	}
}

impl Crop {
	pub fn new<S: Into<String>>(crop_str: S) -> Result<Self> {
		let crop_str = crop_str.into();
		let re = Regex::new(r"(-?\d+)").unwrap();

		let numbers = re.find_iter(crop_str.as_str()).map(|s| {
			s.as_str().parse::<u64>()
				.map_err(|_| anyhow!("\"{crop_str}\" is not a valid crop value"))
		}).collect::<Result<Vec<u64>, anyhow::Error>>()?;

		match numbers.as_slice() {
			[h]
			if *h > 0 => {
				Ok(Crop {
					height: *h,
					..Crop::default()
				})
			}
			[w, h]
			if *w > 0 && *h > 0 => {
				Ok(Crop {
					width: *w,
					height: *h,
					..Crop::default()
				})
			}
			[w, h, x, y]
			if *w > 0 && *h > 0 => {
				Ok(Crop {
					width: *w,
					height: *h,
					x: *x,
					y: *y,
				})
			}
			_ => anyhow::bail!("\"{crop_str}\" is not a valid crop value")
		}
	}
}

impl fmt::Display for Crop {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// crop=w=100:h=100:x=12:y=34
		let mut crop_str = match (self.width, self.height) {
			(w, 0) => format!("w={w}"),
			(0, h) => format!("h={h}"),
			(w, h) => format!("w={w}:h={h}")
		};

		if self.x > 0 {
			crop_str += format!(":x={}", self.x).as_str();
		}
		if self.y > 0 {
			crop_str += format!(":y={}", self.y).as_str();
		}

		write!(f, "{crop_str}")
	}
}

impl fmt::Display for StatsMode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			StatsMode::Full => write!(f, "full"),
			StatsMode::Diff => write!(f, "diff"),
			StatsMode::Single => write!(f, "single"),
		}
	}
}

impl fmt::Display for DitherMode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			DitherMode::Bayer => write!(f, "bayer"),
			DitherMode::Heckbert => write!(f, "heckbert"),
			DitherMode::FloydSteinberg => write!(f, "floyd_steinberg"),
			DitherMode::Sierra2 => write!(f, "sierra2"),
			DitherMode::Sierra2_4a => write!(f, "sierra2-4a"),
			DitherMode::Sierra3 => write!(f, "sierra3"),
			DitherMode::Burkes => write!(f, "burkes"),
			DitherMode::Atkinson => write!(f, "atkinson"),
			DitherMode::None => write!(f, "none"),
		}
	}
}