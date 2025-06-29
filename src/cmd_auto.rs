use std::time::Duration;

use anyhow::Result;

use crate::commands::AutoArgs;
use crate::common::*;
use crate::vec_push_ext::PushStrExt;
use ffauto_rs::ffmpeg::enums::{OptimizeTarget, VideoCodec};
use ffauto_rs::ffmpeg::ffmpeg::ffmpeg;
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType::Subtitle;
use ffauto_rs::ffmpeg::ffprobe_struct::{Stream, Tags};

pub(crate) fn ffmpeg_auto(args: &AutoArgs, debug: bool) -> Result<()> {
	let probe = ffprobe_output(&args.input)?;

	let first_audio_stream = probe.get_first_audio_stream();
	let first_video_stream = probe.get_first_video_stream();

	if first_audio_stream.is_none() && first_video_stream.is_none() {
		anyhow::bail!("The input file contains no usable audio/video streams")
	}

	let video_stream = first_video_stream.expect("The input file needs to contain a usable video stream").clone();
	let video_duration = probe.duration()?;

	let mut ffmpeg_args: Vec<String> = vec![
		"-hide_banner".to_string(),
		"-loglevel".to_string(), "warning".to_string(),
		"-y".to_string(),
	];

	let seek = args.parse_seek();
	let duration = args.parse_duration();

	if let Some(seek) = seek {
		ffmpeg_args.add_two("-ss", format!("{}", seek.as_secs_f64()));
	}

	let input = args.input.as_os_str().to_str().unwrap();
	ffmpeg_args.add_two("-i", input);
	if let Some(duration) = duration {
		ffmpeg_args.add_two("-t", format!("{}", duration.as_secs_f64()));
	}

	ffmpeg_args.add_two("-disposition", "0");
	ffmpeg_args.add_two("-metadata:s", "handler_name=\"\"");
	ffmpeg_args.add_two("-empty_hdlr_name", "1");

	#[derive(PartialEq)]
	enum UsedIndex<'a> {
		Index(usize),
		Language(&'a String),
	}

	let all_streams = [&args.video_streams, &args.audio_streams, &args.sub_streams];
	let stream_types = ["V", "a", "s"];

	// -metadata expects output stream indices, so keep track of those
	let mut output_stream_idx = 0_usize;

	// select appropriate streams, default to the first one respectively if none were specified
	for (streams, stream_type) in all_streams.iter().zip(stream_types.iter()) {
		let mut used_indices: Vec<UsedIndex> = vec![];
		for stream in *streams {
			if let Ok(i) = stream.parse::<usize>() {
				let used_idx = UsedIndex::Index(i);
				if used_indices.contains(&used_idx) {
					continue;
				}

				ffmpeg_args.add_two("-map", format!("0:{stream_type}:{i}"));
				if let Some(Stream { tags: Some(Tags { language: Some(lang), .. }), .. }) = match *stream_type {
					"V" => probe.get_video_stream(i),
					"a" => probe.get_audio_stream(i),
					"s" => probe.get_subtitle_stream(i),
					_ => panic!("you shouldn't be here")
				} {
					let lang = iso639_lut(lang.clone());
					ffmpeg_args.add_two(format!("-metadata:s:{output_stream_idx}"), format!("language={lang}"));
				}

				used_indices.push(used_idx);
			} else if stream.trim().is_empty() {
				let used_lang = UsedIndex::Language(stream);
				if used_indices.contains(&used_lang) {
					continue;
				}

				ffmpeg_args.add_two("-map", format!("0:{stream_type}:m:language:{stream}"));

				let lang = iso639_lut(stream.clone());
				ffmpeg_args.add_two(format!("-metadata:s:{output_stream_idx}"), format!("language={lang}"));

				used_indices.push(used_lang);
			}

			output_stream_idx += 1;
		}
	}

	// subtitle fixup
	if args.sub_streams.is_empty() {
		if probe.streams.iter().any(|s| s.codec_type == Subtitle && s.codec_name != Some("hdmv_pgs_subtitle".into())) {
			// there are subtitles that are not of type hdmv_pgs_subtitle, so we can actually use this
			// TODO: this might fail for files that have both usable subtitles and hdmv_pgs_subtitle subtitles
			ffmpeg_args.add_two("-map", "0:s?");
		} else {
			// there are only hdmv_pgs_subtitle subtitles, so ignore them
			ffmpeg_args.add("-sn");
		}
	}

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
		(video_duration - seek.unwrap_or(Duration::ZERO)).as_secs_f64() - fade_out
	};

	// region Audio Filtering

	if first_audio_stream.is_none() || args.mute {
		// input has no audio streams or explicit mute was requested
		ffmpeg_args.add("-an");
	} else if let Some(audio_stream) = first_audio_stream.cloned() {
		if args.audio_copy_possible(audio_stream.codec_name) {
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
				ffmpeg_args.add_two("-ac", audio_channels.to_string());
			}

			if args.needs_audio_filter() {
				let mut audio_filter: Vec<String> = vec![];

				if args.audio_volume != 1.0 {
					audio_filter.push(format!("volume={:.3}", args.audio_volume));
				}

				if fade_in > 0.0 {
					audio_filter.push(format!("afade=t=in:st=0:d={fade_in:.3}:curve=losi"));
				}
				if fade_out > 0.0 {
					audio_filter.push(format!("afade=t=out:st={fade_out_start:.3}:d={fade_out:.3}:curve=losi"));
				}

				let audio_filter_str = audio_filter.join(",");
				ffmpeg_args.add_two("-af", audio_filter_str);
			}
		}
	}

	// endregion

	// region Video Filtering

	ffmpeg_args.add_two("-c:v", args.video_codec.video_codec());
	ffmpeg_args.add_two("-crf", format!("{}", &args.video_codec.crf_with_garbage(args.garbage)));
	ffmpeg_args.add_two("-pix_fmt", args.video_codec.pix_fmt());
	ffmpeg_args.add_two("-preset", "slower");
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

	ffmpeg_args.add_two("-partitions", "all");
	ffmpeg_args.add_two("-me_method", "tesa");

	// add extra ffmpeg arguments that aren't handled by optimize_settings()
	// TODO: test this on actual target devices
	match args.optimize_target {
		None => (),
		Some(OptimizeTarget::Ipod5) => {
			ffmpeg_args.add_two("-profile:v", "baseline"); // apple: baseline
			ffmpeg_args.add_two("-level", "1.3"); // apple: 1.3
			ffmpeg_args.add_two("-maxrate", "768K"); // apple: 768 kbps, actual level limit
			ffmpeg_args.add_two("-bufsize", "2M");
			ffmpeg_args.add_two("-c:s", "mov_text");
			ffmpeg_args.add_two("-tag:s", "tx3g");
		}
		Some(OptimizeTarget::Ipod) => {
			ffmpeg_args.add_two("-profile:v", "baseline"); // apple: baseline
			ffmpeg_args.add_two("-level", "3.0"); // apple: 3.0
			ffmpeg_args.add_two("-maxrate", "2.5M"); // apple: 2.5 mbps
			ffmpeg_args.add_two("-bufsize", "5M");
			ffmpeg_args.add_two("-c:s", "mov_text");
			ffmpeg_args.add_two("-tag:s", "tx3g");
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

	if args.faststart {
		ffmpeg_args.add_two("-movflags", "faststart");
	}

	if args.needs_video_filter() {
		let mut video_filter: Vec<String> = vec![];

		if let Some(fps_filter) = args.generate_fps_filter(video_stream.frame_rate()) {
			video_filter.push(fps_filter);
		}

		if let Some(crop_filter) = args.generate_crop_filter() {
			video_filter.push(crop_filter);
		}

		if let Some(scale_filter) = args.generate_scale_filter() {
			video_filter.push(scale_filter);
		}

		if (args.tonemap || args.video_codec != VideoCodec::H265_10) && video_stream.is_hdr() {
			video_filter.push(TONEMAP_FILTER.parse()?);
		}

		if fade_in > 0.0 {
			video_filter.push(format!("fade=t=in:st=0:d={fade_in:.3}"));
		}
		if fade_out > 0.0 {
			video_filter.push(format!("fade=t=out:st={fade_out_start:.3}:d={fade_out:.3}"));
		}

		let video_filter_str = video_filter.join(",");
		ffmpeg_args.add_two("-vf", video_filter_str);
	}

	// endregion

	ffmpeg_args.push(args.output.to_str().unwrap().to_string());

	ffmpeg(&ffmpeg_args, true, debug)
}
