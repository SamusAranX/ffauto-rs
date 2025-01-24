use std::time::Duration;

use anyhow::Result;

use crate::commands::{AutoArgs, Cli};
use crate::common::{add_crop_scale_tonemap_filters, add_fps_filter, ffprobe_output, parse_duration, parse_seek};
use crate::vec_push_ext::PushStrExt;
use ffauto_rs::ffmpeg::enums::{OptimizeTarget, VideoCodec};
use ffauto_rs::ffmpeg::ffmpeg::ffmpeg;
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType::Subtitle;

pub(crate) fn ffmpeg_auto(cli: &Cli, args: &AutoArgs) -> Result<()> {
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

	let seek = parse_seek(&cli.seek);
	let duration = parse_duration(seek, &args.duration, &args.duration_to);

	if let Some(seek) = seek {
		ffmpeg_args.add_two("-ss", format!("{}", seek.as_secs_f64()));
	}

	let input = args.input.as_os_str().to_str().unwrap();
	ffmpeg_args.add_two("-i", input);
	if let Some(duration) = duration {
		ffmpeg_args.add_two("-t", format!("{}", duration.as_secs_f64()));
	}

	ffmpeg_args.add_two("-metadata:s", "handler_name=\"\"");
	ffmpeg_args.add_two("-empty_hdlr_name", "1");

	// select appropriate video stream, default to the first one if no language was specified
	if let Some(video_language) = &args.video_language {
		ffmpeg_args.add_two("-map", format!("0:V:m:language:{}", video_language));
	} else {
		ffmpeg_args.add_two("-map", format!("0:V:{}", &args.video_index));
	}

	// select appropriate audio stream, default to the first one if no language was specified
	if let Some(audio_language) = &args.audio_language {
		ffmpeg_args.add_two("-map", format!("0:a:m:language:{}", audio_language));
	} else {
		ffmpeg_args.add_two("-map", format!("0:a:{}", &args.audio_index));
	}

	// select appropriate subtitle stream, default to all of them if neither of language/index was specified
	if let Some(sub_language) = &args.sub_language {
		ffmpeg_args.add_two("-map", format!("0:s:m:language:{}:?", sub_language));
	} else if let Some(sub_index) = &args.sub_index {
		ffmpeg_args.add_two("-map", format!("0:s:{}:?", sub_index));
	} else if probe.streams.iter().any(|s| s.codec_type == Subtitle && s.codec_name != Some("hdmv_pgs_subtitle".into())) {
		// there are subtitles that are not of type hdmv_pgs_subtitle, so we can actually use this
		// TODO: this might fail for files that have both usable subtitles and hdmv_pgs_subtitle subtitles
		ffmpeg_args.add_two("-map", "0:s?");
	} else {
		// there are only hdmv_pgs_subtitle subtitles, so ignore them
		ffmpeg_args.add("-sn");
	}

	let (mut fade_in, mut fade_out) = (args.fade_in, args.fade_out);
	if args.fade != 0.0 {
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

	if args.needs_video_filter(cli) {
		let mut video_filter: Vec<String> = vec![];

		add_fps_filter(&mut video_filter, args.framerate, args.framerate_mult, video_stream.frame_rate());

		let is_hdr = (args.tonemap || args.video_codec != VideoCodec::H265_10) && video_stream.is_hdr();
		add_crop_scale_tonemap_filters(&mut video_filter, cli, is_hdr)?;

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

	ffmpeg(&ffmpeg_args, true, cli.debug)
}
