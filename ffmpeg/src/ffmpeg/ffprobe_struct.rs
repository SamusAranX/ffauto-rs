use crate::ffmpeg::deserialize_bool_from_int;
use crate::ffmpeg::timestamps::parse_ffmpeg_duration;
use anyhow::{Context, Result, anyhow};
use colored::Color;
use isolang::Language;
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;

/// Normalizes a language code to ISO 639-2/B, the form ffmpeg writes into `language` tags.
/// Accepts 639-1 (`en`), 639-2/T (`deu`), 639-3 and locales (`en-US`). Anything that can't
/// be recognized is returned unchanged.
#[must_use]
pub fn normalize_language_code(code: &str) -> &str {
	Language::from_str(code)
		.ok()
		.or_else(|| Language::from_locale(&code.replace('-', "_")))
		.map_or(code, |l| l.to_639_2b())
}

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
				stream_duration
					.parse()
					.map_err(|e| anyhow!("{e}: stream duration \"{stream_duration}\""))?,
			));
		}

		if let Some(tags_duration) = video_stream
			.tags
			.clone()
			.and_then(|t| t.duration)
			.and_then(|s| parse_ffmpeg_duration(&s))
		{
			// return first video stream tags duration
			return Ok(tags_duration);
		}

		if let Some(format_duration) = &self.format.duration {
			// return format duration
			return Ok(Duration::from_secs_f64(format_duration.parse()?));
		}

		if let (Some(read_frames), Some(frame_rate)) =
			(&video_stream.nb_read_frames, video_stream.frame_rate())
		{
			// divide number of frames by frame rate and return the result

			let read_frames = read_frames.parse::<f64>()?;
			return Ok(Duration::from_secs_f64(read_frames / frame_rate));
		}

		anyhow::bail!("ffprobe could not find a duration for the input file")
	}

	#[must_use]
	pub fn get_stream_by_index(&self, index: usize) -> Option<&Stream> {
		self.streams.get(index)
	}

	fn get_typed_streams(&self, stream_type: &StreamType) -> impl Iterator<Item = &Stream> {
		self.streams.iter().filter(|s| s.codec_type == *stream_type)
	}

	#[must_use]
	fn get_typed_stream_by_index(&self, stream_type: &StreamType, index: usize) -> Option<&Stream> {
		self.get_typed_streams(stream_type).nth(index)
	}

	#[must_use]
	fn get_typed_stream_by_language<S: Into<String>>(
		&self,
		lang: S,
		stream_type: &StreamType,
	) -> Option<&Stream> {
		// both sides of the check are normalized, so "en", "eng" and "en-US" all match a stream tagged "eng"
		let lang = lang.into();
		let lang = normalize_language_code(&lang);
		self.get_typed_streams(stream_type).find(|s| {
			s.tags
				.as_ref()
				.and_then(|t| t.language.as_deref())
				.is_some_and(|l| normalize_language_code(l) == lang)
		})
	}

	#[must_use]
	pub fn get_video_stream_by_index(&self, index: usize) -> Option<&Stream> {
		self.get_typed_stream_by_index(&StreamType::Video, index)
	}

	#[must_use]
	pub fn get_audio_stream_by_index(&self, index: usize) -> Option<&Stream> {
		self.get_typed_stream_by_index(&StreamType::Audio, index)
	}

	#[must_use]
	pub fn get_subtitle_stream_by_index(&self, index: usize) -> Option<&Stream> {
		self.get_typed_stream_by_index(&StreamType::Subtitle, index)
	}

	#[must_use]
	pub fn get_video_stream_by_language<S: Into<String>>(&self, lang: S) -> Option<&Stream> {
		self.get_typed_stream_by_language(lang, &StreamType::Video)
	}

	#[must_use]
	pub fn get_audio_stream_by_language<S: Into<String>>(&self, lang: S) -> Option<&Stream> {
		self.get_typed_stream_by_language(lang, &StreamType::Audio)
	}

	#[must_use]
	pub fn get_subtitle_stream_by_language<S: Into<String>>(&self, lang: S) -> Option<&Stream> {
		self.get_typed_stream_by_language(lang, &StreamType::Subtitle)
	}

	pub fn checked_get_video_stream_by_index_or_language(
		&self,
		lang: &Option<String>,
		index: usize,
	) -> Result<(Stream, String)> {
		let (video_stream, video_stream_id) = if let Some(language) = lang {
			let stream = self
				.get_video_stream_by_language(language)
				.context(format!("No stream with language \"{language}\" found"))?
				.clone();
			// get the stream's actual language tag to build the m:language:{tag} selector
			let stream_id = {
				let tag = stream
					.tags
					.as_ref()
					.and_then(|t| t.language.as_deref())
					.unwrap_or(language);
				format!("0:V:m:language:{tag}")
			};
			(stream, stream_id)
		} else {
			let stream = self
				.get_video_stream_by_index(index)
				.context(format!("No stream with index {index} found"))?
				.clone();
			(stream, format!("0:V:{index}"))
		};

		match video_stream.height {
			None => anyhow::bail!("The selected video stream contains no height information"),
			Some(0) => anyhow::bail!("The selected video stream contains invalid height information"),
			_ => (),
		}

		Ok((video_stream, video_stream_id))
	}

	#[must_use]
	pub fn get_first_video_stream(&self) -> Option<&Stream> {
		self.get_typed_streams(&StreamType::Video).next()
	}

	#[must_use]
	pub fn get_first_audio_stream(&self) -> Option<&Stream> {
		self.get_typed_streams(&StreamType::Audio).next()
	}

	#[must_use]
	pub fn get_first_subtitle_stream(&self) -> Option<&Stream> {
		self.get_typed_streams(&StreamType::Subtitle).next()
	}

	#[must_use]
	pub fn has_video_streams(&self) -> bool {
		self.get_first_video_stream().is_some()
	}

	#[must_use]
	pub fn has_audio_streams(&self) -> bool {
		self.get_first_audio_stream().is_some()
	}

	#[must_use]
	pub fn has_subtitle_streams(&self) -> bool {
		self.get_first_subtitle_stream().is_some()
	}
}

