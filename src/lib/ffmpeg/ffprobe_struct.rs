use crate::ffmpeg::deserialize_bool_from_int;
use crate::ffmpeg::timestamps::parse_ffmpeg_duration;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use colored::Color;

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
		self.streams
			.iter()
			.find(|s| s.codec_type == stream_type && s.tags.as_ref().and_then(|t| t.language.as_ref()).map(|l| l == &lang).is_some_and(|x| x))
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
				let stream = self.get_video_stream(index).context(format!("No stream with index {index} found"))?.clone();
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
	Attachment,
}

impl StreamType {
	pub fn identifier(&self) -> &str {
		match self {
			StreamType::Audio => "a",
			StreamType::Video => "V",
			StreamType::Subtitle => "s",
			StreamType::Data => "d",
			StreamType::Attachment => "t",
		}
	}

	pub fn color(&self) -> Color {
		match self {
			StreamType::Video => Color::Blue,
			StreamType::Audio => Color::Red,
			StreamType::Subtitle => Color::Magenta,
			StreamType::Data => Color::Green,
			StreamType::Attachment => Color::Yellow,
		}
	}
}

impl Display for StreamType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			StreamType::Audio => write!(f, "Audio"),
			StreamType::Video => write!(f, "Video"),
			StreamType::Subtitle => write!(f, "Subtitle"),
			StreamType::Data => write!(f, "Data"),
			StreamType::Attachment => write!(f, "Attachment"),
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
	pub filename: Option<String>,
	pub mimetype: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Disposition {
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub default: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub dub: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub original: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub comment: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub lyrics: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub karaoke: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub forced: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub hearing_impaired: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub visual_impaired: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub clean_effects: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub attached_pic: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub timed_thumbnails: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub non_diegetic: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub captions: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub descriptions: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub metadata: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub dependent: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub still_image: bool,
	#[serde(deserialize_with = "deserialize_bool_from_int")]
	pub multilayer: bool,
}

impl Disposition {
	pub fn any_true(&self) -> bool {
		if self.default { return true }
		if self.dub { return true }
		if self.original { return true }
		if self.comment { return true }
		if self.lyrics { return true }
		if self.karaoke { return true }
		if self.forced { return true }
		if self.hearing_impaired { return true }
		if self.visual_impaired { return true }
		if self.clean_effects { return true }
		if self.attached_pic { return true }
		if self.timed_thumbnails { return true }
		if self.non_diegetic { return true }
		if self.captions { return true }
		if self.descriptions { return true }
		if self.metadata { return true }
		if self.dependent { return true }
		if self.still_image { return true }
		if self.multilayer { return true }

		false
	}
}

impl Display for Disposition {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut disposition: Vec<String> = Vec::with_capacity(19);

		if self.default { disposition.push("default".into()) }
		if self.dub { disposition.push("dub".into()) }
		if self.original { disposition.push("original".into()) }
		if self.comment { disposition.push("comment".into()) }
		if self.lyrics { disposition.push("lyrics".into()) }
		if self.karaoke { disposition.push("karaoke".into()) }
		if self.forced { disposition.push("forced".into()) }
		if self.hearing_impaired { disposition.push("hearing impaired".into()) }
		if self.visual_impaired { disposition.push("visual impaired".into()) }
		if self.clean_effects { disposition.push("clean effects".into()) }
		if self.attached_pic { disposition.push("attached pic".into()) }
		if self.timed_thumbnails { disposition.push("timed thumbnails".into()) }
		if self.non_diegetic { disposition.push("non diegetic".into()) }
		if self.captions { disposition.push("captions".into()) }
		if self.descriptions { disposition.push("descriptions".into()) }
		if self.metadata { disposition.push("metadata".into()) }
		if self.dependent { disposition.push("dependent".into()) }
		if self.still_image { disposition.push("still image".into()) }
		if self.multilayer { disposition.push("multilayer".into()) }

		if disposition.is_empty() {
			write!(f, "")
		} else {
			let disposition_str = disposition.join(", ");
			write!(f, "{disposition_str}")
		}
	}
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
