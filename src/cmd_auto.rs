use std::time::Duration;

use anyhow::Result;

use ffauto_rs::ffmpeg::enums::{Crop, VideoCodec};
use ffauto_rs::ffmpeg::ffmpeg::ffmpeg;
use ffauto_rs::ffmpeg::ffprobe_struct::StreamType::{Audio, Video};

use crate::commands::{AutoArgs, Cli};
use crate::common::{ffprobe_output, generate_scale_filter, parse_duration, parse_seek};
use crate::vec_push_ext::PushStrExt;

pub(crate) fn ffmpeg_auto(cli: &Cli, args: &AutoArgs) -> Result<()> {
	let probe = ffprobe_output(&args.input)?;

	let first_audio_stream = probe.streams.iter().find(|s| s.codec_type == Audio);
	let first_video_stream = probe.streams.iter().find(|s| s.codec_type == Video);

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

	ffmpeg_args.add_two("-preset", args.preset.to_string());

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
			// input stream is not aac or filtering was requested, do transcode
			ffmpeg_args.add_two("-c:a", args.video_codec.audio_codec());
			ffmpeg_args.add_two("-b:a", "256k");

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

	let crf = format!("{}", &args.video_codec.crf_with_garbage(args.garbage));
	ffmpeg_args.add_two("-c:v", args.video_codec.video_codec());
	ffmpeg_args.add_two("-crf", crf);
	ffmpeg_args.add_two("-pix_fmt", args.video_codec.pix_fmt());
	ffmpeg_args.add("-tune");
	match args.video_codec {
		VideoCodec::H264 => { ffmpeg_args.add("film"); }
		VideoCodec::H265 | VideoCodec::H265_10 => {
			ffmpeg_args.add("grain");
			ffmpeg_args.add("-tag:v");
			ffmpeg_args.add("hvc1");
		}
	}

	if args.faststart {
		ffmpeg_args.add_two("-movflags", "faststart");
	}

	if args.needs_video_filter(cli) {
		let mut video_filter: Vec<String> = vec![];

		if let Some(fps) = args.framerate {
			video_filter.push(format!("fps=fps={fps:.3}"));
		} else if let (Some(fps_mult), Some(fps)) = (args.framerate_mult, video_stream.frame_rate()) {
			video_filter.push(format!("fps=fps={:.3}", fps * fps_mult));
		}

		if let Some(crop_str) = &cli.crop {
			let crop = Crop::new(crop_str)?;
			video_filter.push(format!("crop={crop}"));
		}

		let scale = generate_scale_filter(cli)?;
		if !scale.is_empty() {
			video_filter.push(scale);
		}

		let color_transfer = video_stream.color_transfer.unwrap_or_default();
		if args.tonemap && (color_transfer.contains("smpte2084") || color_transfer.contains("arib-std-b67")) {
			video_filter.add("zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709");
		} else if args.tonemap {
			eprintln!("HDR-to-SDR tonemap requested but input video is already SDR");
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

	ffmpeg(&ffmpeg_args, true, cli.debug)
}