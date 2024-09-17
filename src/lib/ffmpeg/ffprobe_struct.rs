use crate::ffmpeg::ffprobe_struct::StreamType::Video;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::time::Duration;
use crate::ffmpeg::timestamps::parse_ffmpeg_duration;

#[derive(Debug, Clone, Deserialize)]
pub struct FFProbeOutput {
	pub streams: Vec<Stream>,
	pub format: Format,
}

impl FFProbeOutput {
	pub fn duration(&self) -> Result<Duration> {
		// intentionally not dealing with FloatParseErrors here.
		// if ffprobe ever feeds us bad data we've got bigger problems anyway

		// unwrapping here because the caller should've already done a stream check
		let video_stream = self.streams.iter().find(|s| s.codec_type == Video).unwrap();

		if let Some(stream_duration) = video_stream.duration.clone() {
			// println!("stream duration: {stream_duration}");
			return Ok(Duration::from_secs_f64(
				stream_duration.parse()
					.map_err(|e| anyhow!("{e}: stream duration \"{stream_duration}\""))?
			));
		}

		if let Some(tags_duration) = video_stream.tags.clone()
			.and_then(|t| t.duration)
			.and_then(|s| parse_ffmpeg_duration(&s)) {
			// println!("tags duration: {tags_duration:?}");
			return Ok(tags_duration);
		}

		// println!("format duration: {}", self.format.duration);
		Ok(Duration::from_secs_f64(self.format.duration.parse()?))
	}
}

#[derive(clap::ValueEnum, Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
	Audio,
	Video,
	Subtitle,
	Data,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Tags {
	#[serde(rename = "DURATION")]
	pub duration: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Format {
	pub duration: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Stream {
	pub index: u64,
	pub codec_name: Option<String>,
	pub codec_type: StreamType,
	pub width: Option<u64>,
	pub height: Option<u64>,
	pub pix_fmt: Option<String>,
	pub color_range: Option<String>,
	pub color_space: Option<String>,
	pub color_transfer: Option<String>,
	pub color_primaries: Option<String>,
	pub r_frame_rate: Option<String>,
	pub avg_frame_rate: Option<String>,
	pub sample_rate: Option<String>,
	pub channels: Option<u64>,
	pub bit_rate: Option<String>,
	pub duration: Option<String>,
	pub nb_read_frames: Option<u64>,
	pub tags: Option<Tags>,
}

impl Stream {
	pub fn frame_rate(&self) -> Option<f64> {
		match &self.r_frame_rate {
			None => { return None; }
			Some(fps) => {
				if fps.contains("/") {
					if let Some(split) = fps.split_once("/") {
						let left = split.0.parse::<f64>().unwrap();
						let right = split.1.parse::<f64>().unwrap();
						return Some(left / right);
					}
				} else {
					return fps.parse::<f64>().ok();
				}
			}
		}

		None
	}
}