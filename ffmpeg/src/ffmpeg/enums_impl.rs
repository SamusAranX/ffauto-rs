use crate::ffmpeg::enums::*;

impl VideoCodec {
	#[must_use]
	pub fn video_codec(&self) -> &str {
		match self {
			VideoCodec::H264 => "libx264",
			VideoCodec::H265 | VideoCodec::H265_10 => "libx265",
		}
	}

	#[must_use]
	#[expect(clippy::unnecessary_literal_bound)]
	pub fn audio_codec(&self) -> &str {
		"aac"
	}

	#[must_use]
	pub fn pix_fmt(&self) -> &str {
		match self {
			VideoCodec::H264 | VideoCodec::H265 => "yuv420p",
			VideoCodec::H265_10 => "yuv420p10le",
		}
	}

	#[must_use]
	pub fn default_crf(&self) -> u8 {
		match self {
			VideoCodec::H264 => 23,
			VideoCodec::H265 | VideoCodec::H265_10 => 28,
		}
	}

	#[must_use]
	pub fn crf_with_garbage(&self, garbage: u8) -> u8 {
		(self.default_crf() + (garbage * 3)).clamp(0, 51)
	}
}
