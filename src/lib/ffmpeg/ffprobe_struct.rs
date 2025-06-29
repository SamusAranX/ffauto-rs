use crate::ffmpeg::timestamps::parse_ffmpeg_duration;
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct FFProbeOutput {
	pub streams: Vec<Stream>,
	pub format: Format,
}

impl FFProbeOutput {
	pub fn duration(&self) -> Result<Duration> {
		// intentionally not dealing with FloatParseErrors here.
		// if ffprobe ever feeds us bad data we've got bigger problems anyway

		let video_stream = self
			.streams
			.iter()
			.find(|s| s.codec_type == StreamType::Video)
			.ok_or_else(|| anyhow!("The input file needs to contain a usable video stream"))?;

		if let Some(stream_duration) = video_stream.duration.clone() {
			// return first video stream duration
			return Ok(Duration::from_secs_f64(
				stream_duration.parse().map_err(|e| anyhow!("{e}: stream duration \"{stream_duration}\""))?,
			));
		}

		if let Some(tags_duration) = video_stream.tags.clone().and_then(|t| t.duration).and_then(|s| parse_ffmpeg_duration(&s)) {
			// return first video stream tags duration
			return Ok(tags_duration);
		}

		if let Some(format_duration) = &self.format.duration {
			// return format duration
			return Ok(Duration::from_secs_f64(format_duration.parse()?));
		}

		if let (Some(read_frames), Some(frame_rate)) = (&video_stream.nb_read_frames, video_stream.frame_rate()) {
			// divide number of frames by frame rate and return the result

			let read_frames = read_frames.parse::<f64>()?;
			return Ok(Duration::from_secs_f64(read_frames / frame_rate));
		}

		anyhow::bail!("ffprobe could not find a duration for the input file")
	}

	pub fn get_stream(&self, index: usize) -> Option<&Stream> {
		self.streams.get(index)
	}

	fn get_typed_stream(&self, index: usize, stream_type: StreamType) -> Option<&Stream> {
		self.streams.iter().filter(|s| s.codec_type == stream_type).nth(index)
	}

	fn get_typed_stream_by_language<S: Into<String>>(&self, lang: S, stream_type: StreamType) -> Option<&Stream> {
		let lang = lang.into();
		self.streams.iter().find(|s| {
			s.codec_type == stream_type &&
				s.tags.as_ref()
					.and_then(|t| t.language.as_ref())
					.map(|l| l == &lang)
					.is_some_and(|x| x)
		})
	}

	pub fn get_video_stream(&self, index: usize) -> Option<&Stream> {
		self.get_typed_stream(index, StreamType::Video)
	}

	pub fn get_audio_stream(&self, index: usize) -> Option<&Stream> {
		self.get_typed_stream(index, StreamType::Audio)
	}

	pub fn get_subtitle_stream(&self, index: usize) -> Option<&Stream> {
		self.get_typed_stream(index, StreamType::Subtitle)
	}

	pub fn get_video_stream_by_language<S: Into<String>>(&self, lang: S) -> Option<&Stream> {
		self.get_typed_stream_by_language(lang, StreamType::Video)
	}

	pub fn checked_get_video_stream_by_index_or_language(&self, lang: &Option<String>, index: usize) -> Result<(Stream, String)> {
		let (video_stream, video_stream_id) = match lang {
			Some(language) => {
				let stream = self
					.get_video_stream_by_language(language)
					.context(format!("No stream with language \"{language}\" found"))?
					.clone();
				(stream, format!("0:V:m:language:{language}"))
			}
			None => {
				let stream = self
					.get_video_stream(index)
					.context(format!("No stream with index {index} found"))?
					.clone();
				(stream, format!("0:V:{index}"))
			}
		};

		match video_stream.height {
			None => anyhow::bail!("The selected video stream contains no height information"),
			Some(0) => anyhow::bail!("The selected video stream contains invalid height information"),
			_ => (),
		}

		Ok((video_stream, video_stream_id))
	}

	pub fn get_audio_stream_by_language<S: Into<String>>(&self, lang: S) -> Option<&Stream> {
		self.get_typed_stream_by_language(lang, StreamType::Audio)
	}

	pub fn get_subtitle_stream_by_language<S: Into<String>>(&self, lang: S) -> Option<&Stream> {
		self.get_typed_stream_by_language(lang, StreamType::Subtitle)
	}

	pub fn get_first_video_stream(&self) -> Option<&Stream> {
		self.streams.iter().find(|s| s.codec_type == StreamType::Video)
	}

	pub fn get_first_audio_stream(&self) -> Option<&Stream> {
		self.streams.iter().find(|s| s.codec_type == StreamType::Audio)
	}

	pub fn get_first_subtitle_stream(&self) -> Option<&Stream> {
		self.streams.iter().find(|s| s.codec_type == StreamType::Subtitle)
	}

	pub fn has_video_streams(&self) -> bool {
		self.get_first_video_stream().is_some()
	}

	pub fn has_audio_streams(&self) -> bool {
		self.get_first_audio_stream().is_some()
	}

	pub fn has_subtitle_streams(&self) -> bool {
		self.get_first_subtitle_stream().is_some()
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

impl Display for StreamType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			StreamType::Audio => write!(f, "Audio"),
			StreamType::Video => write!(f, "Video"),
			StreamType::Subtitle => write!(f, "Subtitle"),
			StreamType::Data => write!(f, "Data"),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Tags {
	#[serde(rename = "DURATION")]
	pub duration: Option<String>,
	pub language: Option<String>,
	pub title: Option<String>,
	pub handler_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Disposition {
	pub default: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Format {
	pub duration: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Stream {
	pub index: u64,
	pub codec_name: Option<String>,
	pub profile: Option<String>,
	pub codec_type: StreamType,
	pub codec_tag_string: Option<String>,
	pub width: Option<u64>,
	pub height: Option<u64>,
	#[serde(rename = "sample_aspect_ratio")]
	pub sar: Option<String>,
	#[serde(rename = "display_aspect_ratio")]
	pub dar: Option<String>,
	pub pix_fmt: Option<String>,
	pub field_order: Option<String>,
	pub color_range: Option<String>,
	pub color_space: Option<String>,
	pub color_transfer: Option<String>,
	pub color_primaries: Option<String>,
	pub r_frame_rate: Option<String>,
	pub avg_frame_rate: Option<String>,
	pub sample_fmt: Option<String>,
	pub sample_rate: Option<String>,
	pub channels: Option<u64>,
	pub channel_layout: Option<String>,
	pub bits_per_raw_sample: Option<String>,
	pub bit_rate: Option<String>,
	pub duration: Option<String>,
	pub nb_frames: Option<String>,
	pub nb_read_frames: Option<String>,
	pub tags: Option<Tags>,
	pub disposition: Option<Disposition>,
}

impl Stream {
	pub fn frame_rate(&self) -> Option<f64> {
		match &self.r_frame_rate {
			None => {
				return None;
			}
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

	pub fn is_hdr(&self) -> bool {
		if let Some(color_transfer) = &self.color_transfer {
			return color_transfer.contains("smpte2084") || color_transfer.contains("arib-std-b67");
		}

		false
	}

	pub fn total_frames(&self) -> Option<u64> {
		if let Some(nb_read_frames) = &self.nb_read_frames {
			return nb_read_frames.parse().ok();
		} else if let Some(nb_frames) = &self.nb_frames {
			return nb_frames.parse().ok();
		}

		None
	}
}