#[derive(clap::ValueEnum, Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
	Audio,
	Video,
	Subtitle,
	Data,
	Attachment,
}

impl StreamType {
	#[must_use]
	pub fn identifier(&self) -> &str {
		match self {
			StreamType::Audio => "a",
			StreamType::Video => "V",
			StreamType::Subtitle => "s",
			StreamType::Data => "d",
			StreamType::Attachment => "t",
		}
	}

	#[must_use]
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
	/// Every flag paired with its human readable name, in ffprobe's field order.
	/// Single source of truth for [`Disposition::any_true`] and the [`Display`] impl.
	fn flags(&self) -> [(bool, &'static str); 19] {
		[
			(self.default, "default"),
			(self.dub, "dub"),
			(self.original, "original"),
			(self.comment, "comment"),
			(self.lyrics, "lyrics"),
			(self.karaoke, "karaoke"),
			(self.forced, "forced"),
			(self.hearing_impaired, "hearing impaired"),
			(self.visual_impaired, "visual impaired"),
			(self.clean_effects, "clean effects"),
			(self.attached_pic, "attached pic"),
			(self.timed_thumbnails, "timed thumbnails"),
			(self.non_diegetic, "non diegetic"),
			(self.captions, "captions"),
			(self.descriptions, "descriptions"),
			(self.metadata, "metadata"),
			(self.dependent, "dependent"),
			(self.still_image, "still image"),
			(self.multilayer, "multilayer"),
		]
	}

	#[must_use]
	pub fn any_true(&self) -> bool {
		self.flags().into_iter().any(|(set, _)| set)
	}
}

impl Display for Disposition {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut first = true;
		for (set, name) in self.flags() {
			if !set {
				continue;
			}
			if !first {
				f.write_str(", ")?;
			}
			f.write_str(name)?;
			first = false;
		}
		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Format {
	pub duration: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Stream {
	pub index: usize,
	#[serde(default)]
	pub typed_index: usize, // filled in as part of ffprobe()
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
	#[must_use]
	pub fn frame_rate(&self) -> Option<f64> {
		match &self.r_frame_rate {
			None => {
				return None;
			}
			Some(fps) => {
				if fps.contains('/') {
					if let Some(split) = fps.split_once('/') {
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

	#[must_use]
	pub fn is_hdr(&self) -> bool {
		if let Some(color_transfer) = &self.color_transfer {
			return color_transfer.contains("smpte2084") || color_transfer.contains("arib-std-b67");
		}

		false
	}

	#[must_use]
	pub fn total_frames(&self) -> Option<u64> {
		if let Some(nb_read_frames) = &self.nb_read_frames {
			return nb_read_frames.parse().ok();
		} else if let Some(nb_frames) = &self.nb_frames {
			return nb_frames.parse().ok();
		}

		None
	}
}

#[cfg(test)]
mod tests {
	use super::normalize_language_code;

	#[test]
	fn language_codes_normalize_to_639_2b() {
		// already 639-2/B, which is what ffmpeg wants
		assert_eq!(normalize_language_code("eng"), "eng");
		assert_eq!(normalize_language_code("ger"), "ger");

		// 639-1
		assert_eq!(normalize_language_code("en"), "eng");
		assert_eq!(normalize_language_code("de"), "ger");

		// 639-2/T, which differs from 639-2/B for some languages
		assert_eq!(normalize_language_code("deu"), "ger");

		// locales
		assert_eq!(normalize_language_code("en-US"), "eng");

		// unrecognized input is passed through untouched
		assert_eq!(normalize_language_code("qqq"), "qqq");
		assert_eq!(normalize_language_code(""), "");
	}
}
