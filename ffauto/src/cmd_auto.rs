use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use crate::commands::AutoArgs;
use crate::common::*;
use crate::vec_push_ext::PushStrExt;
use anyhow::{Context, Result};
use clap::ArgMatches;
use ffmpeg::ffmpeg::enums::{OptimizeTarget, VideoCodec};
use ffmpeg::ffmpeg::ffmpeg::ffmpeg;
use ffmpeg::ffmpeg::ffmpeg_cropdetect::ffmpeg_cropdetect;
use ffmpeg::ffmpeg::ffprobe_struct::{StreamType, Tags, normalize_language_code};
use ffmpeg::filters::{Afade, Aformat, Crop, Fade, FilterChain, Fps, Subtitles, Volume};
use isolang::Language;

#[derive(PartialEq)]
enum StreamIndex {
	Index(usize),
	Language(Language),
}

/// Splits a file path passed via `--sub-streams` into its path and an optional language code.
/// Subtitle paths can be passed as either:
/// * `subs.srt`
/// * `subs.en.srt`
/// * `subs.srt:eng`
fn split_subtitle_path_and_language(entry: &str) -> (&Path, Option<&str>) {
	// only the last path segment may carry the `:lang` suffix. colons earlier in the path
	// (windows drive letters, volume names, etc) are part of the path itself.
	let segment_start = entry.rfind(['/', '\\']).map_or(0, |i| i + 1);

	if let Some(colon) = entry[segment_start..].find(':') {
		let split_at = segment_start + colon;
		let lang = &entry[split_at + 1..];
		return (Path::new(&entry[..split_at]), (!lang.is_empty()).then_some(lang));
	}

	let path = Path::new(entry);

	// `subs.eng.srt` -> the file stem is `subs.eng`, whose "extension" is the language
	let lang = path
		.file_stem()
		.map(Path::new)
		.and_then(Path::extension)
		.and_then(|ext| ext.to_str());

	(path, lang)
}

#[allow(clippy::too_many_lines)]
pub(crate) fn ffmpeg_auto(args: &AutoArgs, matches: &ArgMatches, debug: bool) -> Result<()> {
	let probe = ffprobe_output(&args.input)?;

	if !probe.has_video_streams() && !probe.has_audio_streams() {
		anyhow::bail!("The input file contains no usable audio/video streams")
	}

	let first_video_stream = probe.get_first_video_stream();
	let first_audio_stream = probe.get_first_audio_stream();

	let video_stream = first_video_stream.context("The input file needs to contain a usable video stream")?;

	let video_duration = probe.duration()?;

	// we'll maintain two lists of input and output options so we can properly add subtitle files
	let mut input_args: Vec<String> = Vec::new();
	let mut ffmpeg_args: Vec<String> = Vec::new();

	// counts added subtitle files so stream indices stay accurate
	let mut next_input_idx: usize = 0;

	let seek = args.parse_seek();

	if let Some(seek) = seek {
		let seek = format!("{}", seek.as_secs_f64());
		if args.burn_subtitle {
			// we have to use slow seeking when burning subtitles, so this goes in the output args
			ffmpeg_args.add_two("-ss", seek);
		} else {
			input_args.add_two("-ss", seek);
		}
	}

	let input = args.input.as_os_str().to_str().unwrap();
	input_args.add_two("-i", input);

	let duration = args.parse_duration();
	if let Some(duration) = duration {
		ffmpeg_args.add_two("-t", format!("{}", duration.as_secs_f64()));
	}

	ffmpeg_args.add_two("-disposition", "0");
	ffmpeg_args.add_two("-metadata:s", "handler_name=");
	ffmpeg_args.add_two("-empty_hdlr_name", "1");

	if args.faststart {
		ffmpeg_args.add_two("-movflags", "faststart");
	}

	// region Stream Selection

	let selected_streams_and_types = [
		(&args.video_streams, StreamType::Video),
		(&args.audio_streams, StreamType::Audio),
		(&args.sub_streams, StreamType::Subtitle),
	];

	// -metadata expects output stream indices, so keep track of those
	let mut output_stream_idx: usize = 0;

	// the index intended for use with --burn-subtitles
	let mut burn_subtitle_idx: Option<usize> = None;

	// select appropriate streams, default to the first one respectively if none were specified
	for (selected_streams, stream_type) in selected_streams_and_types {
		match stream_type {
			StreamType::Video if !probe.has_video_streams() => {
				continue;
			}
			StreamType::Audio if !probe.has_audio_streams() => {
				continue;
			}
			_ => (),
		}

		let mut used_indices: Vec<StreamIndex> = vec![];
		for stream in selected_streams {
			let stream = stream.trim();
			if let Ok(i) = stream.parse::<usize>() {
				// value is a numeric stream ID
				let used_idx = StreamIndex::Index(i);
				if used_indices.contains(&used_idx) {
					continue;
				}

				ffmpeg_args.add_two("-map", format!("0:{}:{i}", stream_type.identifier()));
				if let Some(selected_stream) = match stream_type {
					StreamType::Video => probe.get_video_stream_by_index(i),
					StreamType::Audio => probe.get_audio_stream_by_index(i),
					StreamType::Subtitle => probe.get_subtitle_stream_by_index(i),
					_ => panic!("you shouldn't be here"),
				} {
					if stream_type == StreamType::Subtitle
						&& args.burn_subtitle
						&& burn_subtitle_idx.is_none()
					{
						burn_subtitle_idx = Some(selected_stream.typed_index);
					}

					if let Some(Tags { language: Some(lang), .. }) = &selected_stream.tags {
						let lang = normalize_language_code(lang);
						ffmpeg_args.add_two(
							format!("-metadata:s:{output_stream_idx}"),
							format!("language={lang}"),
						);
					}
				}

				used_indices.push(used_idx);
			} else if let Ok(lang) = Language::from_str(stream) {
				// value is a valid language code
				let used_lang = StreamIndex::Language(lang);
				if used_indices.contains(&used_lang) {
					continue;
				}

				let selected_stream = match stream_type {
					StreamType::Video => probe.get_video_stream_by_language(stream),
					StreamType::Audio => probe.get_audio_stream_by_language(stream),
					StreamType::Subtitle => probe.get_subtitle_stream_by_language(stream),
					_ => None,
				};

				if stream_type == StreamType::Subtitle
					&& args.burn_subtitle
					&& burn_subtitle_idx.is_none()
					&& let Some(selected_stream) = selected_stream
				{
					burn_subtitle_idx = Some(selected_stream.typed_index);
				}

				// grab the selected stream's language tag because the
				// supplied language might not work for ffmpeg
				let tag = selected_stream
					.and_then(|s| s.tags.as_ref())
					.and_then(|t| t.language.as_deref())
					.unwrap_or(stream);

				ffmpeg_args.add_two("-map", format!("0:{}:m:language:{tag}", stream_type.identifier()));

				let lang = normalize_language_code(stream);
				ffmpeg_args.add_two(
					format!("-metadata:s:{output_stream_idx}"),
					format!("language={lang}"),
				);

				used_indices.push(used_lang);
			} else if stream_type == StreamType::Subtitle {
				// value is a path to an external subtitle file
				let (path, lang) = split_subtitle_path_and_language(stream);

				match path.canonicalize() {
					Ok(canon) => {
						next_input_idx += 1;
						input_args.add_two("-i", canon.into_os_string().into_string().unwrap());
						ffmpeg_args.add_two("-map", format!("{next_input_idx}:s:0"));

						// files without a language part are still usable,
						// they just don't get any language metadata.
						if let Some(lang) = lang {
							ffmpeg_args.add_two(
								format!("-metadata:s:{output_stream_idx}"),
								format!("language={}", normalize_language_code(lang)),
							);
						}
					}
					_ => continue,
				}
			} else {
				// value is neither a stream index, a language code, nor (for subtitles) a file path.
				// skip this one before output_stream_idx is incremented.
				continue;
			}

			output_stream_idx += 1;
		}
	}

	// subtitle fixup
	if args.burn_subtitle {
		// if we're burning subtitles into a video, add -sn so no subtitle *streams* make it into the output file.
		// this is because we have to use slow seeking during subtitle burning,
		// and ffmpeg only rebases video and audio timestamps but *not* subtitle ones.
		// tl;dr without -sn the output file would have desynced stream timestamps,
		// causing playback problems in several players.
		ffmpeg_args.add("-sn");
	} else if args.sub_streams.is_empty() {
		if probe
			.streams
			.iter()
			.any(|s| s.codec_type == StreamType::Subtitle && s.codec_name != Some("hdmv_pgs_subtitle".into()))
		{
			// there are subtitles that are not of type hdmv_pgs_subtitle, so we can actually use this
			// TODO: this might fail for files that have both usable subtitles and hdmv_pgs_subtitle subtitles
			ffmpeg_args.add_two("-map", "0:s?");
		} else {
			// there are only hdmv_pgs_subtitle subtitles, ignore them
			ffmpeg_args.add("-sn");
		}
	}

	// this tool is currently mostly designed to output mp4 files and those only support mov_text subtitles
	if probe.has_subtitle_streams() && !args.burn_subtitle {
		ffmpeg_args.add_two("-c:s", "mov_text");
	}

	// if --burn-subtitle was specified but no subtitle was selected or supplied, print an error and exit
	if args.burn_subtitle && args.burn_subtitle_file.is_none() && burn_subtitle_idx.is_none() {
		anyhow::bail!(
			"--burn-subtitle needs a subtitle stream selected with --sub-streams/--Ss, or a subtitle file given with --subtitle-file/--Bf"
		)
	}

	// endregion

	let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
	if args.fade > 0.0 {
		fade_in = args.fade;
		fade_out = args.fade;
	}
	let fade_out_start = if let Some(duration) = duration {
		// duration was given
		duration.as_secs_f64() - fade_out
	} else {
		// duration wasn't given, use video duration
		video_duration
			.saturating_sub(seek.unwrap_or(Duration::ZERO))
			.as_secs_f64()
			- fade_out
	};

	// region Audio Filtering

	if first_audio_stream.is_none() || args.mute {
		// input has no audio streams or explicit mute was requested
		ffmpeg_args.add("-an");
	} else if let Some(audio_stream) = first_audio_stream.cloned() {
		if args.audio_copy_possible(audio_stream.codec_name.as_deref()) {
			// input stream is already aac, copy stream
			ffmpeg_args.add_two("-c:a", "copy");
		} else {
			// input stream is not aac or transcoding is needed
			ffmpeg_args.add_two("-c:a", args.video_codec.audio_codec());

			match args.optimize_target {
				Some(OptimizeTarget::Ipod) => {
					ffmpeg_args.add_two("-b:a", "160k");
				}
				_ => {
					ffmpeg_args.add_two("-b:a", "256k");
				}
			}

			if let Some(audio_channels) = &args.audio_channels {
				ffmpeg_args.add_two("-ac", audio_channels.clone());
			}

			let mut audio_filters = FilterChain::new();

			// this tool is designed to always output aac audio, but aac does not support all possible channel layouts.
			// this aformat filter is here to constrain input channel layouts to something aac can represent.
			audio_filters.push(Aformat::aac_channel_layouts());

			#[allow(clippy::float_cmp)]
			if args.audio_volume != 1.0 {
				audio_filters.push(Volume::new(args.audio_volume));
			}

			if fade_in > 0.0 {
				audio_filters.push(Afade::r#in(0.0, fade_in));
			}
			if fade_out > 0.0 {
				audio_filters.push(Afade::out(fade_out_start, fade_out));
			}

			if !audio_filters.is_empty() {
				ffmpeg_args.add_two("-af", audio_filters.to_string());
			}
		}
	}

	// endregion

	// region Video Filtering

	ffmpeg_args.add_two("-c:v", args.video_codec.video_codec());
	ffmpeg_args.add_two(
		"-crf",
		args.video_codec.crf_with_garbage(args.garbage).to_string(),
	);
	ffmpeg_args.add_two("-pix_fmt", args.video_codec.pix_fmt());
	ffmpeg_args.add_two("-preset", args.preset.to_string());
	ffmpeg_args.add("-tune");
	match args.video_codec {
		VideoCodec::H264 => {
			ffmpeg_args.add("film");
		}
		VideoCodec::H265 | VideoCodec::H265_10 => {
			ffmpeg_args.add("grain");
			ffmpeg_args.add("-tag:v");
			ffmpeg_args.add("hvc1");
		}
	}

	// add extra ffmpeg arguments that aren't handled by optimize_settings()
	// TODO: test this on actual target devices
	match args.optimize_target {
		None => (),
		Some(OptimizeTarget::Ipod5) => {
			ffmpeg_args.add_two("-profile:v", "baseline"); // apple: baseline
			ffmpeg_args.add_two("-level", "1.3"); // apple: 1.3
			ffmpeg_args.add_two("-maxrate", "768K"); // apple: 768 kbps, actual level limit
			ffmpeg_args.add_two("-bufsize", "2M");
			ffmpeg_args.add("-sn"); // the 5th gen iPod does not support subtitles
			ffmpeg_args.add_two("-map_chapters", "0"); // it does however support video chapters
		}
		Some(OptimizeTarget::Ipod) => {
			ffmpeg_args.add_two("-profile:v", "baseline"); // apple: baseline
			ffmpeg_args.add_two("-level", "3.0"); // apple: 3.0
			ffmpeg_args.add_two("-maxrate", "2.5M"); // apple: 2.5 mbps
			ffmpeg_args.add_two("-bufsize", "5M");
			ffmpeg_args.add_two("-c:s", "mov_text");
			ffmpeg_args.add_two("-tag:s", "tx3g");
			ffmpeg_args.add_two("-map_chapters", "0");
		}
		Some(OptimizeTarget::Psp) => {
			ffmpeg_args.add_two("-profile:v", "main");
			ffmpeg_args.add_two("-level", "3.0");
			ffmpeg_args.add_two("-maxrate", "3M"); // needs verification
			ffmpeg_args.add_two("-bufsize", "6M");
		}
		Some(OptimizeTarget::PsVita) => {
			// H.264/MPEG-4 AVC Hi/Main/Baseline Profile (AAC)
			ffmpeg_args.add_two("-profile:v", "high");
			ffmpeg_args.add_two("-level", "4.1");
			ffmpeg_args.add_two("-maxrate", "10M");
			ffmpeg_args.add_two("-bufsize", "20M");
		}
	}

	if args.needs_video_filter() {
		let mut video_filters = FilterChain::new();

		// the framerate everything downstream should assume. changed later by the framerate multiplier.
		let mut effective_fps = video_stream.frame_rate();

		// --framerate and --framerate-mult are mutually exclusive via clap's "framerates" group
		if let Some(framerate) = args.framerate {
			if framerate <= 0.0 {
				anyhow::bail!("The frame rate must be greater than zero, got {framerate}")
			}

			video_filters.push(Fps::new(framerate));
			effective_fps = Some(framerate);
		} else if let Some(framerate_mult) = args.framerate_mult {
			if framerate_mult <= 0.0 {
				anyhow::bail!("The frame rate multiplier must be greater than zero, got {framerate_mult}")
			}

			let input_fps = video_stream
				.frame_rate()
				.context("The input video stream contains no frame rate information")?;

			let target_fps = input_fps * framerate_mult;

			video_filters.push(Fps::new(target_fps));
			effective_fps = Some(target_fps);
		}

		#[allow(clippy::single_match)]
		match args.optimize_target {
			Some(OptimizeTarget::Ipod | OptimizeTarget::Ipod5) => {
				// cap framerate at 30
				if let Some(fps) = effective_fps
					&& fps > 30.0
				{
					video_filters.push(Fps::target(fps, 30.0));
				}
			}
			_ => (),
		}

		let remove_bars_crop: Option<Crop> = if args.remove_bars {
			eprintln!("Gathering autocrop information…");
			Some(ffmpeg_cropdetect(&args.input)?)
		} else {
			None
		};

		if let Some(remove_bars_crop) = remove_bars_crop {
			video_filters.push(remove_bars_crop);
		}

		let mut crop_and_scale = FilterChain::new();

		if let Some(Ok(crop)) = args.crop.clone().map(Crop::from_arg) {
			crop_and_scale.push(crop);
		}

		if let Some(scale_filter) = generate_scale_filter(
			args.width,
			args.height,
			args.size_fit.as_ref(),
			args.size_fill.as_ref(),
			args.scale_mode,
			args.target_video_range.as_ref(),
		) {
			crop_and_scale.push(scale_filter);
		}

		// get order of crop and scale arguments so we can reorder the crop and scale filters below
		let (crop_index, scale_index) = get_crop_and_scale_order(matches);

		if scale_index < crop_index {
			// if the scale argument was provided before the crop argument, flip this list around
			crop_and_scale.reverse();
		}
		video_filters.extend(crop_and_scale);

		if (args.tonemap || args.video_codec != VideoCodec::H265_10) && video_stream.is_hdr() {
			let tonemap_chain = sdr_tonemap_chain();
			video_filters.extend(tonemap_chain);
		}

		if args.burn_subtitle
			&& let Some(subtitles) = args
				.burn_subtitle_file
				.as_ref()
				.map(Subtitles::new_with_file)
				.or_else(|| {
					burn_subtitle_idx
						.as_ref()
						.map(|i| Subtitles::new_with_file_and_stream_index(&args.input, *i))
				}) {
			// macOS comes with Helvetica Neue. other platforms will presumably default to Arial.
			#[cfg(target_os = "macos")]
			let subtitles = {
				let mut subtitles = subtitles;
				subtitles.set_font("Helvetica Neue");
				subtitles
			};

			video_filters.push(subtitles);
		}

		if fade_in > 0.0 {
			video_filters.push(Fade::r#in(0.0, fade_in));
		}
		if fade_out > 0.0 {
			video_filters.push(Fade::out(fade_out_start, fade_out));
		}

		if !video_filters.is_empty() {
			ffmpeg_args.add_two("-vf", video_filters.to_string());
		}
	}

	// endregion

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	// all inputs first, then everything that applies to the output
	let ffmpeg_args = [input_args, ffmpeg_args].concat();

	ffmpeg(
		&ffmpeg_args,
		args.hwaccel.then(|| args.accelerator.clone()),
		true,
		debug,
	)
}

#[cfg(test)]
mod tests {
	use super::split_subtitle_path_and_language;

	fn split(entry: &str) -> (String, Option<&str>) {
		let (path, lang) = split_subtitle_path_and_language(entry);
		(path.to_string_lossy().into_owned(), lang)
	}

	#[test]
	fn parse_explicit_and_implicit_languages() {
		// explicit language
		assert_eq!(split("subs.srt:eng"), ("subs.srt".into(), Some("eng")));

		// implicit language
		assert_eq!(split("subs.eng.srt"), ("subs.eng.srt".into(), Some("eng")));

		// a full path with implicit language
		assert_eq!(
			split("/tmp/Movie.en-US.ass"),
			("/tmp/Movie.en-US.ass".into(), Some("en-US"))
		);

		// an explicit language wins over an implicit one
		assert_eq!(split("subs.deu.srt:eng"), ("subs.deu.srt".into(), Some("eng")));

		// a trailing colon with nothing after it is not a language
		assert_eq!(split("subs.srt:"), ("subs.srt".into(), None));

		// a trailing colon with nothing after it parses as a path with no language, despite the implicit one in the name
		assert_eq!(split("subs.deu.srt:"), ("subs.deu.srt".into(), None));

		// nonsense languages are passed through with no normalization or filtering
		assert_eq!(split("subs.srt:bimp"), ("subs.srt".into(), Some("bimp")));
	}

	#[test]
	fn colons_outside_the_last_path_segment_are_part_of_the_path() {
		assert_eq!(
			split("/Volumes/My:Disk/subs.srt"),
			("/Volumes/My:Disk/subs.srt".into(), None)
		);
		assert_eq!(
			split("/Volumes/My:Disk/subs.srt:eng"),
			("/Volumes/My:Disk/subs.srt".into(), Some("eng"))
		);
		assert_eq!(
			split(r"C:\subs\subs.eng.srt"),
			(r"C:\subs\subs.eng.srt".into(), Some("eng"))
		);
	}
}
